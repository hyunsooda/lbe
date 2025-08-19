use std::cmp::{max, min};

pub type Data = Vec<u8>;

#[derive(PartialEq, Debug)]
pub enum TestResult {
    Pass,
    Fail,
}

pub type TestFn = Box<dyn Fn(&Data) -> TestResult>;

pub fn ddmin(data: &Data, test: TestFn) -> Data {
    do_ddmin(data, 2, test)
}

fn do_ddmin(data: &Data, n: usize, test: TestFn) -> Data {
    let (delta_set, complement_set) = split(data, n);
    for delta in &delta_set {
        if test(delta) == TestResult::Fail {
            if delta.len() == 1 {
                return delta.to_vec();
            }
            return do_ddmin(delta, 2, test);
        }
    }
    for complement in &complement_set {
        if test(complement) == TestResult::Fail {
            return do_ddmin(complement, max(n - 1, 2), test);
        }
    }
    if n < data.len() {
        return do_ddmin(data, min(data.len(), 2 * n), test);
    }
    data.to_vec()
}

/// Return delta and complement
pub fn split(data: &Data, n: usize) -> (Vec<Data>, Vec<Data>) {
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    let data_len = data.len();
    let exact_chunk_size = data_len / n;
    let remainder = data_len % n;

    let mut delta_boundaries = Vec::new();
    let mut cur_pos = 0;

    for i in 0..n {
        let mut chunk_size = exact_chunk_size;
        if i < remainder {
            chunk_size += 1;
        }
        if cur_pos + chunk_size > data_len {
            break;
        }
        delta_boundaries.push((cur_pos, cur_pos + chunk_size));
        cur_pos += chunk_size;
    }

    let delta_set: Vec<Data> = delta_boundaries
        .iter()
        .map(|&(start, end)| data[start..end].to_vec())
        .collect();

    let mut complement_set: Vec<Data> = Vec::new();
    for i in 0..delta_set.len() {
        let (start, end) = delta_boundaries[i];
        let mut complement_bytes_raw = Vec::new();
        if start > 0 {
            complement_bytes_raw.extend_from_slice(&data[0..start]);
        }
        if end < data_len {
            complement_bytes_raw.extend_from_slice(&data[end..data_len]);
        }
        complement_set.push(complement_bytes_raw);
    }
    let filtered_complements: Vec<Data> = complement_set
        .into_iter()
        .filter(|c| !delta_set.contains(c))
        .collect();
    (delta_set, filtered_complements)
}
