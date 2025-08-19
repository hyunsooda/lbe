use coverage_runtime::pp::CovReport;
use memmap2::MmapMut;
use std::env;
use std::fs::OpenOptions;

pub fn set_mmap(id: &str) -> Option<MmapMut> {
    match env::var(id) {
        Ok(path) => {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .expect("Failed to open SHM file");
            let mmap = unsafe { MmapMut::map_mut(&file).expect("mmap failed in runtime") };
            Some(mmap)
        }
        Err(_) => None,
    }
}

pub fn init_shm(id: &str, size_id: &str) -> (Option<MmapMut>, Option<usize>) {
    let ret = set_mmap(id);
    let mut shm_size = None;
    if let Ok(size) = env::var(size_id) {
        shm_size = Some(size.parse::<usize>().unwrap());
    }
    (ret, shm_size)
}

pub fn init_forkserver_fd(id: &str) -> Option<i32> {
    if let Ok(fd) = env::var(id) {
        return Some(fd.parse::<i32>().unwrap());
    }
    None
}

pub fn read_u64(mem: &[u8], start: usize) -> u64 {
    let mut slice = [0u8; 8];
    slice.copy_from_slice(&mem[start..start + 8]);
    u64::from_le_bytes(slice)
}

pub fn read_u128(mem: &[u8], start: usize) -> u128 {
    let mut slice = [0u8; 16];
    slice.copy_from_slice(&mem[start..start + 16]);
    u128::from_le_bytes(slice)
}

pub fn read_cov_report(mem: &[u8]) -> Vec<CovReport> {
    let size = read_u64(mem, 0) as usize;
    let mut slice = vec![0u8; size];
    slice.copy_from_slice(&mem[8..8 + size]);
    if let Ok(des) = bincode::deserialize(&slice) {
        des
    } else {
        vec![]
    }
}

pub fn write_u64(mem: &mut [u8], start: usize, v: usize) {
    mem[start..start + 8].copy_from_slice(&v.to_le_bytes())
}

pub fn write_u128(mem: &mut [u8], start: usize, v: u128) {
    mem[start..start + 16].copy_from_slice(&v.to_le_bytes())
}
