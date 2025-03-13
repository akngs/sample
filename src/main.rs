use rand::rngs::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use std::io::{self, BufRead};
use std::process;

use sample::{config, error::Error, percentage_sample_iter, reservoir_sample, CsvHashSampler};

fn process_input(config: &config::Config) -> sample::Result<()> {
    // Handle hash-based sampling with CSV library
    if config.csv_mode && config.percentage.is_some() && config.hash_column.is_some() {
        return process_hash_based_sampling(config);
    }

    // For other sampling methods, use the existing code
    let mut rng = if let Some(seed) = config.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_rng(thread_rng()).unwrap()
    };

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Handle header if enabled
    if config.csv_mode {
        if let Some(header) = lines.next() {
            let header_str = header?;
            println!("{}", header_str);
        }
    }

    // Create an iterator over the remaining lines
    let lines_iter = lines.map_while(|line: std::io::Result<String>| line.ok());

    // Perform sampling based on the configuration
    match (config.sample_size, config.percentage) {
        (Some(k), None) => process_reservoir_sampling(lines_iter, k, &mut rng),
        (None, Some(percentage)) => process_percentage_sampling(lines_iter, percentage, rng),
        _ => unreachable!("Config validation ensures one of sample_size or percentage is set"),
    };

    Ok(())
}

fn process_hash_based_sampling(config: &config::Config) -> sample::Result<()> {
    let percentage = config.percentage.unwrap();
    let column_name = config.hash_column.as_ref().unwrap();

    // Create a CSV reader from stdin
    let stdin = io::stdin();
    let reader = stdin.lock();

    // Create the CSV hash sampler
    let sampler = match CsvHashSampler::new(reader, percentage, column_name) {
        Ok(s) => s,
        Err(e) => {
            if e.kind() == io::ErrorKind::InvalidInput {
                return Err(Error::ColumnNotFound(column_name.clone()));
            } else {
                return Err(Error::IoError(e));
            }
        }
    };

    // Print the header
    println!("{}", sampler.header().iter().collect::<Vec<_>>().join(","));

    // Sample the data and print the results using the streaming iterator
    for record_result in sampler {
        match record_result {
            Ok(record) => {
                println!("{}", record.iter().collect::<Vec<_>>().join(","));
            }
            Err(e) => return Err(Error::IoError(e)),
        }
    }

    Ok(())
}

fn process_reservoir_sampling<I>(lines_iter: I, k: usize, rng: &mut impl Rng)
where
    I: Iterator<Item = String>,
{
    // For reservoir sampling, we need to collect all lines
    let lines: Vec<String> = lines_iter.collect();
    let sampled_lines = reservoir_sample(lines.iter(), k, rng);
    for line in sampled_lines {
        println!("{}", line);
    }
}

fn process_percentage_sampling<I, R>(lines_iter: I, percentage: f64, rng: R)
where
    I: Iterator<Item = String>,
    R: Rng,
{
    // Regular percentage sampling
    let sampled_iter = percentage_sample_iter(lines_iter, percentage, rng);
    for line in sampled_iter {
        println!("{}", line);
    }
}

fn main() {
    let config = match config::parse_args() {
        Ok(config) => config,
        Err(err) => {
            let error_message = match err {
                Error::InvalidSampleSize => "sample size must be a positive integer",
                Error::InvalidSeedValue => "seed must be a valid number",
                Error::InvalidPercentage => "percentage must be between 0 and 100",
                Error::HashRequiresCsvMode => "hash-based sampling requires --csv mode",
                Error::HashRequiresPercentage => {
                    "hash-based sampling only works with --percentage option"
                }
                Error::MissingRequiredOption(msg) => return eprintln!("Error: {}", msg),
                Error::ColumnNotFound(column) => {
                    return eprintln!("Error: column '{}' not found in CSV header", column)
                }
                Error::IoError(e) => return eprintln!("Error reading input: {}", e),
            };

            eprintln!("Error: {}", error_message);
            process::exit(1);
        }
    };

    if let Err(e) = process_input(&config) {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}
