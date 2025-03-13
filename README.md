# Reservoir Sampling CLI

A simple command-line tool that performs reservoir sampling on input data.

## What is Reservoir Sampling?

Reservoir sampling is a family of randomized algorithms for randomly selecting k samples from a list of n items, where n is either a very large or unknown number. This implementation uses Algorithm R, which has O(n) time complexity.

## Installation

If you have Rust and Cargo installed:

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

The executable will be located at `target/release/sample`.

## Testing

Run the test suite using:

```bash
cargo test
```

For verbose test output that shows all test cases:

```bash
cargo test -- --nocapture
```

## Usage

```
sample <sample_size> [--header] [--seed <number>]
```

The program reads lines from standard input and outputs a random sample of the specified size.

Options:
- `--header`: Preserve the first line as header (don't count in sampling)
- `--seed <number>`: Set a fixed random seed for reproducible output

### Example

Sample 10 lines from a file:

```bash
cat data.txt | sample 10
```

Sample 5 lines from a command output:

```bash
ls -la | sample 5
```

Get reproducible output by setting a fixed seed:

```bash
cat data.txt | sample 10 --seed 42
```

## How It Works

1. The first k elements are put into the "reservoir".
2. For each subsequent element (i >= k), randomly decide whether it should replace an element in the reservoir.
3. When the algorithm finishes, the reservoir contains a random sample of k elements from the stream.

This implementation ensures that each item in the stream has an equal probability of being selected in the final sample.

When using a fixed seed, the output will be deterministic, making it useful for reproducible sampling.