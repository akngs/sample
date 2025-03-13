use rand::Rng;
use std::iter::Iterator;

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

/// A streaming iterator that performs random sampling based on a percentage
pub struct PercentageSampleIter<I, R> {
    iter: I,
    rng: R,
    probability: f64,
}

impl<I, R> PercentageSampleIter<I, R> {
    pub fn new(iter: I, percentage: f64, rng: R) -> Self {
        assert!(
            (0.0..=100.0).contains(&percentage),
            "Percentage must be between 0 and 100"
        );
        PercentageSampleIter {
            iter,
            rng,
            probability: percentage / 100.0,
        }
    }
}

impl<T, I: Iterator<Item = T>, R: Rng> Iterator for PercentageSampleIter<I, R> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(item) => {
                    if self.rng.gen::<f64>() < self.probability {
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }
}

/// Creates a streaming percentage sampler that returns an iterator
/// Each item has percentage% chance of being included in the output
pub fn percentage_sample_iter<T, I, R>(
    iter: I,
    percentage: f64,
    rng: R,
) -> PercentageSampleIter<I, R>
where
    I: Iterator<Item = T>,
    R: Rng,
{
    PercentageSampleIter::new(iter, percentage, rng)
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

    #[test]
    fn test_percentage_sample_iter() {
        let items: Vec<i32> = (1..1001).collect(); // 1000 items
        let percentage = 10.0; // 10%
        let seed = [42; 32];
        let rng = StdRng::from_seed(seed);

        let sample: Vec<_> = percentage_sample_iter(items.iter(), percentage, rng).collect();

        // With 10%, we expect roughly 100 items, but allow for random variation
        assert!(sample.len() > 50 && sample.len() < 150);

        // Check that all sampled items are from the original set
        for item in &sample {
            assert!(items.contains(item));
        }
    }

    #[test]
    #[should_panic(expected = "Percentage must be between 0 and 100")]
    fn test_percentage_sample_iter_invalid_percentage() {
        let items = [1, 2, 3];
        let rng = rand::thread_rng();
        let _ = percentage_sample_iter(items.iter(), 101.0, rng);
    }
}
