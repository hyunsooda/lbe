use rand::Rng;

// 1. random insert
// 2. random delete
// e. random change
// 4. random flip
// 5. arithmetic mutation

const KEEP_SEED_MIN_LEN: usize = 1;

#[derive(PartialEq, Eq, Debug)]
pub enum MutateResult {
    EmptyInput,
    SeedTooShortToDelete,
    SeedTooShortToArithmeticMutate,
    Done,
}

fn gen_new_byte() -> u8 {
    rand::random::<u8>()
}

fn gen_random_idx(len: usize) -> usize {
    rand::random::<usize>() % len
}

type MutatorFn = fn(&mut Vec<u8>) -> MutateResult;

const MUTATORS: &[MutatorFn] = &[insert, change, delete, flip, arithmetic];

fn insert(seed: &mut Vec<u8>) -> MutateResult {
    if seed.is_empty() {
        return MutateResult::EmptyInput;
    }
    let idx = gen_random_idx(seed.len());
    seed.insert(idx, gen_new_byte());
    MutateResult::Done
}

fn change(seed: &mut Vec<u8>) -> MutateResult {
    if seed.is_empty() {
        return MutateResult::EmptyInput;
    }
    let idx = gen_random_idx(seed.len());
    seed[idx] = gen_new_byte();
    MutateResult::Done
}

fn delete(seed: &mut Vec<u8>) -> MutateResult {
    if seed.is_empty() {
        return MutateResult::EmptyInput;
    }
    let idx = gen_random_idx(seed.len());
    if seed.len() > KEEP_SEED_MIN_LEN {
        seed.remove(idx);
        MutateResult::Done
    } else {
        MutateResult::SeedTooShortToDelete
    }
}

fn flip(seed: &mut Vec<u8>) -> MutateResult {
    if seed.is_empty() {
        return MutateResult::EmptyInput;
    }
    let idx = gen_random_idx(seed.len());
    seed[idx] ^= 0xFF;
    MutateResult::Done
}

fn arithmetic(seed: &mut Vec<u8>) -> MutateResult {
    if seed.is_empty() {
        return MutateResult::EmptyInput;
    }
    let mut rng = rand::thread_rng();
    let mutate_widths = [1, 2, 4];
    let width = match seed.len() {
        0..=1 => return MutateResult::SeedTooShortToArithmeticMutate,
        2..=3 => 1,
        4 => mutate_widths[rng.gen_range(0..2)],
        _ => mutate_widths[rng.gen_range(0..mutate_widths.len())],
    };
    let idx = rng.gen_range(0..=seed.len() - width);
    // choose a random value from -35 ~ +35
    let delta = rng.gen_range(-35i64..=35);
    match width {
        1 => {
            let val = seed[idx];
            let new_val = val.wrapping_add(delta as u8);
            seed[idx] = new_val;
        }
        2 => {
            let val = u16::from_le_bytes([seed[idx], seed[idx + 1]]);
            let new_val = val.wrapping_add(delta as u16);
            let bytes = new_val.to_le_bytes();
            seed[idx..idx + 2].copy_from_slice(&bytes);
        }
        4 => {
            let val = u32::from_le_bytes([seed[idx], seed[idx + 1], seed[idx + 2], seed[idx + 3]]);
            let new_val = val.wrapping_add(delta as u32);
            let bytes = new_val.to_le_bytes();
            seed[idx..idx + 4].copy_from_slice(&bytes);
        }
        _ => unreachable!(),
    }
    MutateResult::Done
}

pub fn mutate(seed: &mut Vec<u8>) -> Vec<MutateResult> {
    // 50% mutation example
    // let n = if seed.len() <= 1 {
    //     seed.len()
    // } else {
    //     gen_random_idx(seed.len() / 2) // mutate 50% for a given seed
    // };

    // muate 1 ~ 10% for a given seed
    let mut rng = rand::thread_rng();
    let max = ((seed.len() as f32) * 0.2).ceil() as usize;
    let max = max.max(1).min(seed.len());
    let n = rng.gen_range(1..=max);

    let mut results = vec![];
    for _ in 0..=n {
        let idx = gen_random_idx(MUTATORS.len());
        let result = MUTATORS[idx](seed);
        results.push(result);
    }
    results
}
