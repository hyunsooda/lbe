use memmap2::MmapMut;
use std::fs;
use std::fs::OpenOptions;

pub const SHM_PATH: &str = "/tmp/fuzzer_shared_mem";
pub const SHM_AUX_PATH: &str = "/tmp/fuzzer_shared_aux_mem";
pub const SHM_COV_PATH: &str = "/tmp/fuzzer_shared_cov_mem";
pub const SHM_SIZE: usize = 1 << 16;
pub const SHM_AUX_SIZE: usize = 1 << 20;
pub const SHM_COV_SIZE: usize = 1 << 13;

pub struct SHM {
    mem: MmapMut,
    mem_path: String,
    mem_size: usize,
}

impl SHM {
    pub fn new(shm_path: &str, shm_size: usize) -> Self {
        // 1. Create shared memory file
        let _ = fs::remove_file(shm_path); // clean up any previous file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(shm_path)
            .unwrap();
        file.set_len(shm_size as u64).unwrap();

        // 2. Memory-map the file
        let m = unsafe { MmapMut::map_mut(&file).unwrap() };
        let mut mmap = Self {
            mem: m,
            mem_path: shm_path.to_string(),
            mem_size: shm_size,
        };

        // 3. Zeroize
        mmap.zeroize();
        mmap
    }

    pub fn mut_mem(&mut self) -> &mut MmapMut {
        &mut self.mem
    }

    pub fn mem(&self) -> &MmapMut {
        &self.mem
    }

    pub fn path(&self) -> &str {
        &self.mem_path
    }

    pub fn size(&self) -> usize {
        self.mem_size
    }

    pub fn zeroize(&mut self) {
        self.mem[..].fill(0);
    }
}
