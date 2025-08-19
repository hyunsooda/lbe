use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

pub fn read_seed_dir(seed_dir: &str) -> io::Result<Vec<Vec<u8>>> {
    fs::read_dir(seed_dir)?
        .filter_map(|entry| entry.ok().filter(|e| e.path().is_file()))
        .map(|entry| {
            let mut content = Vec::new();
            std::fs::File::open(entry.path())?.read_to_end(&mut content)?;
            Ok(content)
        })
        .collect()
}

pub fn write_seed(filename: &str, seed: &[u8]) -> io::Result<PathBuf> {
    let path = PathBuf::from(filename);
    let mut file = File::create(&path)?;
    file.write_all(seed)?;
    Ok(path)
}
