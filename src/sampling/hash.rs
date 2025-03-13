use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Read};

/// A streaming iterator that performs hash-based sampling on CSV data
pub struct CsvHashSampler<R: Read> {
    reader: csv::Reader<R>,
    probability: f64,
    column_index: usize,
    header: csv::StringRecord,
    current_record: Option<csv::StringRecord>,
    done: bool,
}

// Implement Debug manually since csv::Reader doesn't implement Debug
impl<R: Read> fmt::Debug for CsvHashSampler<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CsvHashSampler")
            .field("probability", &self.probability)
            .field("column_index", &self.column_index)
            .field("header", &self.header)
            .field("done", &self.done)
            .finish_non_exhaustive() // Indicates there are fields not shown (reader)
    }
}

impl<R: Read> CsvHashSampler<R> {
    pub fn new(reader: R, percentage: f64, column_name: &str) -> io::Result<Self> {
        assert!(
            (0.0..=100.0).contains(&percentage),
            "Percentage must be between 0 and 100"
        );

        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true) // Be flexible with the number of fields
            .trim(csv::Trim::All) // Trim whitespace from fields
            .from_reader(reader);

        // Read the header
        let header = match csv_reader.headers() {
            Ok(h) => h.clone(),
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        };

        // Find the column index
        let column_index = match header.iter().position(|h| h.trim() == column_name.trim()) {
            Some(idx) => idx,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Column '{}' not found in CSV header", column_name),
                ))
            }
        };

        Ok(CsvHashSampler {
            reader: csv_reader,
            probability: percentage / 100.0,
            column_index,
            header,
            current_record: None,
            done: false,
        })
    }

    /// Returns the header record
    pub fn header(&self) -> &csv::StringRecord {
        &self.header
    }

    /// Samples the CSV data and returns all records that pass the sampling criteria
    pub fn collect_all(self) -> io::Result<Vec<csv::StringRecord>> {
        self.collect::<io::Result<Vec<_>>>()
    }

    /// Reads the next record from the CSV reader
    fn read_next_record(&mut self) -> Option<io::Result<csv::StringRecord>> {
        if self.done {
            return None;
        }

        match self.reader.read_record(
            self.current_record
                .get_or_insert_with(csv::StringRecord::new),
        ) {
            Ok(has_record) => {
                if !has_record {
                    self.done = true;
                    return None;
                }
                Some(Ok(self.current_record.as_ref().unwrap().clone()))
            }
            Err(e) => {
                self.done = true;
                Some(Err(io::Error::new(io::ErrorKind::InvalidData, e)))
            }
        }
    }
}

/// Implement Iterator for CsvHashSampler to enable streaming access to sampled records
impl<R: Read> Iterator for CsvHashSampler<R> {
    type Item = io::Result<csv::StringRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        // Keep reading records until we find one that should be included or reach the end
        loop {
            // Get the next record from the CSV reader
            let record_result = self.read_next_record()?;

            // Handle any errors reading the record
            let record = match record_result {
                Ok(r) => r,
                Err(e) => return Some(Err(e)),
            };

            // Get the column value
            let column_value = match record.get(self.column_index) {
                Some(value) => value.to_string(),
                None => {
                    // This shouldn't happen due to the validation in new(), but just in case
                    return Some(Ok(record));
                }
            };

            // Calculate hash and make decision directly
            let hash_value = calculate_hash(&column_value);
            let include = (hash_value as f64 / u64::MAX as f64) < self.probability;

            if include {
                return Some(Ok(record));
            }
            // If not included, continue to the next record
        }
    }
}

/// Calculate a hash value for a string
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_csv_hash_sampler() {
        let csv_data = "\
id,name,value
1,Alice,100
2,Bob,200
1,Alice,300
3,Charlie,400
2,Bob,500
4,Dave,600";

        let cursor = Cursor::new(csv_data);
        let percentage = 50.0; // 50%
        let column_name = "id";

        let sampler = CsvHashSampler::new(cursor, percentage, column_name).unwrap();
        let samples = sampler.collect_all().unwrap();

        // Check that rows with the same id are either all included or all excluded
        let has_id_1 = samples.iter().any(|row| row.get(0) == Some("1"));
        let has_id_2 = samples.iter().any(|row| row.get(0) == Some("2"));

        if has_id_1 {
            // Both rows with id=1 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("1")).count(),
                2
            );
        } else {
            // No rows with id=1 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("1")).count(),
                0
            );
        }

        if has_id_2 {
            // Both rows with id=2 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("2")).count(),
                2
            );
        } else {
            // No rows with id=2 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("2")).count(),
                0
            );
        }
    }

    #[test]
    fn test_csv_hash_sampler_iterator() {
        let csv_data = "\
id,name,value
1,Alice,100
2,Bob,200
1,Alice,300
3,Charlie,400
2,Bob,500
4,Dave,600";

        let cursor = Cursor::new(csv_data);
        let percentage = 50.0; // 50%
        let column_name = "id";

        let sampler = CsvHashSampler::new(cursor, percentage, column_name).unwrap();
        let samples: Vec<csv::StringRecord> = sampler.collect::<Result<Vec<_>, _>>().unwrap();

        // Check that rows with the same id are either all included or all excluded
        let has_id_1 = samples.iter().any(|row| row.get(0) == Some("1"));
        let has_id_2 = samples.iter().any(|row| row.get(0) == Some("2"));

        if has_id_1 {
            // Both rows with id=1 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("1")).count(),
                2
            );
        } else {
            // No rows with id=1 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("1")).count(),
                0
            );
        }

        if has_id_2 {
            // Both rows with id=2 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("2")).count(),
                2
            );
        } else {
            // No rows with id=2 should be included
            assert_eq!(
                samples.iter().filter(|row| row.get(0) == Some("2")).count(),
                0
            );
        }
    }

    #[test]
    fn test_csv_hash_sampler_column_not_found() {
        let csv_data = "id,name,value\n1,Alice,100";
        let cursor = Cursor::new(csv_data);
        let percentage = 50.0;
        let column_name = "non_existent_column";

        let result = CsvHashSampler::new(cursor, percentage, column_name);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_hash_consistency() {
        // Test that the same value always hashes to the same decision
        let value = "test_value";
        let hash1 = calculate_hash(&value);
        let hash2 = calculate_hash(&value);

        assert_eq!(hash1, hash2);
    }
}
