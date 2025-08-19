const BUCKET_MAX_VALUE: u8 = 8;
const COUNT_CLASS_LOOKUP: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = match i {
            0..=1 => 1,
            2 => 2,
            3 => 3,
            4..=7 => 4,
            8..=15 => 5,
            16..=31 => 6,
            32..=127 => 7,
            _ => BUCKET_MAX_VALUE,
        };
        i += 1;
    }
    table
};

/// lower value is more valuable because it has been not rarely exercised
fn to_bucket(value: u8) -> u8 {
    COUNT_CLASS_LOOKUP[value as usize]
}

pub fn get_score(value: u8) -> u8 {
    let score = to_bucket(value);
    (BUCKET_MAX_VALUE + 1) - score
}
