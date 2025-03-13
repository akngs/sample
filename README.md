# Sampling utility

A simple command-line tool that performs random sampling on input data, supporting both fixed-size sampling (using reservoir sampling) and percentage-based sampling.

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
A command-line tool for random sampling of input data

Usage: sample [OPTIONS] [SAMPLE_SIZE]

Arguments:
  [SAMPLE_SIZE]  Number of lines to sample using reservoir sampling algorithm

Options:
  -p, --percentage <VALUE>  Percentage of lines to sample (0-100)
  -H, --header             Preserve the first line as header (don't count in sampling)
  -s, --seed <NUMBER>      Set a fixed random seed for reproducible output
  -h, --help              Print help
  -V, --version           Print version

The program reads lines from standard input and outputs a random sample. You can either:
1. Specify a fixed number of lines to sample (using reservoir sampling), or
2. Specify a percentage of lines to sample (using random sampling)
```

### Examples

Sample 10 lines from a file (using reservoir sampling):

```bash
cat data.txt | sample 10
```

Sample 5% of lines from a file (using random sampling):

```bash
cat data.txt | sample -p 5
```

Sample from a CSV file, preserving the header:

```bash
cat data.csv | sample 10 -H
```

Get reproducible output by setting a fixed seed:

```bash
cat data.txt | sample 10 -s 42
```

## How It Works

### Fixed-size Sampling (Reservoir Sampling)

When sampling a fixed number of lines (k):

1. The first k elements are put into the "reservoir".
2. For each subsequent element (i >= k), randomly decide whether it should replace an element in the reservoir.
3. When the algorithm finishes, the reservoir contains a random sample of k elements from the stream.

This implementation ensures that each item in the stream has an equal probability of being selected in the final sample.

### Percentage-based Sampling

When sampling a percentage of lines:

1. Each line is independently selected with probability p = percentage/100.
2. This results in approximately (percentage)% of the lines being selected.
3. The actual number of lines in the output may vary due to the random nature of the sampling.

When using a fixed seed, the output will be deterministic for both sampling methods, making it useful for reproducible sampling.

## Releases

Pre-built binaries for major platforms are available on the [GitHub Releases page](https://github.com/akngs/sample/releases). These binaries are automatically built and published when a new version tag is pushed to the repository.

### Available Platforms

- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

### Installing from Pre-built Binaries

1. Download the appropriate binary for your platform from the [Releases page](https://github.com/akngs/sample/releases)
2. Extract the archive:
   - For `.tar.gz` files: `tar -xzf sample-<version>-<platform>.tar.gz`
   - For `.zip` files: Use your preferred unzip tool
3. Move the binary to a location in your PATH, for example:
   - Linux/macOS: `sudo mv sample-<version>-<platform>/sample /usr/local/bin/sample`
   - Windows: Add the directory containing the executable to your PATH

### Creating a New Release

To create a new release with pre-built binaries:

1. Update the version in `Cargo.toml`
2. Commit your changes: `git commit -am "Bump version to x.y.z"`
3. Create a new tag: `git tag -a vx.y.z -m "Release vx.y.z"`
4. Push the tag: `git push origin vx.y.z`

The GitHub Actions workflow will automatically:

- Create a new release on GitHub
- Build binaries for all supported platforms
- Upload the binaries to the release
