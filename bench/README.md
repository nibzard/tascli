## Benchmark

uses [hyperfine](https://github.com/sharkdp/hyperfine) to conduct benchmark on tascli insertion, list and deletions. `hyperfine` need to be installed separately on the system

`basic.sh` benchmarks task, record insertion, listing and deletion.

`with_config.sh` benchmarks the same but with a configuration file.

`nlp_comparison.sh` benchmarks NLP commands vs traditional commands (new).

`startup.sh` benchmarks startup time impact of NLP integration (new).

As shown, `tascli` has no background process, but it is fast, how fast is it on your machine?

### Example Run (basic.sh)

```
$ ./basic.sh
Benchmark 1: Task Insertion
  Time (mean ± σ):       2.0 ms ±   0.2 ms    [User: 1.1 ms, System: 0.7 ms]
  Range (min … max):     1.8 ms …   2.8 ms    50 runs

Benchmark 1: Record Insertion
  Time (mean ± σ):       2.4 ms ±   0.2 ms    [User: 1.2 ms, System: 0.9 ms]
  Range (min … max):     2.2 ms …   3.2 ms    50 runs

Benchmark 1: List Tasks
  Time (mean ± σ):       2.7 ms ±   0.2 ms    [User: 1.4 ms, System: 1.0 ms]
  Range (min … max):     2.6 ms …   4.0 ms    50 runs

Benchmark 1: Task Deletion
  Time (mean ± σ):       3.1 ms ±   0.3 ms    [User: 1.8 ms, System: 1.9 ms]
  Range (min … max):     2.9 ms …   4.6 ms    50 runs

Benchmark 1: Record Deletion
  Time (mean ± σ):       3.1 ms ±   0.2 ms    [User: 1.8 ms, System: 1.9 ms]
  Range (min … max):     2.9 ms …   3.9 ms    50 runs
```

### NLP Performance Benchmarks

#### Running NLP Comparison

```bash
# Ensure tascli is built and installed
cargo build --release

# Run the NLP comparison benchmark
./bench/nlp_comparison.sh
```

#### Running Startup Benchmarks

```bash
./bench/startup.sh
```

### Performance Target

The NLP integration targets **<20% performance impact** compared to traditional commands when using cached responses.

Key metrics:
- **Traditional commands**: Baseline performance (~2-3ms per operation)
- **NLP with API call**: Higher latency (expected due to network overhead)
- **NLP with cache hit**: Target <20% slower than traditional
- **Startup time**: Should remain fast (<10ms additional overhead)

### Interpreting Results

1. **Cached NLP Performance**: Compare "Traditional" vs "NLP (cached)" results. Cached results should be close to traditional commands.

2. **API Call Overhead**: "NLP (with API)" will show network latency. This is expected for first-time queries.

3. **Startup Impact**: The `startup.sh` benchmark measures the additional overhead of NLP integration initialization.

### Requirements

- hyperfine: `cargo install hyperfine`
- Built tascli binary: `cargo build --release`
- OpenAI API key configured (for NLP benchmarks): `tascli nlp config set-key YOUR_KEY`
