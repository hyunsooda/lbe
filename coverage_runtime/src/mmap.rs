use memmap2::MmapMut;
use std::env;
use std::fs::OpenOptions;

pub fn init_shm(id: &str, size_id: &str) -> (Option<MmapMut>, Option<usize>) {
    let ret = set_mmap(id);
    let mut shm_size = None;
    if let Ok(size) = env::var(size_id) {
        shm_size = Some(size.parse::<usize>().unwrap());
    }
    (ret, shm_size)
}

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
