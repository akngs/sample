use clap::Parser;
use std::process;

use crate::error::{Error, Result};

#[derive(Debug, Parser)]
#[command(
    name = "sample",
    about = "A command-line tool for random sampling of input data",
    long_about = "Reads lines from standard input and outputs a random sample. Supports both fixed-size sampling (using reservoir sampling) and percentage-based sampling.",
    version,
    after_help = "EXAMPLES:
    # Sample 10 lines from a file (using reservoir sampling)
    cat data.txt | sample 10

    # Sample 5% of lines from a file
    cat data.txt | sample -p 5

    # Sample from a CSV file, preserving the header
    cat data.csv | sample 10 --csv

    # Get reproducible output using a fixed seed
    cat data.txt | sample 10 -s 42"
)]
pub struct Config {
    /// Number of lines to sample using reservoir sampling algorithm.
    /// Cannot be used together with --percentage.
    #[arg(conflicts_with = "percentage", value_name = "SAMPLE_SIZE")]
    pub sample_size: Option<usize>,

    /// Percentage of lines to sample (0-100).
    /// Each line has this percentage chance of being included.
    #[arg(short = 'p', long, value_name = "VALUE", value_parser = percentage_validator)]
    pub percentage: Option<f64>,

    /// Preserve the first line as header (don't count in sampling).
    /// Useful when working with CSV files or data with column headers.
    #[arg(short = 'C', long = "csv")]
    pub csv_mode: bool,

    /// Set a fixed random seed for reproducible output.
    /// Using the same seed will produce the same sample for identical input.
    #[arg(short = 's', long, value_name = "NUMBER")]
    pub seed: Option<u64>,

    /// Column name to use for hash-based sampling.
    /// When specified, rows with the same value in this column will be either all included or all excluded.
    /// Only works with --csv and --percentage options.
    #[arg(long = "hash", value_name = "COLUMN_NAME")]
    pub hash_column: Option<String>,
}

fn percentage_validator(s: &str) -> std::result::Result<f64, String> {
    let value = s.parse::<f64>().map_err(|_| "must be a number")?;
    if !(0.0..=100.0).contains(&value) {
        return Err("percentage must be between 0 and 100".to_string());
    }
    Ok(value)
}

impl Config {
    fn validate(&self) -> Result<()> {
        if let Some(size) = self.sample_size {
            if size == 0 {
                return Err(Error::InvalidSampleSize);
            }
        }

        if self.sample_size.is_none() && self.percentage.is_none() {
            eprintln!("Error: either sample size or percentage must be specified");
            process::exit(1);
        }

        // Validate hash-based sampling requirements
        if self.hash_column.is_some() {
            // Hash-based sampling requires CSV mode
            if !self.csv_mode {
                return Err(Error::HashRequiresCsvMode);
            }

            // Hash-based sampling only works with percentage
            if self.percentage.is_none() {
                return Err(Error::HashRequiresPercentage);
            }
        }

        Ok(())
    }
}

pub fn parse_args() -> Result<Config> {
    let config = Config::parse();
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_args_basic() {
        let config = Config::try_parse_from(["sample", "10"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(!config.csv_mode);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_percentage() {
        let config = Config::try_parse_from(["sample", "--percentage", "5.5"]).unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(5.5));
        assert!(!config.csv_mode);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_header() {
        let config = Config::try_parse_from(["sample", "10", "--csv"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(config.csv_mode);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_seed() {
        let config = Config::try_parse_from(["sample", "10", "--seed", "42"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(!config.csv_mode);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_parse_args_with_header_and_seed() {
        let config = Config::try_parse_from(["sample", "10", "--csv", "--seed", "42"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(config.csv_mode);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_parse_args_with_percentage_and_header() {
        let config = Config::try_parse_from(["sample", "--percentage", "10", "--csv"]).unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(10.0));
        assert!(config.csv_mode);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_invalid_percentage() {
        let result = Config::try_parse_from(["sample", "--percentage", "101"]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("percentage must be between 0 and 100"));
    }

    #[test]
    fn test_parse_args_with_both_size_and_percentage() {
        let result = Config::try_parse_from(["sample", "10", "--percentage", "5"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_with_hash_column() {
        let config =
            Config::try_parse_from(["sample", "--percentage", "10", "--csv", "--hash", "user_id"])
                .unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(10.0));
        assert!(config.csv_mode);
        assert_eq!(config.hash_column, Some("user_id".to_string()));
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_hash_column_and_seed() {
        let config = Config::try_parse_from([
            "sample",
            "--percentage",
            "10",
            "--csv",
            "--hash",
            "user_id",
            "--seed",
            "42",
        ])
        .unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(10.0));
        assert!(config.csv_mode);
        assert_eq!(config.hash_column, Some("user_id".to_string()));
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_hash_requires_csv_mode() {
        let result = Config::try_parse_from(["sample", "--percentage", "10", "--hash", "user_id"]);
        assert!(result.is_err() || Config::validate(&result.unwrap()).is_err());
    }

    #[test]
    fn test_hash_requires_percentage() {
        let result = Config::try_parse_from(["sample", "10", "--csv", "--hash", "user_id"]);
        assert!(result.is_err() || Config::validate(&result.unwrap()).is_err());
    }
}
