use rand::Rng;
use std::env;
use std::io::{self, BufRead};
use std::process;

fn print_usage() {
    eprintln!("Usage: sample <sample_size>");
    eprintln!("Reads lines from stdin and outputs a random sample of the specified size.");
    eprintln!("Example: cat data.txt | sample 10");
}

/// Performs reservoir sampling on an iterator of items
/// Returns a random sample of size k
pub fn reservoir_sample<T, I, R>(iter: I, k: usize, rng: &mut R) -> Vec<T>
where
    I: Iterator<Item = T>,
    R: Rng,
{
    let mut reservoir: Vec<T> = Vec::with_capacity(k);
    let mut count: usize = 0;

    for item in iter {
        count += 1;

        if count <= k {
            // Fill the reservoir with the first k items
            reservoir.push(item);
        } else {
            // Replace elements with decreasing probability
            let j = rng.gen_range(0..count);
            if j < k {
                reservoir[j] = item;
            }
        }
    }

    reservoir
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        print_usage();
        process::exit(1);
    }

    // Parse the sample size
    let k = match args[1].parse::<usize>() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("Error: sample size must be a positive integer");
            print_usage();
            process::exit(1);
        }
    };

    // Set up RNG and stdin
    let mut rng = rand::thread_rng();
    let stdin = io::stdin();

    // Read lines from stdin and perform reservoir sampling
    let lines_iter = stdin.lock().lines().map_while(Result::ok);
    let sampled_lines = reservoir_sample(lines_iter, k, &mut rng);

    // Output the sampled items
    for line in sampled_lines {
        println!("{}", line);
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
}
