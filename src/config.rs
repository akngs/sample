use std::env;
use std::process;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Config {
    pub sample_size: usize,
    pub preserve_header: bool,
}

pub fn print_usage() {
    eprintln!("Usage: sample <sample_size> [--header]");
    eprintln!("Reads lines from stdin and outputs a random sample of the specified size.");
    eprintln!("Options:");
    eprintln!("  --header  Preserve the first line as header (don't count in sampling)");
    eprintln!("Example: cat data.txt | sample 10");
    eprintln!("         cat data.csv | sample 10 --header");
}

pub fn parse_args() -> Result<Config> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        print_usage();
        process::exit(1);
    }

    let preserve_header = args.len() == 3 && args[2] == "--header";

    let sample_size = args[1]
        .parse::<usize>()
        .map_err(|_| Error::InvalidSampleSize)?;
    if sample_size == 0 {
        return Err(Error::InvalidSampleSize);
    }

    Ok(Config {
        sample_size,
        preserve_header,
    })
}
