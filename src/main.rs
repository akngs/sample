use rand::rngs::StdRng;
use rand::{thread_rng, SeedableRng};
use std::io::{self, BufRead};
use std::process;

use sample::{config, error::Error, percentage_sample_iter, reservoir_sample};

fn process_input(config: &config::Config) -> sample::Result<()> {
    let mut rng = if let Some(seed) = config.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_rng(thread_rng()).unwrap()
    };

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Handle header if enabled
    if config.preserve_header {
        if let Some(header) = lines.next() {
            println!("{}", header?);
        }
    }

    // Create an iterator over the remaining lines
    let lines_iter = lines.map_while(|line: std::io::Result<String>| line.ok());

    // Perform sampling based on the configuration
    match (config.sample_size, config.percentage) {
        (Some(k), None) => {
            // For reservoir sampling, we need to collect all lines
            let lines: Vec<String> = lines_iter.collect();
            let sampled_lines = reservoir_sample(lines.iter(), k, &mut rng);
            for line in sampled_lines {
                println!("{}", line);
            }
        }
        (None, Some(percentage)) => {
            // For percentage sampling, we can stream directly
            let sampled_iter = percentage_sample_iter(lines_iter, percentage, rng);
            for line in sampled_iter {
                println!("{}", line);
            }
        }
        _ => unreachable!("Config validation ensures one of sample_size or percentage is set"),
    };

    Ok(())
}

fn main() {
    let config = match config::parse_args() {
        Ok(config) => config,
        Err(Error::InvalidSampleSize) => {
            eprintln!("Error: sample size must be a positive integer");
            process::exit(1);
        }
        Err(Error::InvalidSeedValue) => {
            eprintln!("Error: seed must be a valid number");
            process::exit(1);
        }
        Err(Error::InvalidPercentage) => {
            eprintln!("Error: percentage must be between 0 and 100");
            process::exit(1);
        }
        Err(Error::IoError(e)) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = process_input(&config) {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_reservoir_sample_fewer_items_than_k() {
        let items = vec![1, 2, 3];
        let k = 5;
        let mut rng = rand::thread_rng();

        let sample = reservoir_sample(items.into_iter(), k, &mut rng);

        assert_eq!(sample.len(), 3);
        // All items should be included when there are fewer than k
        assert!(sample.contains(&1));
        assert!(sample.contains(&2));
        assert!(sample.contains(&3));
    }

    #[test]
    fn test_reservoir_sample_exact_k_items() {
        let items = vec![1, 2, 3, 4, 5];
        let k = 5;
        let mut rng = rand::thread_rng();

        let sample = reservoir_sample(items.into_iter(), k, &mut rng);

        assert_eq!(sample.len(), 5);
        // All items should be included when there are exactly k
        assert!(sample.contains(&1));
        assert!(sample.contains(&2));
        assert!(sample.contains(&3));
        assert!(sample.contains(&4));
        assert!(sample.contains(&5));
    }

    #[test]
    fn test_reservoir_sample_more_items_than_k() {
        // Use a seeded RNG for deterministic testing
        let seed = [42; 32];
        let mut rng = StdRng::from_seed(seed);

        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let k = 3;

        let sample = reservoir_sample(items.clone().into_iter(), k, &mut rng);

        assert_eq!(sample.len(), k);
        // With a seeded RNG, we should get consistent results
        // Note: This test is brittle and depends on the RNG implementation
        // In a real-world scenario, we might test statistical properties instead
        for item in &sample {
            assert!(items.contains(item));
        }
    }

    #[test]
    fn test_reservoir_sample_empty_input() {
        let items: Vec<i32> = vec![];
        let k = 5;
        let mut rng = rand::thread_rng();

        let sample = reservoir_sample(items.into_iter(), k, &mut rng);

        assert_eq!(sample.len(), 0);
    }

    #[test]
    fn test_reservoir_sample_with_header() {
        let mut rng = rand::thread_rng();
        let lines = [
            "header".to_string(),
            "data1".to_string(),
            "data2".to_string(),
            "data3".to_string(),
        ];
        let k = 2;

        // Simulate sampling without header
        let sample = reservoir_sample(lines[1..].iter(), k, &mut rng);
        assert_eq!(sample.len(), k);
    }
}
