use std::env;
use std::process;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Config {
    pub sample_size: usize,
    pub preserve_header: bool,
    pub seed: Option<u64>,
}

pub fn print_usage() {
    eprintln!("Usage: sample <sample_size> [--header] [--seed <number>]");
    eprintln!("Reads lines from stdin and outputs a random sample of the specified size.");
    eprintln!("Options:");
    eprintln!("  --header  Preserve the first line as header (don't count in sampling)");
    eprintln!("  --seed    Set a fixed random seed for reproducible output");
    eprintln!("Example: cat data.txt | sample 10");
    eprintln!("         cat data.csv | sample 10 --header");
    eprintln!("         cat data.txt | sample 10 --seed 42");
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

    let sample_size = args[1]
        .parse::<usize>()
        .map_err(|_| Error::InvalidSampleSize)?;
    if sample_size == 0 {
        return Err(Error::InvalidSampleSize);
    }

    let mut preserve_header = false;
    let mut seed = None;
    let mut i = 2;

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
            _ => {
                eprintln!("Error: unknown option '{}'", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    Ok(Config {
        sample_size,
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
        assert_eq!(config.sample_size, 10);
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
        assert_eq!(config.sample_size, 10);
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
        assert_eq!(config.sample_size, 10);
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
        assert_eq!(config.sample_size, 10);
        assert!(config.preserve_header);
        assert_eq!(config.seed, Some(42));
    }
}
