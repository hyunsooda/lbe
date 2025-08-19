use std::hash::{Hash, Hasher};
use std::{collections::BTreeSet, fs, path::PathBuf};

use crate::{
    mutator::{self, MutateResult},
    util::read_seed_dir,
};

const MAX_SEED_LEN: usize = 1000;
const CRASH_OUTPUT_DIR: &str = "crashes";

#[derive(Debug, Clone)]
pub struct Seed {
    input: Vec<u8>,
    score: u64,
}

impl Seed {
    pub fn new(input: Vec<u8>, score: u64) -> Self {
        Self { input, score }
    }

    pub fn mutate(&mut self) -> Vec<MutateResult> {
        mutator::mutate(&mut self.input)
    }

    pub fn get_input(&self) -> &[u8] {
        &self.input
    }

    pub fn get_score(&self) -> u64 {
        self.score
    }

    pub fn to_hex(&self) -> String {
        self.get_input()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn to_file(&self, cnt: usize) {
        fs::create_dir_all(CRASH_OUTPUT_DIR).unwrap();
        let mut file_path = PathBuf::from(CRASH_OUTPUT_DIR);
        file_path.push(format!("{}.crash", cnt));
        fs::write(&file_path, self.input.clone()).unwrap();
    }

    pub fn set_score(&mut self, v: u64) {
        self.score = v;
    }
}

impl Ord for Seed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.input.cmp(&other.input))
    }
}

impl PartialOrd for Seed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Seed {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input && self.score == other.score
    }
}

impl Eq for Seed {}

impl Hash for Seed {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.score.hash(state);
    }
}

pub struct SeedPool {
    seeds: BTreeSet<Seed>,
}

impl SeedPool {
    pub fn new(seed_dir: &str) -> Self {
        let mut btree = BTreeSet::new();
        let init_seeds = read_seed_dir(seed_dir).unwrap();
        for init_seed in init_seeds {
            btree.insert(Seed::new(init_seed, 0));
        }
        Self { seeds: btree }
    }

    pub fn add_seed(&mut self, seed: Seed) {
        if self.seeds.len() > MAX_SEED_LEN {
            if let Some(min_seed) = self.get_min_score_seed() {
                self.seeds.remove(&min_seed);
            }
        }
        self.seeds.insert(seed);
    }

    fn get_min_score_seed(&self) -> Option<Seed> {
        self.seeds.iter().next().cloned()
    }

    fn get_max_score_seed(&self) -> Option<Seed> {
        self.seeds.iter().next_back().cloned()
    }

    pub fn pop_seed(&mut self) -> Option<Seed> {
        if let Some(seed) = self.get_max_score_seed() {
            self.seeds.remove(&seed);
            Some(seed)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.seeds.len() == 0
    }

    pub fn len(&self) -> usize {
        self.seeds.len()
    }
}
