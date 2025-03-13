use clap::Parser;
use std::process;

use crate::error::{Error, Result};

#[derive(Debug, Parser)]
#[command(
    name = "sample",
    about = "Reads lines from stdin and outputs a random sample.",
    long_about = None
)]
pub struct Config {
    /// Number of lines to sample
    #[arg(conflicts_with = "percentage", value_name = "SAMPLE_SIZE")]
    pub sample_size: Option<usize>,

    /// Percentage of lines to sample (0-100)
    #[arg(long, value_name = "VALUE", value_parser = percentage_validator)]
    pub percentage: Option<f64>,

    /// Preserve the first line as header (don't count in sampling)
    #[arg(long = "header")]
    pub preserve_header: bool,

    /// Set a fixed random seed for reproducible output
    #[arg(long, value_name = "NUMBER")]
    pub seed: Option<u64>,
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
        assert!(!config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_percentage() {
        let config = Config::try_parse_from(["sample", "--percentage", "5.5"]).unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(5.5));
        assert!(!config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_header() {
        let config = Config::try_parse_from(["sample", "10", "--header"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_seed() {
        let config = Config::try_parse_from(["sample", "10", "--seed", "42"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(!config.preserve_header);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_parse_args_with_header_and_seed() {
        let config = Config::try_parse_from(["sample", "10", "--header", "--seed", "42"]).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(config.preserve_header);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_parse_args_with_percentage_and_header() {
        let config = Config::try_parse_from(["sample", "--percentage", "10", "--header"]).unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(10.0));
        assert!(config.preserve_header);
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
}
