use std::env;
use std::process;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Config {
    pub sample_size: Option<usize>,
    pub percentage: Option<f64>,
    pub preserve_header: bool,
    pub seed: Option<u64>,
}

pub fn print_usage() {
    eprintln!(
        "Usage: sample (<sample_size> | --percent <percentage>) [--header] [--seed <number>]"
    );
    eprintln!("Reads lines from stdin and outputs a random sample.");
    eprintln!("Options:");
    eprintln!("  <sample_size>      Number of lines to sample");
    eprintln!("  --percent <value>  Percentage of lines to sample (0-100)");
    eprintln!("  --header           Preserve the first line as header (don't count in sampling)");
    eprintln!("  --seed            Set a fixed random seed for reproducible output");
    eprintln!("Examples:");
    eprintln!("  cat data.txt | sample 10           # Sample 10 lines");
    eprintln!("  cat data.txt | sample --percent 5  # Sample 5% of lines");
    eprintln!("  cat data.csv | sample 10 --header");
    eprintln!("  cat data.txt | sample 10 --seed 42");
}

pub fn parse_args() -> Result<Config> {
    let args: Vec<String> = env::args().collect();
    parse_args_from_vec(&args)
}

fn parse_args_from_vec(args: &[String]) -> Result<Config> {
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut sample_size = None;
    let mut percentage = None;
    let mut preserve_header = false;
    let mut seed = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                preserve_header = true;
                i += 1;
            }
            "--seed" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --seed requires a number");
                    print_usage();
                    process::exit(1);
                }
                seed = Some(args[i + 1].parse().map_err(|_| Error::InvalidSeedValue)?);
                i += 2;
            }
            "--percent" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --percent requires a value");
                    print_usage();
                    process::exit(1);
                }
                let value = args[i + 1].parse().map_err(|_| Error::InvalidPercentage)?;
                if !(0.0..=100.0).contains(&value) {
                    return Err(Error::InvalidPercentage);
                }
                if sample_size.is_some() {
                    return Err(Error::InvalidPercentage);
                }
                percentage = Some(value);
                i += 2;
            }
            arg if i == 1 => {
                // First argument should be sample size if not a flag
                sample_size = Some(arg.parse().map_err(|_| Error::InvalidSampleSize)?);
                if sample_size == Some(0) {
                    return Err(Error::InvalidSampleSize);
                }
                if percentage.is_some() {
                    return Err(Error::InvalidPercentage);
                }
                i += 1;
            }
            _ => {
                eprintln!("Error: unknown option '{}'", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    // Ensure either sample_size or percentage is provided
    if sample_size.is_none() && percentage.is_none() {
        eprintln!("Error: either sample size or percentage must be specified");
        print_usage();
        process::exit(1);
    }

    Ok(Config {
        sample_size,
        percentage,
        preserve_header,
        seed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_basic() {
        let args = vec!["sample".to_string(), "10".to_string()];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(!config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_percentage() {
        let args = vec![
            "sample".to_string(),
            "--percent".to_string(),
            "5.5".to_string(),
        ];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(5.5));
        assert!(!config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_header() {
        let args = vec![
            "sample".to_string(),
            "10".to_string(),
            "--header".to_string(),
        ];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_seed() {
        let args = vec![
            "sample".to_string(),
            "10".to_string(),
            "--seed".to_string(),
            "42".to_string(),
        ];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(!config.preserve_header);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_parse_args_with_header_and_seed() {
        let args = vec![
            "sample".to_string(),
            "10".to_string(),
            "--header".to_string(),
            "--seed".to_string(),
            "42".to_string(),
        ];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.sample_size, Some(10));
        assert_eq!(config.percentage, None);
        assert!(config.preserve_header);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_parse_args_with_percentage_and_header() {
        let args = vec![
            "sample".to_string(),
            "--percent".to_string(),
            "10".to_string(),
            "--header".to_string(),
        ];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.sample_size, None);
        assert_eq!(config.percentage, Some(10.0));
        assert!(config.preserve_header);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_parse_args_with_invalid_percentage() {
        let args = vec![
            "sample".to_string(),
            "--percent".to_string(),
            "101".to_string(),
        ];
        let result = parse_args_from_vec(&args);
        assert!(matches!(result, Err(Error::InvalidPercentage)));
    }

    #[test]
    fn test_parse_args_with_both_size_and_percentage() {
        let args = vec![
            "sample".to_string(),
            "10".to_string(),
            "--percent".to_string(),
            "5".to_string(),
        ];
        let result = parse_args_from_vec(&args);
        assert!(matches!(result, Err(Error::InvalidPercentage)));
    }
}
