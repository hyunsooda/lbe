use anyhow::Result;
use clap::{Arg, Command};

#[derive(Clone, Copy)]
pub enum ModuleTyp {
    Coverage,
    Asan,
    Fuzz,
    Symbolic,
    Race,
    All,
}

impl ModuleTyp {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Coverage => "coverage",
            Self::Asan => "asan",
            Self::Fuzz => "fuzz",
            Self::Race => "race",
            Self::Symbolic => "symbolic",
            Self::All => "all",
        }
    }

    fn from_str(module_typ: &str) -> Option<Self> {
        [
            Self::Coverage,
            Self::Asan,
            Self::Fuzz,
            Self::Race,
            Self::Symbolic,
            Self::All,
        ]
        .iter()
        .copied()
        .find(|m| m.as_str() == module_typ)
    }
}

pub fn get_args() -> Result<(String, String, ModuleTyp)> {
    let matches = Command::new("coverage")
        .arg(
            Arg::new("input_file_path")
                .short('i')
                .long("input")
                .value_name("string")
                .help("Input ll file")
                .required(true), // Make it mandatory
        )
        .arg(
            Arg::new("output_file_path")
                .short('o')
                .long("output")
                .value_name("string")
                .help("Instrumented ll file")
                .required(true),
        )
        .arg(
            Arg::new("module_type")
                .short('m')
                .long("module")
                .value_name("string")
                .help("coverage || asan || fuzz || all")
                .required(true),
        )
        .get_matches();

    // Get values
    let input_filename: String = matches
        .get_one::<String>("input_file_path")
        .unwrap()
        .parse()
        .expect("Invalid path");
    let output_filename: String = matches
        .get_one::<String>("output_file_path")
        .unwrap()
        .parse()
        .expect("Invalid path");

    let module_typ_str: String = matches
        .get_one::<String>("module_type")
        .unwrap()
        .parse()
        .expect("Invalid module");

    let module_typ = ModuleTyp::from_str(&module_typ_str);
    if module_typ.is_none() {
        return Err(anyhow::anyhow!(
            "Invalid module type. Only <coaverage || asan || fuzz || all> available "
        ));
    }
    Ok((input_filename, output_filename, module_typ.unwrap()))
}
