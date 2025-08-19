use anyhow::Result;
use fuzzer::{
    cli::get_args,
    fuzzer::Fuzzer,
    mmap::{SHM_AUX_PATH, SHM_AUX_SIZE, SHM_COV_PATH, SHM_COV_SIZE, SHM_PATH, SHM_SIZE},
    seed::SeedPool,
    ui::run_ui,
};
use std::sync::mpsc;

fn main() -> Result<()> {
    let (pgm_path, seed_dir_path, input_typ) = get_args()?;
    let init_seeds = SeedPool::new(&seed_dir_path);
    let (tx, rx) = mpsc::channel();
    let mut fuzzer = Fuzzer::new(
        SHM_PATH,
        SHM_SIZE,
        SHM_AUX_PATH,
        SHM_AUX_SIZE,
        SHM_COV_PATH,
        SHM_COV_SIZE,
        init_seeds,
        input_typ,
        tx,
    );
    let handle = std::thread::spawn(move || {
        let _ = run_ui(rx);
    });
    let _ = fuzzer.run(&pgm_path);
    handle.join().unwrap();
    Ok(())
}
