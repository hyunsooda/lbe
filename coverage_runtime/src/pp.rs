use serde::{Deserialize, Serialize};
use tabled::settings::{
    object::{Columns, Rows, Segment},
    style::BorderColor,
    Alignment, Color,
};
use tabled::{Table, Tabled};

const MAX_LINE_NUM: usize = 5;

#[derive(Debug, Tabled, Serialize, Deserialize)]
pub struct CovReport {
    #[tabled(rename = "File")]
    pub file: String,
    #[tabled(rename = "% Funcs")]
    pub funcs_hit_ratio: String,
    #[tabled(rename = "Uncovered Funcs")]
    pub uncovered_funcs: String,
    #[tabled(rename = "% Branch")]
    pub brs_hit_ratio: String,
    #[tabled(rename = "Uncovered Branches")]
    pub uncovered_brs: String,
    #[tabled(rename = "% Lines")]
    pub lines_hit_ratio: String,
    #[tabled(rename = "Uncovered lines")]
    pub uncovered_lines: String,
}

impl CovReport {
    pub fn new(
        filename: String,
        funcs_hits_ratio: f64,
        untouched_func_lines: &[u32],
        brs_hits_ratio: f64,
        untouched_brs_lines: &[String],
        lines_hits_ratio: f64,
        untouched_lines_lines: &[u32],
    ) -> Self {
        let untouched_func_lines_report = make_report_str(untouched_func_lines);
        let untouched_brs_lines_report = make_report_str(untouched_brs_lines);
        let untouched_lines_lines_report = make_report_str(untouched_lines_lines);
        Self {
            file: filename,
            funcs_hit_ratio: format!("{:.2}", funcs_hits_ratio),
            uncovered_funcs: untouched_func_lines_report,
            brs_hit_ratio: format!("{:.2}", brs_hits_ratio),
            uncovered_brs: untouched_brs_lines_report,
            lines_hit_ratio: format!("{:.2}", lines_hits_ratio),
            uncovered_lines: untouched_lines_lines_report,
        }
    }
}

pub enum TableFormatter {
    TableWithColor,
    TableWithoutColor,
}

impl TableFormatter {
    pub fn format(typ: Self, reports: &Vec<CovReport>) -> String {
        let mut tbl = Table::new(reports);
        match typ {
            Self::TableWithColor => {
                tbl.with(BorderColor::filled(Color::FG_BRIGHT_BLACK));
                tbl.modify(Segment::all(), Alignment::right());
                tbl.modify(Columns::single(2), Color::FG_RED);
                tbl.modify(Columns::single(4), Color::FG_RED);
                tbl.modify(Columns::single(6), Color::FG_RED);
                tbl.modify(Rows::single(0), Color::FG_WHITE);
                tbl.to_string()
            }
            Self::TableWithoutColor => tbl.to_string(),
        }
    }
}

fn make_report_str<T: ToString>(lines: &[T]) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        let slice = &lines[..lines.len().min(MAX_LINE_NUM)];
        let joined = slice
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        if lines.len() > MAX_LINE_NUM {
            format!("...{}", joined)
        } else {
            joined
        }
    }
}
