# halfband-rs

**Read every octave of a signal at the same instant, with no delay
bookkeeping.** A signal and its successively halved copies stay aligned by
absolute sample index, and feeding audio in callback-sized chunks gives
bit-identical samples to feeding it all at once. Safe Rust.

[API documentation](https://docs.rs/halfband-rs) · [Changelog](./CHANGELOG.md)

## What you get

- **Aligned octaves.** A frame centred on full-rate sample `t` reads level
  `d` at `t >> d`. Decimation is zero-phase, so there is no per-level group
  delay to track and compensate.
- **Streaming equals batch.** Incremental `extend` + `propagate` produces the
  same samples as one bulk call, so offline tests hold for the real-time path.
- **A filter you design.** `HalfBand::design(transition, attenuation_db)`
  trades tap count for alias rejection instead of shipping fixed
  coefficients.
- **Fast.** Only the centre tap and odd taps are non-zero, and each output is
  computed at the lower rate: about 5x faster than a plain FIR of the same
  length (numbers below).

Without it, each stage of an ordinary FIR decimator delays its output by the
filter's group delay, and that lag compounds down the cascade. Every reader
then has to shift each level by its own accumulated delay before frames from
different octaves line up.

## Usage

```rust
use halfband_rs::{HalfBand, Levels};

let filter = HalfBand::design(0.3, 80.0);
let sr = 8_000.0f32;
let signal: Vec<f32> = (0..16_000)
    .map(|i| (std::f32::consts::TAU * 50.0 * i as f32 / sr).sin())
    .collect();
let mut levels = Levels::new(Some(filter), 3);
levels.extend(&signal);
levels.propagate();

// Level 2 sample n corresponds to full-rate sample 4n.
let mut frame = [0.0f32; 1];
levels.frame(2, 1_000, &mut frame);
assert!((frame[0] - signal[4_000]).abs() < 2e-3);
```

For streaming, call `extend` and `propagate` per chunk. Alignment is
automatic; trimming old samples with `discard_before` to bound memory is up
to you. [`examples/octave_frames.rs`](./examples/octave_frames.rs) shows both,
streaming four seconds of 48 kHz audio in irregular chunks into eight levels
and reading every level at the same instants:

```text
level   rate Hz  tones kept         |err| @ t1  |err| @ t2  |err| @ t3  peak kept
    0   48000.0  30,525,4200            2.9e-8      2.1e-8      2.5e-8       3926
    3    6000.0  30,525                 3.0e-6      4.7e-6      1.8e-6        471
    7     375.0  30                     3.8e-6      2.7e-6      4.0e-6          8
```

## Measured

Frequency response of `HalfBand::design(0.2, 80.0)` (55 taps), measured
through the public API by [`examples/response.rs`](./examples/response.rs):

![Frequency response of HalfBand::design(0.2, 80.0)](https://raw.githubusercontent.com/F0rty-Tw0/halfband-rs/master/docs/response.svg)

| | measured |
|---|---|
| pass-band ripple, f ≤ 0.4 × Nyquist | 9.6e-4 dB |
| stop-band attenuation, f ≥ 0.6 × Nyquist | 78.8 dB |

The Kaiser tap-count estimate lands about 1.2 dB short of the requested
attenuation, so ask for a little more than you need. Run
`cargo run --release --example response -- <transition> <attenuation_db>` to
measure your own design.

Throughput, criterion medians on an Intel i7-13700H
(`cargo bench`, `cargo bench --no-default-features`):

| benchmark | serial | `parallel` |
|---|---|---|
| `decimate_block`, 1 s at 48 kHz, 55 taps | 54 µs | 47 µs |
| plain 55-tap FIR, then drop every second output | 279 µs | 256 µs |
| `Levels`, 10 s at 48 kHz, 8 levels, one bulk call | 1.01 ms | 0.25 ms |
| `Levels`, same, streamed in 512-sample chunks | 1.15 ms | 1.18 ms |

The streamed cascade runs about 8,700x faster than real time on one thread.
`decimate_block` and streaming never reach the rayon pool, so their two
columns differ only by run-to-run noise.

## When to use something else

- **Arbitrary rate conversion** (44.1 kHz to 48 kHz):
  [rubato](https://crates.io/crates/rubato). halfband-rs only halves.
- **`no_std`, no allocation, integer math on a microcontroller**:
  [idsp](https://crates.io/crates/idsp)'s `hbf` module, which ships fixed
  half-band coefficients for rate changes up to 32. It does not keep the
  intermediate levels addressable.

## Features

- `parallel` (default): spreads long decimations and copies over a rayon
  thread pool. Disable default features for a single-threaded build.

Minimum supported Rust version: 1.85.

Used by [cqt-rs](https://github.com/F0rty-Tw0/cqt-rs) to analyse each
octave at its own sample rate.

## License

MIT
