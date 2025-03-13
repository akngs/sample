use rand::rngs::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use std::io::{self, BufRead, Read, Write};
use std::process;

use sample::{config, error::Error, percentage_sample_iter, reservoir_sample, CsvHashSampler};

/// Run the application with the given arguments, input, and output streams.
pub fn run_app<I, O>(args: &[&str], input: I, mut output: O) -> sample::Result<()>
where
    I: Read,
    O: Write,
{
    // Parse command line arguments
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let config = config::parse_args(args_owned.iter().cloned())?;

    // Handle hash-based sampling with CSV library
    if config.csv_mode && config.percentage.is_some() && config.hash_column.is_some() {
        return process_hash_based_sampling(config, input, output);
    }

    // For other sampling methods, use the existing code
    let mut rng = if let Some(seed) = config.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_rng(thread_rng()).unwrap()
    };

    let reader = io::BufReader::new(input);
    let mut lines = reader.lines();

    // Handle header if enabled
    if config.csv_mode {
        if let Some(header) = lines.next() {
            let header_str = header?;
            writeln!(output, "{}", header_str)?;
        }
    }

    // Create an iterator over the remaining lines
    let lines_iter = lines.map_while(|line: std::io::Result<String>| line.ok());

    // Perform sampling based on the configuration
    match (config.sample_size, config.percentage) {
        (Some(k), None) => process_reservoir_sampling(lines_iter, k, &mut rng, output)?,
        (None, Some(percentage)) => {
            process_percentage_sampling(lines_iter, percentage, rng, output)?
        }
        _ => unreachable!("Config validation ensures one of sample_size or percentage is set"),
    };

    Ok(())
}

fn process_hash_based_sampling<I, O>(
    config: config::Config,
    input: I,
    mut output: O,
) -> sample::Result<()>
where
    I: Read,
    O: Write,
{
    let percentage = config.percentage.unwrap();
    let column_name = config.hash_column.as_ref().unwrap();

    // Create the CSV hash sampler
    let sampler = match CsvHashSampler::new(input, percentage, column_name) {
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
    writeln!(
        output,
        "{}",
        sampler.header().iter().collect::<Vec<_>>().join(",")
    )?;

    // Sample the data and print the results using the streaming iterator
    for record_result in sampler {
        match record_result {
            Ok(record) => {
                writeln!(output, "{}", record.iter().collect::<Vec<_>>().join(","))?;
            }
            Err(e) => return Err(Error::IoError(e)),
        }
    }

    Ok(())
}

fn process_reservoir_sampling<I, O, R>(
    lines_iter: I,
    k: usize,
    rng: &mut R,
    mut output: O,
) -> sample::Result<()>
where
    I: Iterator<Item = String>,
    O: Write,
    R: Rng,
{
    let lines: Vec<String> = lines_iter.collect();
    let sampled_lines = reservoir_sample(lines.iter(), k, rng);
    for line in sampled_lines {
        writeln!(output, "{}", line)?;
    }
    Ok(())
}

fn process_percentage_sampling<I, O, R>(
    lines_iter: I,
    percentage: f64,
    rng: R,
    mut output: O,
) -> sample::Result<()>
where
    I: Iterator<Item = String>,
    O: Write,
    R: Rng,
{
    let sampled_iter = percentage_sample_iter(lines_iter, percentage, rng);
    for line in sampled_iter {
        writeln!(output, "{}", line)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = run_app(&args_str, io::stdin(), io::stdout());

    if let Err(err) = result {
        let error_message = match err {
            Error::InvalidSampleSize => "sample size must be a positive integer",
            Error::InvalidSeedValue => "seed must be a valid number",
            Error::InvalidPercentage => "percentage must be between 0 and 100",
            Error::HashRequiresCsvMode => "hash-based sampling requires --csv mode",
            Error::HashRequiresPercentage => {
                "hash-based sampling only works with --percentage option"
            }
            Error::MissingRequiredOption(msg) => {
                eprintln!("Error: {}", msg);
                process::exit(1);
            }
            Error::ColumnNotFound(column) => {
                eprintln!("Error: column '{}' not found in CSV header", column);
                process::exit(1);
            }
            Error::IoError(e) => {
                eprintln!("Error reading input: {}", e);
                process::exit(1);
            }
        };

        eprintln!("Error: {}", error_message);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_reservoir_sampling() {
        let result = run("2 --seed 42", "0\n1\n2\n3\n4\n");
        assert_eq!(result.lines().count(), 2);
    }

    #[test]
    fn test_percentage_sampling() {
        let result = run(
            "--percentage 50 --seed 42",
            "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n",
        );
        assert_eq!(result.lines().count(), 5);
    }

    #[test]
    fn test_csv_mode() {
        let result = run("1 --csv --seed 42", "a,b\n0,0\n1,1\n");
        assert_eq!(result, "a,b\n0,0\n");
    }

    fn run(cmd: &str, input: &str) -> String {
        // Split the command string into arguments
        let args: Vec<&str> = std::iter::once("sample")
            .chain(cmd.split_whitespace())
            .collect();

        // Create input and output buffers
        let input_cursor = Cursor::new(input);
        let mut output = Vec::new();

        // Run the application
        let result = run_app(&args, input_cursor, &mut output);

        // Check for errors
        assert!(result.is_ok(), "Command failed: {:?}", result.err());

        // Convert output to string
        String::from_utf8(output).unwrap()
    }
}
