# halfband-rs

Kaiser half-band decimation filter and a multi-rate sample cascade with
absolute indexing, for octave-by-octave analysis, in safe Rust.

[API documentation](https://docs.rs/halfband-rs) · [Changelog](./CHANGELOG.md)

`HalfBand` is a symmetric, odd-length half-band FIR filter designed with a
Kaiser window for a given transition width and stop-band attenuation. Only
the centre tap and the odd taps are non-zero, so halving the sample rate
costs about a quarter of the tap count per output sample. `Levels` keeps a
signal and its successively halved copies, each with an absolute index
origin, so a frame centred on full-rate sample `t` reads level `d` around
`t >> d`. Decimation is zero-phase and feeding samples incrementally gives
the same output as feeding them at once.

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

## Features

- `parallel` (default): spreads long decimations and copies over a rayon
  thread pool. Disable default features for a single-threaded build.

Used by [cqt-rs](https://github.com/F0rty-Tw0/cqt-rs) to analyse each
octave at its own sample rate.

## License

MIT
