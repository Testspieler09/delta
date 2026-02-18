use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// The filepath to the config file
    #[arg(short = 'f', long, value_name = "FILE")]
    pub config_file: PathBuf,

    /// The filepath to the output folder
    #[arg(short = 'o', long, value_name = "FOLDER")]
    pub output_folder: PathBuf,

    /// Disable incremental saving (enabled by default)
    #[arg(short = 's', long = "no-incremental", default_value_t = false)]
    pub no_incremental: bool,
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_args_parsing_short_flags() {
        let argv = ["my_program", "-f", "config.toml", "-o", "output_folder"];
        let args = Args::try_parse_from(argv).expect("Failed to parse args");

        assert_eq!(args.config_file, PathBuf::from("config.toml"));
        assert_eq!(args.output_folder, PathBuf::from("output_folder"));
        assert!(
            !args.no_incremental,
            "Incremental should be enabled by default"
        );
    }

    #[test]
    fn test_args_parsing_long_flags() {
        let argv = [
            "my_program",
            "--config-file",
            "config.toml",
            "--output-folder",
            "output_folder",
        ];
        let args = Args::try_parse_from(argv).expect("Failed to parse args");

        assert_eq!(args.config_file, PathBuf::from("config.toml"));
        assert_eq!(args.output_folder, PathBuf::from("output_folder"));
        assert!(
            !args.no_incremental,
            "Incremental should be enabled by default"
        );
    }

    #[test]
    fn test_args_disable_incremental_short_flag() {
        let argv = [
            "my_program",
            "-f",
            "config.toml",
            "-o",
            "output_folder",
            "-s",
        ];
        let args = Args::try_parse_from(argv).expect("Failed to parse args");

        assert!(
            args.no_incremental,
            "Incremental should be disabled with -s"
        );
    }

    #[test]
    fn test_args_disable_incremental_long_flag() {
        let argv = [
            "my_program",
            "--config-file",
            "config.toml",
            "--output-folder",
            "output_folder",
            "--no-incremental",
        ];
        let args = Args::try_parse_from(argv).expect("Failed to parse args");

        assert!(
            args.no_incremental,
            "Incremental should be disabled with --no-incremental"
        );
    }

    #[test]
    fn test_args_missing_config_file() {
        let argv = ["my_program", "-o", "output_folder"];
        let result = Args::try_parse_from(argv);
        assert!(result.is_err(), "Missing config_file should return error");

        let err = result.err().unwrap();
        assert!(err.to_string().contains("--config-file"));
    }

    #[test]
    fn test_args_missing_output_folder() {
        let argv = ["my_program", "-f", "config.toml"];
        let result = Args::try_parse_from(argv);
        assert!(result.is_err(), "Missing output_folder should return error");

        let err = result.err().unwrap();
        assert!(err.to_string().contains("--output-folder"));
    }
}
