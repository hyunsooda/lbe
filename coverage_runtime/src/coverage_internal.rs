use crate::{
    coverage_runtime::{SourceMapping, COVERAGE_STATE, SHM_COV},
    pp::{CovReport, TableFormatter},
    util::{get_intersect, get_symmetric_diff},
};
use std::collections::HashSet;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::sync::atomic::Ordering;

const COVERAGE_DEBUG: &str = "COVERAGE_DEBUG";

pub fn cov_clear() {
    let mut state = COVERAGE_STATE.write().unwrap();
    state.source_map.lock().unwrap().clear();
    state.location_map.lock().unwrap().clear();
    state.lines.clear();
    state.enabled.store(1, Ordering::Relaxed);
}

pub fn write_coverage_data(color_enabled: usize) {
    let reports = make_cov();
    let mut tbl_reports = TableFormatter::format(TableFormatter::TableWithoutColor, &reports);
    // Write report into file without ANSI color
    std::fs::write("cov.out", &tbl_reports).unwrap();

    if color_enabled == 1 {
        tbl_reports = TableFormatter::format(TableFormatter::TableWithColor, &reports);
    }
    println!("{}", tbl_reports);
}

pub fn write_cov_shm() {
    let mut shm_cov = SHM_COV.write().unwrap();
    if let Some(ref mut mem) = &mut shm_cov.mem {
        let cov_report = make_cov();
        let encoded: Vec<u8> = bincode::serialize(&cov_report).unwrap();
        mem[0..8].copy_from_slice(&encoded.len().to_le_bytes());
        mem[8..8 + encoded.len()].copy_from_slice(&encoded);
    }
}

pub fn make_cov() -> Vec<CovReport> {
    let hit_mappings = make_hit_mapping();
    let mut cov_reports = vec![];

    for (filename, (src_map, hit_lines)) in hit_mappings {
        let src_lines: HashSet<_> = src_map.lines.iter().cloned().collect();
        let hit_lines: HashSet<_> = hit_lines.iter().cloned().collect();

        let (lines_untouched, lines_hits_ratio) = get_lines_cov(&src_lines, &hit_lines);
        let (funcs_untouched, funcs_hits_ratio) = get_func_cov(&src_map.funcs, &hit_lines);
        let (br_untouched, brs_hits_ratio, brs_untouched_with_tb) =
            get_br_cov(&src_map.brs, &hit_lines);

        if env::var(COVERAGE_DEBUG).is_ok() {
            println!("filename                      : {}", filename);
            println!("src_lines                     : {:?}", src_lines);
            println!("hit_lines                     : {:?}", hit_lines);
            println!("func hits ratio               : {:?}", funcs_hits_ratio);
            println!("untouched funcs               : {:?}", funcs_untouched);
            println!("line hit ratio                : {:?}", lines_hits_ratio);
            println!("untouched lines               : {:?}", lines_untouched);
            println!("br hits ratio                 : {:?}", brs_hits_ratio);
            println!("untouched brs                 : {:?}", br_untouched);
            println!(
                "untouched brs with true false : {:?}",
                brs_untouched_with_tb
            );
        }

        cov_reports.push(CovReport::new(
            filename,
            funcs_hits_ratio,
            &funcs_untouched,
            brs_hits_ratio,
            &brs_untouched_with_tb,
            lines_hits_ratio,
            &lines_untouched,
        ));
    }
    cov_reports
}

fn make_hit_mapping() -> HashMap<String, (SourceMapping, Vec<u32>)> {
    let state = COVERAGE_STATE.read().unwrap();
    let src_map = state.source_map.lock().unwrap();
    let loc_map = state.location_map.lock().unwrap();
    let locs: Vec<_> = loc_map.iter().collect();

    let mut all_hit_mapping = HashMap::new();
    for (filename, srcs) in src_map.iter() {
        let mut hit_lines = vec![];
        for (loc, _) in &locs {
            if loc.file == *filename {
                hit_lines.push(loc.line);
            }
        }
        all_hit_mapping.insert(filename.clone(), (srcs.clone(), hit_lines));
    }
    all_hit_mapping
}

fn get_func_cov(funcs: &[u32], hit_lines: &HashSet<u32>) -> (Vec<u32>, f64) {
    let func_locs = funcs.iter().cloned().collect::<HashSet<_>>();
    let func_hits = get_intersect(hit_lines, &func_locs);
    let mut funcs_untouched = get_symmetric_diff(&func_locs, &func_hits);
    let funcs_hits_ratio = func_hits.len() as f64 / funcs.len() as f64 * 100.0;
    funcs_untouched.sort();
    (funcs_untouched, funcs_hits_ratio)
}

fn get_br_cov(brs: &[u32], hit_lines: &HashSet<u32>) -> (Vec<u32>, f64, Vec<String>) {
    let br_locs = brs.iter().cloned().collect::<HashSet<_>>();
    let br_hits = get_intersect(hit_lines, &br_locs);
    let mut br_untouched = get_symmetric_diff(&br_locs, &br_hits);
    br_untouched.sort();

    let mut br_true_map = BTreeMap::new();
    let mut br_false_map = BTreeMap::new();
    if !brs.is_empty() {
        for i in (0..brs.len()).step_by(2) {
            br_true_map.insert(brs[i], brs[i + 1]);
            br_false_map.insert(brs[i + 1], brs[i]);
        }
    }
    let brs_untouched_with_tb = br_untouched
        .iter()
        .map(|line| {
            if let Some(fbr_line) = br_true_map.get(line) {
                return format!("{}({}:F)", line, fbr_line);
            }
            if let Some(tbr_line) = br_false_map.get(line) {
                return format!("{}({}:T)", line, tbr_line);
            }
            "".to_string()
        })
        .collect::<Vec<_>>();
    let brs_hits_ratio =
        br_hits.len() as f64 / brs.iter().collect::<HashSet<_>>().len() as f64 * 100.0;
    (br_untouched, brs_hits_ratio, brs_untouched_with_tb)
}

fn get_lines_cov(src_lines: &HashSet<u32>, hit_lines: &HashSet<u32>) -> (Vec<u32>, f64) {
    let lines_hits = get_intersect(hit_lines, src_lines);
    let lines_hits_ratio = lines_hits.len() as f64 / src_lines.len() as f64 * 100.0;
    let mut lines_untouched = src_lines
        .symmetric_difference(hit_lines)
        .cloned()
        .collect::<Vec<_>>();
    lines_untouched.sort();
    (lines_untouched, lines_hits_ratio)
}
