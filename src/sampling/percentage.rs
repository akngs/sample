use rand::Rng;

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
