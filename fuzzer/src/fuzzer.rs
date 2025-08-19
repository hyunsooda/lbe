use crate::{
    bucket::get_score,
    campaign::{FuzzShot, FuzzerSeed},
    mmap::SHM,
    seed::{Seed, SeedPool},
    util::write_seed,
};
use crate::{
    campaign::{CrashInfo, FuzzerMetadata},
    cli::FuzzInput,
};
use anyhow::Result;
use fuzzer_runtime::{
    coverage::PROCESS_EXIT_NORMAL,
    internal::{read_cov_report, read_u64, write_u64},
};
use libc::{c_void, eventfd, read, write, SIGKILL};
use std::{
    cmp::{max, min},
    collections::HashSet,
    os::unix::io::RawFd,
    process::{ChildStderr, ChildStdin, ChildStdout},
    thread,
};
use std::{
    io::Write,
    mem,
    process::{Command, Stdio},
};
use std::{
    io::{BufRead, BufReader},
    sync::mpsc,
};

use delta_debugging::{split, TestResult};
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

const NEW_COVERAGES: usize = fuzzer_runtime::coverage::NEW_COVERAGES;
const VISIT_EDGES_INDICIES: usize = fuzzer_runtime::coverage::VISIT_EDGES_INDICIES;
const VISIT_MARK: usize = fuzzer_runtime::coverage::VISIT_MARK;
const VISIT_EDGES_INDEX_SIZE: usize = fuzzer_runtime::coverage::VISIT_EDGES_INDEX_SIZE;

const INITIAL_TIMEOUT_UPPER_BOUND: u32 = 5;

pub enum HostSend {
    Terminate,   // 1
    Wakeup(u64), // 2,
}

#[derive(Debug)]
pub enum FuzzResult {
    AllSeedConsumed(u64),
    UserTerminated,
    Success,
}

pub struct Fuzzer {
    shm: SHM,
    shm_aux: SHM,
    shm_cov: SHM,
    pub forkserver_host: i32,
    pub forkserver_runtime: i32,
    seeds: SeedPool,
    new_paths: usize,
    input_typ: FuzzInput,
    seed_file_path: String, // if fuzz input is `ProgramArgument`, we write seed into this file path
    crashes: HashSet<Seed>,
    tx: mpsc::Sender<FuzzShot>,
    fuzz_terminate: bool,
}

impl Fuzzer {
    pub fn new(
        shm_path: &str,
        shm_size: usize,
        shm_aux_path: &str,
        shm_aux_size: usize,
        shm_cov_path: &str,
        shm_cov_size: usize,
        init_seeds: SeedPool,
        input_typ: FuzzInput,
        tx: mpsc::Sender<FuzzShot>,
    ) -> Self {
        let shm = SHM::new(shm_path, shm_size);
        let shm_aux = SHM::new(shm_aux_path, shm_aux_size);
        let shm_cov = SHM::new(shm_cov_path, shm_cov_size);
        let host_efd: RawFd = unsafe { eventfd(0, 0) };
        let runtime_efd: RawFd = unsafe { eventfd(0, 0) };
        Self {
            shm,
            shm_aux,
            shm_cov,
            forkserver_host: host_efd,
            forkserver_runtime: runtime_efd,
            seeds: init_seeds,
            new_paths: 0,
            input_typ,
            seed_file_path: format!("{}.seed", Uuid::new_v4()),
            crashes: HashSet::new(),
            tx,
            fuzz_terminate: false,
        }
    }

    pub fn wakeup_forkserver(&self, kind: HostSend) {
        let value = match kind {
            HostSend::Wakeup(timeout) => {
                if timeout == 0 {
                    1
                } else {
                    timeout
                }
            }
            HostSend::Terminate => 99999999999_u64,
        };
        let bytes = value.to_ne_bytes();
        unsafe {
            write(
                self.forkserver_runtime,
                bytes.as_ptr() as *const _,
                bytes.len(),
            )
        };
    }

    pub fn wait_forkserver(&mut self) -> i32 {
        let mut status: u64 = 0;
        unsafe {
            read(
                self.forkserver_host,
                &mut status as *mut _ as *mut c_void,
                mem::size_of::<u64>(),
            );
        }
        status as i32
    }

    fn is_crash(&self, status: i32) -> bool {
        status != PROCESS_EXIT_NORMAL as i32 && status != SIGKILL
    }

    fn oracle(
        &mut self,
        child_stdin: &mut Option<ChildStdin>,
        timeout: Duration,
        seed: &Seed,
    ) -> Result<TestResult> {
        self.feed_seed(child_stdin, seed)?;
        self.wakeup_forkserver(HostSend::Wakeup(timeout.as_secs()));
        let status = self.wait_forkserver();
        self.clear_new_coverage();
        self.clear_visited_edges();
        if self.is_crash(status) {
            Ok(TestResult::Fail)
        } else {
            Ok(TestResult::Pass)
        }
    }

    fn ddmin(
        &mut self,
        child_stdin: &mut Option<ChildStdin>,
        timeout: Duration,
        seed: &Seed,
    ) -> Result<Seed> {
        self.do_ddmin(child_stdin, timeout, seed, 2)
    }

    fn do_ddmin(
        &mut self,
        child_stdin: &mut Option<ChildStdin>,
        timeout: Duration,
        seed: &Seed,
        n: usize,
    ) -> Result<Seed> {
        let (delta_set, complement_set) = split(&seed.get_input().to_vec(), n);
        for delta in &delta_set {
            let delta_seed = Seed::new(delta.to_vec(), 0);
            if let Ok(test_result) = self.oracle(child_stdin, timeout, &delta_seed) {
                if test_result == TestResult::Fail {
                    if delta.len() == 1 {
                        return Ok(delta_seed);
                    }
                    return self.do_ddmin(child_stdin, timeout, &delta_seed, 2);
                }
            }
        }
        for complement in &complement_set {
            let complement_seed = Seed::new(complement.to_vec(), 0);
            if let Ok(test_result) = self.oracle(child_stdin, timeout, &complement_seed) {
                if test_result == TestResult::Fail {
                    return self.do_ddmin(child_stdin, timeout, &complement_seed, max(n - 1, 2));
                }
            }
        }
        let seed_len = seed.get_input().len();
        if n < seed_len {
            return self.do_ddmin(child_stdin, timeout, seed, min(seed_len, 2 * n));
        }
        Ok(seed.clone())
    }

    pub fn clear_new_coverage(&mut self) {
        write_u64(self.shm_aux.mut_mem(), NEW_COVERAGES, 0);
    }

    pub fn clear_visited_edges(&mut self) {
        let visit_edges = read_u64(self.shm_aux.mem(), VISIT_MARK);
        for i in 0..visit_edges {
            write_u64(
                self.shm_aux.mut_mem(),
                VISIT_EDGES_INDICIES + (i as usize) * VISIT_EDGES_INDEX_SIZE,
                0,
            );
        }
        write_u64(self.shm_aux.mut_mem(), VISIT_MARK, 0);
    }

    fn add_seed(&mut self, seed: Seed) {
        self.seeds.add_seed(seed);
    }

    fn pop_seed(&mut self) -> Option<Seed> {
        self.seeds.pop_seed()
    }

    fn is_seed_empty(&self) -> bool {
        self.seeds.is_empty()
    }

    pub fn is_new_coverage(&self) -> bool {
        let new_covs = read_u64(self.shm_aux.mem(), NEW_COVERAGES);
        new_covs != 0
    }

    fn feed_seed(&self, child_stdin: &mut Option<ChildStdin>, seed: &Seed) -> Result<()> {
        if let Some(ref mut stdin) = child_stdin {
            stdin.write_all(seed.get_input())?;
        }
        if self.input_typ == FuzzInput::ProgramArgument {
            write_seed(&self.seed_file_path, seed.get_input())?;
        }
        Ok(())
    }

    pub fn eval_seed(&mut self, status: i32) -> (u64, u64) {
        if status == SIGKILL {
            return (0, 0); // give zero score for hangs
        }
        let visit_edges = read_u64(self.shm_aux.mem(), VISIT_MARK);
        // if new coverage found, give max score, otherwise give higher score if the path is rare
        if self.is_new_coverage() {
            self.new_paths += 1;
            return (visit_edges, u64::MAX);
        }
        let mut score = 0;
        for i in 0..visit_edges {
            let edge = read_u64(
                self.shm_aux.mem(),
                VISIT_EDGES_INDICIES + (i as usize) * VISIT_EDGES_INDEX_SIZE,
            ) as usize;
            if edge != 0 {
                score += get_score(self.shm.mem()[edge]) as u64;
            }
        }
        (visit_edges, score)
    }

    fn spawn_reader_thread<T>(
        &self,
        reader_stream: T,
        sender: mpsc::Sender<FuzzShot>,
        stream_name: &'static str,
    ) where
        T: std::io::Read + Send + 'static,
    {
        thread::spawn(move || {
            let reader = BufReader::new(reader_stream);
            let mut reader_iter = reader.lines();

            loop {
                match reader_iter.next() {
                    Some(Ok(line)) => {
                        let _ = sender.send(FuzzShot::ProgramOutput(line));
                    }
                    Some(Err(e)) => {
                        panic!("{} reader error: {:?}", stream_name, e);
                    }
                    None => {
                        break;
                    }
                }
            }
        });
    }

    fn pgm_output_reader(&mut self, stdout: ChildStdout, stderr: ChildStderr) {
        self.spawn_reader_thread(stdout, self.tx.clone(), "stdout");
        self.spawn_reader_thread(stderr, self.tx.clone(), "stderr");
    }

    fn send(&mut self, fuzz_result: FuzzShot) {
        if self.tx.send(fuzz_result).is_err() {
            self.fuzz_terminate = true;
        }
    }

    fn terminate(&self) {
        self.wakeup_forkserver(HostSend::Terminate);
    }

    fn debug(
        &mut self,
        loop_cnt: u64,
        _seed: &Seed,
        _score: u64,
        timeout: Duration,
        target_elapsed: Duration,
        total_elapsed: Duration,
    ) {
        let cov_report = read_cov_report(self.shm_cov.mem());
        self.send(FuzzShot::Coverage(cov_report));
        self.send(FuzzShot::Metadata(FuzzerMetadata::new(
            loop_cnt,
            self.input_typ,
            timeout,
            target_elapsed,
            total_elapsed,
        )));

        // Uncomment if debugging is required without UI
        // println!(
        //     "Fuzz#{}, target_elapsed:{:?}, total_seeds:{}, seed:{}, crashes:{}, score:{}, new_paths:{}",
        //     loop_cnt,
        //     target_elapsed,
        //     self.seeds.len(),
        //     &_seed.to_hex(),
        //     self.crashes.len(),
        //     _score,
        //     self.new_paths,
        // );
    }

    pub fn run(&mut self, program_file: &str) -> Result<FuzzResult> {
        let mut cmd = Command::new(program_file);
        let child_process_cmd = cmd
            .env("LD_LIBRARY_PATH", ".")
            .env("SHM_ID", self.shm.path())
            .env("SHM_AUX_ID", self.shm_aux.path())
            .env("SHM_SIZE", format!("{}", &self.shm.size()))
            .env("SHM_AUX_SIZE", format!("{}", &self.shm_aux.size()))
            .env("SHM_COV_ID", self.shm_cov.path())
            .env("SHM_COV_SIZE", format!("{}", &self.shm_cov.size()))
            .env("FORK_SERVER_HOST", format!("{}", self.forkserver_host))
            .env(
                "FORK_SERVER_RUNTIME",
                format!("{}", self.forkserver_runtime),
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if self.input_typ == FuzzInput::ProgramArgument {
            child_process_cmd.args(vec![&self.seed_file_path]);
        } else {
            child_process_cmd.stdin(Stdio::piped());
        }

        let mut child_process = child_process_cmd.spawn()?;
        let mut child_stdin = if self.input_typ == FuzzInput::Stdin {
            Some(child_process.stdin.take().unwrap())
        } else {
            None
        };

        self.pgm_output_reader(
            child_process.stdout.take().unwrap(),
            child_process.stderr.take().unwrap(),
        );
        let mut init_set_timeout = false;
        let mut timeout = Duration::new(9999, 0);
        let fuzzer_started = Instant::now();

        let mut loop_cnt = 0;
        if self.is_seed_empty() {
            return Ok(FuzzResult::AllSeedConsumed(loop_cnt));
        }
        let mut seed = self.pop_seed().unwrap();

        loop {
            if self.fuzz_terminate {
                self.terminate();
                self.send(FuzzShot::Terminated);
                return Ok(FuzzResult::UserTerminated);
            }
            loop_cnt += 1;
            self.feed_seed(&mut child_stdin, &seed)?;
            self.wakeup_forkserver(HostSend::Wakeup(timeout.as_secs()));
            let target_started = Instant::now();
            let status = self.wait_forkserver();

            let elapsed = target_started.elapsed();
            if !init_set_timeout {
                timeout = elapsed * INITIAL_TIMEOUT_UPPER_BOUND;
                init_set_timeout = true;
            }
            // evalulate seed and add it
            let (visit_edges, score) = self.eval_seed(status);
            self.debug(
                loop_cnt,
                &seed,
                score,
                timeout,
                elapsed,
                fuzzer_started.elapsed(),
            );
            self.clear_new_coverage();
            self.clear_visited_edges();
            if self.is_seed_empty() {
                self.terminate();
                self.send(FuzzShot::Terminated);
                return Ok(FuzzResult::AllSeedConsumed(loop_cnt));
            }

            // crash found
            if self.is_crash(status) {
                if let Ok(minimized) = self.ddmin(&mut child_stdin, timeout, &seed) {
                    self.crashes.insert(minimized.clone());
                    minimized.to_file(self.crashes.len());
                    self.send(FuzzShot::Crash(CrashInfo::new(
                        self.crashes.len(),
                        seed,
                        minimized,
                    )));
                }
            } else if score > 0 {
                // re-evaulate score after execution
                seed.set_score(score);
                self.add_seed(seed.clone());
                let cur_seed = seed.clone();
                seed.mutate();
                seed.set_score(score.saturating_add(1));
                self.add_seed(seed.clone());

                self.send(FuzzShot::SeedInfo(FuzzerSeed::new(
                    self.seeds.len(),
                    cur_seed,
                    seed,
                    visit_edges,
                    self.new_paths,
                )));
            }
            seed = self.pop_seed().unwrap();
        }
    }
}
