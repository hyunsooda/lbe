use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

#[derive(Debug)]
enum Flag {
    InputFilePath,
    OutDir,
    OutBinName,
    Compiler,
    OptLevel,
    CoverageRuntimeLibPath,
    CoverageRuntimeLibName,
    AsanRuntimeLibPath,
    AsanRuntimeLibName,
    FuzzerRuntimeLibPath,
    FuzzerRuntimeLibName,
    SymbolicRuntimeLibPath,
    SymbolicRuntimeLibName,
    RaceRuntimeLibPath,
    RaceRuntimeLibName,
}

impl Flag {
    fn as_str(&self) -> &'static str {
        match self {
            Self::InputFilePath => "input_file_path",
            Self::OutDir => "output directory",
            Self::OutBinName => "output_bin_name",
            Self::Compiler => "compiler",
            Self::OptLevel => "opt_level",
            Self::CoverageRuntimeLibPath => "coverage_runtime_lib_path",
            Self::CoverageRuntimeLibName => "coverage_runtime_lib_name",
            Self::AsanRuntimeLibPath => "asan_runtime_lib_path",
            Self::AsanRuntimeLibName => "asan_runtime_lib_name",
            Self::FuzzerRuntimeLibPath => "fuzzer_runtime_lib_path",
            Self::FuzzerRuntimeLibName => "fuzzer_runtime_lib_name",
            Self::SymbolicRuntimeLibPath => "symbolic_runtime_lib_path",
            Self::SymbolicRuntimeLibName => "symbolic_runtime_lib_name",
            Self::RaceRuntimeLibPath => "race_runtime_lib_path",
            Self::RaceRuntimeLibName => "race_runtime_lib_name",
        }
    }
}

pub struct CLI {
    matches: ArgMatches,
}

impl CLI {
    pub fn new() -> Self {
        let matches = Command::new("coverage-extension")
            .arg(
                Arg::new(Flag::InputFilePath.as_str())
                    .short('i')
                    .long("input")
                    .value_name("string")
                    .help("Input C/C++ file")
                    .required(true), // Make it mandatory
            )
            .arg(
                Arg::new(Flag::OutDir.as_str())
                    .short('o')
                    .long("outdir")
                    .value_name("string")
                    .help("Output directory")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::OutBinName.as_str())
                    .short('b')
                    .long("bin")
                    .value_name("string")
                    .help("Binary file name")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::Compiler.as_str())
                    .short('c')
                    .long("compiler")
                    .value_name("string")
                    .help("Sepcify compiler (clang | clang++)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::OptLevel.as_str())
                    .short('p')
                    .long("opt")
                    .value_name("string")
                    .help("Sepcify optimization level <O0 || O1 || O2 || O3>")
                    .default_value("O0"), // default is O0 (disable optimization)
            )
            .arg(
                Arg::new(Flag::CoverageRuntimeLibPath.as_str())
                    .short('q')
                    .long("coverage_runtime_path")
                    .value_name("string")
                    .help("Coverage Runtime library path (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::CoverageRuntimeLibName.as_str())
                    .short('w')
                    .long("coverage_runtime_name")
                    .value_name("string")
                    .help("Coverage Runtime library name (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::AsanRuntimeLibPath.as_str())
                    .short('a')
                    .long("asan_runtime_path")
                    .value_name("string")
                    .help("Asan Runtime library path (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::AsanRuntimeLibName.as_str())
                    .short('s')
                    .long("asan_runtime_name")
                    .value_name("string")
                    .help("Asan Runtime library name (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::FuzzerRuntimeLibPath.as_str())
                    .short('f')
                    .long("fuzzer_runtime_path")
                    .value_name("string")
                    .help("Fuzzer Runtime library path (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::FuzzerRuntimeLibName.as_str())
                    .short('m')
                    .long("fuzzer_runtime_name")
                    .value_name("string")
                    .help("Fuzzer Runtime library name (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::SymbolicRuntimeLibPath.as_str())
                    .short('v')
                    .long("symbolic_runtime_path")
                    .value_name("string")
                    .help("Symbolic Runtime library path (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::SymbolicRuntimeLibName.as_str())
                    .short('g')
                    .long("symbolic_runtime_name")
                    .value_name("string")
                    .help("Symbolic Runtime library name (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::RaceRuntimeLibPath.as_str())
                    .short('k')
                    .long("race_runtime_path")
                    .value_name("string")
                    .help("Race Runtime library path (C runtime)")
                    .required(true),
            )
            .arg(
                Arg::new(Flag::RaceRuntimeLibName.as_str())
                    .short('j')
                    .long("race_runtime_name")
                    .value_name("string")
                    .help("Race Runtime library name (C runtime)")
                    .required(true),
            )
            .get_matches();
        Self { matches }
    }

    fn get_arg(&self, flag: &str) -> String {
        self.matches
            .get_one::<String>(flag)
            .unwrap()
            .parse()
            .unwrap()
    }

    pub fn get_args(
        &self,
    ) -> Result<(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )> {
        let input_filename = self.get_arg(Flag::InputFilePath.as_str());
        let out_dir = self.get_arg(Flag::OutDir.as_str());
        let out_bin = self.get_arg(Flag::OutBinName.as_str());
        let compiler = self.get_arg(Flag::Compiler.as_str());
        let opt_level = self.get_arg(Flag::OptLevel.as_str());
        let coverage_runtime_lib_path_name = self.get_arg(Flag::CoverageRuntimeLibPath.as_str());
        let coverage_runtime_lib_name = self.get_arg(Flag::CoverageRuntimeLibName.as_str());
        let asan_runtime_lib_path_name = self.get_arg(Flag::AsanRuntimeLibPath.as_str());
        let asan_runtime_lib_name = self.get_arg(Flag::AsanRuntimeLibName.as_str());
        let fuzzer_runtime_lib_path_name = self.get_arg(Flag::FuzzerRuntimeLibPath.as_str());
        let fuzzer_runtime_lib_name = self.get_arg(Flag::FuzzerRuntimeLibName.as_str());
        let symbolic_runtime_lib_path_name = self.get_arg(Flag::SymbolicRuntimeLibPath.as_str());
        let symbolic_runtime_lib_name = self.get_arg(Flag::SymbolicRuntimeLibName.as_str());
        let race_runtime_lib_path_name = self.get_arg(Flag::RaceRuntimeLibPath.as_str());
        let race_runtime_lib_name = self.get_arg(Flag::RaceRuntimeLibName.as_str());

        match opt_level.as_str() {
            "O0" | "O1" | "O2" | "O3" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid optimization level. Only <O0 || O1 || O2 || O3> are allowed."
                ))
            }
        }
        match compiler.as_str() {
            "clang" | "clang++" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid compiler. Only 'clang | clang++' are allowd"
                ))
            }
        };
        Ok((
            input_filename,
            out_dir,
            out_bin,
            compiler,
            opt_level,
            coverage_runtime_lib_path_name,
            coverage_runtime_lib_name,
            asan_runtime_lib_path_name,
            asan_runtime_lib_name,
            fuzzer_runtime_lib_path_name,
            fuzzer_runtime_lib_name,
            symbolic_runtime_lib_path_name,
            symbolic_runtime_lib_name,
            race_runtime_lib_path_name,
            race_runtime_lib_name,
        ))
    }
}
