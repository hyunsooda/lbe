use defer_lite::defer;
use fuzzer::fuzzer::Fuzzer;
use fuzzer::{fuzzer::HostSend, mmap::SHM_COV_SIZE};
use fuzzer::{
    mmap::{SHM_AUX_SIZE, SHM_SIZE},
    seed::SeedPool,
};
use fuzzer_runtime::runtime::__fuzzer_trace_edge;
use serial_test::serial;
use std::env;
use std::fs;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

fn create_test_dir(dirname: &str) {
    fs::create_dir(dirname).unwrap();
}

fn remove_test_dir(dirname: &str) {
    fs::remove_dir(dirname).unwrap();
}

fn gen_filename() -> String {
    format!("/tmp/{}", Uuid::new_v4())
}

fn set_env(
    shm_path: &str,
    shm_size: usize,
    shm_aux_path: &str,
    shm_aux_size: usize,
    forkserver_host: Option<i32>,
    forkserver_runtime: Option<i32>,
) {
    env::set_var("SHM_ID", shm_path);
    env::set_var("SHM_AUX_ID", shm_aux_path);
    env::set_var("SHM_SIZE", format!("{}", shm_size));
    env::set_var("SHM_AUX_SIZE", format!("{}", shm_aux_size));
    if let Some(forkserver_host) = forkserver_host {
        env::set_var("FORK_SERVER_HOST", format!("{}", forkserver_host));
    }
    if let Some(forkserver_runtime) = forkserver_runtime {
        env::set_var("FORK_SERVER_RUNTIME", format!("{}", forkserver_runtime));
    }
}

#[test]
#[serial]
fn test_path_coverage() {
    let test_dirname = format!("{}", Uuid::new_v4());
    create_test_dir(&test_dirname);
    defer! {
        remove_test_dir(&test_dirname);
    }
    let (shm_path, shm_aux_path, shm_cov_path) = (gen_filename(), gen_filename(), gen_filename());
    let (tx, _) = mpsc::channel();
    let mut fuzzer = Fuzzer::new(
        &shm_path,
        SHM_SIZE,
        &shm_aux_path,
        SHM_AUX_SIZE,
        &shm_cov_path,
        SHM_COV_SIZE,
        SeedPool::new(&test_dirname),
        fuzzer::cli::FuzzInput::Stdin,
        tx,
    );

    set_env(&shm_path, SHM_SIZE, &shm_aux_path, SHM_AUX_SIZE, None, None);
    fuzzer_runtime::runtime::__init();

    let (a, b, c, d, e) = (100, 200, 300, 400, 500);
    // path: a -> b -> c -> d -> e -> a
    __fuzzer_trace_edge(a);
    assert!(fuzzer.is_new_coverage());
    __fuzzer_trace_edge(b);
    assert!(fuzzer.is_new_coverage());
    __fuzzer_trace_edge(c);
    assert!(fuzzer.is_new_coverage());
    __fuzzer_trace_edge(d);
    assert!(fuzzer.is_new_coverage());
    __fuzzer_trace_edge(e);
    assert!(fuzzer.is_new_coverage());
    __fuzzer_trace_edge(a);
    assert!(fuzzer.is_new_coverage());

    fuzzer.clear_new_coverage();

    // known path: a -> b -> c -> d -> e
    // new   path: a -> b
    __fuzzer_trace_edge(b);
    assert!(!fuzzer.is_new_coverage());

    // known path: a -> b -> c -> d -> e
    // new   path: a -> b -> c
    __fuzzer_trace_edge(c);
    assert!(!fuzzer.is_new_coverage());

    // known path: a -> b -> c -> d -> e
    // new   path: a -> b -> c -> d
    __fuzzer_trace_edge(d);
    assert!(!fuzzer.is_new_coverage());

    // known path: a -> b -> c -> d -> e
    // new   path: a -> b -> c -> d -> e
    __fuzzer_trace_edge(e);
    assert!(!fuzzer.is_new_coverage());
}

#[test]
#[serial]
fn test_coverage_bucket() {
    let test_dirname = format!("{}", Uuid::new_v4());
    create_test_dir(&test_dirname);
    defer! {
        remove_test_dir(&test_dirname);
    }
    let (tx, _) = mpsc::channel();
    let (shm_path, shm_aux_path, shm_cov_path) = (gen_filename(), gen_filename(), gen_filename());
    let mut fuzzer = Fuzzer::new(
        &shm_path,
        SHM_SIZE,
        &shm_aux_path,
        SHM_AUX_SIZE,
        &shm_cov_path,
        SHM_COV_SIZE,
        SeedPool::new(&test_dirname),
        fuzzer::cli::FuzzInput::Stdin,
        tx,
    );

    set_env(&shm_path, SHM_SIZE, &shm_aux_path, SHM_AUX_SIZE, None, None);
    fuzzer_runtime::runtime::__init();

    let (a, b, c) = (100, 200, 300);
    // path: a -> b -> a
    //       (a,b) , (b,a)
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    __fuzzer_trace_edge(a);
    assert!(fuzzer.eval_seed(0) == (3, u64::MAX));
    fuzzer.clear_new_coverage();

    // path: a -> b
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 15));
    fuzzer.clear_new_coverage();

    // path: b -> a -> b
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 14));
    fuzzer.clear_new_coverage();

    // path: b -> a -> b
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 13));
    fuzzer.clear_new_coverage();

    // path: b -> a -> b
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 13));
    fuzzer.clear_new_coverage();

    // path: b -> a -> b
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 13));
    fuzzer.clear_new_coverage();

    // path: b -> a -> b
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 13));
    fuzzer.clear_new_coverage();

    // path: b -> a -> b
    __fuzzer_trace_edge(a);
    __fuzzer_trace_edge(b);
    assert!(fuzzer.eval_seed(0) == (3, 12));
    fuzzer.clear_new_coverage();

    // path: b -> c
    __fuzzer_trace_edge(c);
    assert!(fuzzer.eval_seed(0) == (4, u64::MAX));
}

#[test]
#[serial]
fn test_coverage_wakeup() {
    let test_dirname = format!("{}", Uuid::new_v4());
    create_test_dir(&test_dirname);
    let (tx, _) = mpsc::channel();
    let (shm_path, shm_aux_path, shm_cov_path) = (gen_filename(), gen_filename(), gen_filename());
    let mut fuzzer = Fuzzer::new(
        &shm_path,
        SHM_SIZE,
        &shm_aux_path,
        SHM_AUX_SIZE,
        &shm_cov_path,
        SHM_COV_SIZE,
        SeedPool::new(&test_dirname),
        fuzzer::cli::FuzzInput::Stdin,
        tx,
    );

    set_env(
        &shm_path,
        SHM_SIZE,
        &shm_aux_path,
        SHM_AUX_SIZE,
        Some(fuzzer.forkserver_host),
        Some(fuzzer.forkserver_runtime),
    );
    fuzzer_runtime::runtime::__init();

    thread::spawn(move || {
        fuzzer_runtime::runtime::__fuzzer_forkserver_init();
    });
    fuzzer.wakeup_forkserver(HostSend::Wakeup(1));
    fuzzer.wait_forkserver();
    fuzzer.wakeup_forkserver(HostSend::Terminate);
    remove_test_dir(&test_dirname);
}
