use anyhow::Result;
use clap::{Arg, Command};
use std::path::Path;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FuzzInput {
    ProgramArgument,
    Stdin,
}

impl FuzzInput {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ProgramArgument => "file",
            Self::Stdin => "stdin",
        }
    }

    fn from_str(input_typ: &str) -> Option<Self> {
        [Self::ProgramArgument, Self::Stdin]
            .iter()
            .copied()
            .find(|t| t.as_str() == input_typ)
    }
}

pub fn get_args() -> Result<(String, String, FuzzInput)> {
    let matches = Command::new("fuzz")
        .arg(
            Arg::new("target_program_path")
                .short('p')
                .long("program")
                .value_name("string")
                .help("Specify input program path")
                .required(true), // Make it mandatory <- (yes)
        )
        .arg(
            Arg::new("seed_directory_path")
                .short('c')
                .long("seed")
                .value_name("string")
                .help("Specify seed directory path")
                .required(true), // Make it mandatory <- (yes)
        )
        .arg(
            Arg::new("input_type")
                .short('t')
                .long("input_type")
                .value_name("string")
                .help("<file | stdin>")
                .required(true), // Make it mandatory <- (yes)
        )
        .get_matches();

    // Get values
    let program_path: String = matches
        .get_one::<String>("target_program_path")
        .unwrap()
        .parse()
        .expect("Invalid path");
    let seed_dirname: String = matches
        .get_one::<String>("seed_directory_path")
        .unwrap()
        .parse()
        .expect("Invalid path");
    let input_typ_str: String = matches
        .get_one::<String>("input_type")
        .unwrap()
        .parse()
        .expect("invalid input type");

    if !check_if_exist(&program_path) {
        return Err(anyhow::anyhow!(format!(
            "program path ({program_path}) does not exist"
        )));
    }
    if !check_if_exist(&seed_dirname) {
        return Err(anyhow::anyhow!(format!(
            "seed directory path ({seed_dirname}) does not exist"
        )));
    }
    let input_typ = FuzzInput::from_str(&input_typ_str);
    if input_typ.is_none() {
        return Err(anyhow::anyhow!(
            "Invalid input type. Only <file | stdin> available "
        ));
    }
    Ok((
        format!("./{}", program_path),
        seed_dirname,
        input_typ.unwrap(),
    ))
}

fn check_if_exist(filepath: &str) -> bool {
    let path = Path::new(filepath);
    path.exists()
}
