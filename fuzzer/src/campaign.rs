use crate::{cli::FuzzInput, seed::Seed};
use coverage_runtime::pp::CovReport;
use std::time::Duration;

#[derive(Debug)]
pub struct CrashInfo {
    pub crashes: usize,
    pub origin: Seed,
    pub minimized: Seed,
}
impl CrashInfo {
    pub fn new(crashes: usize, origin: Seed, minimized: Seed) -> Self {
        return Self {
            crashes,
            origin,
            minimized,
        };
    }
}

#[derive(Debug)]
pub struct FuzzerMetadata {
    pub fuzz_cnt: u64,
    pub fuzz_input_typ: FuzzInput,
    pub timeout: Duration,
    pub target_elpased_time: Duration,
    pub total_elpased_time: Duration,
}
impl FuzzerMetadata {
    pub fn new(
        fuzz_cnt: u64,
        fuzz_input_typ: FuzzInput,
        timeout: Duration,
        target_elpased_time: Duration,
        total_elpased_time: Duration,
    ) -> Self {
        Self {
            fuzz_cnt,
            fuzz_input_typ,
            timeout,
            target_elpased_time,
            total_elpased_time,
        }
    }
}

#[derive(Debug)]
pub struct FuzzerSeed {
    pub seeds: usize,
    pub cur_seed: Seed,
    pub next_seed: Seed,
    pub visit_edges: u64,
    pub new_paths: usize,
}
impl FuzzerSeed {
    pub fn new(
        seeds: usize,
        cur_seed: Seed,
        next_seed: Seed,
        visit_edges: u64,
        new_paths: usize,
    ) -> Self {
        Self {
            seeds,
            cur_seed,
            next_seed,
            visit_edges,
            new_paths,
        }
    }
}

#[derive(Debug)]
pub enum FuzzShot {
    ProgramOutput(String),
    Coverage(Vec<CovReport>),
    Crash(CrashInfo),
    Metadata(FuzzerMetadata),
    SeedInfo(FuzzerSeed),
    Terminated,
}

#[derive(Debug)]
pub struct FuzzingCampaign {
    pub program_output: Vec<String>,
    pub coverage: Vec<CovReport>,
    pub crash: CrashInfo,
    pub metadata: FuzzerMetadata,
    pub seed_info: FuzzerSeed,
}
impl FuzzingCampaign {
    pub fn new(fuzz_input: FuzzInput) -> Self {
        Self {
            program_output: vec!["".to_string()],
            coverage: vec![CovReport {
                file: "".to_string(),
                funcs_hit_ratio: "".to_string(),
                uncovered_funcs: "".to_string(),
                brs_hit_ratio: "".to_string(),
                uncovered_brs: "".to_string(),
                lines_hit_ratio: "".to_string(),
                uncovered_lines: "".to_string(),
            }],
            crash: CrashInfo::new(0, Seed::new(vec![], 0), Seed::new(vec![], 0)),
            metadata: FuzzerMetadata::new(
                0,
                fuzz_input,
                Duration::default(),
                Duration::default(),
                Duration::default(),
            ),
            seed_info: FuzzerSeed::new(0, Seed::new(vec![], 0), Seed::new(vec![], 0), 0, 0),
        }
    }
    pub fn add_program_output(&mut self, pgm_output: String) {
        self.program_output.push(pgm_output);
    }
    pub fn set_coverage(&mut self, coverage: Vec<CovReport>) {
        if !coverage.is_empty() {
            self.coverage = coverage;
        }
    }
    pub fn set_crash(&mut self, crash: CrashInfo) {
        self.crash = crash;
    }
    pub fn set_metadata(&mut self, metadata: FuzzerMetadata) {
        self.metadata = metadata;
    }
    pub fn set_seed_info(&mut self, seed_info: FuzzerSeed) {
        self.seed_info = seed_info;
    }
}
