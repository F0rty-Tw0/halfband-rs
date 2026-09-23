//! Read every octave of a signal at the same instant, with no delay
//! bookkeeping.
//!
//! [`HalfBand`] is a symmetric, odd-length Kaiser-windowed half-band FIR
//! filter that halves the sample rate. [`Levels`] is a cascade of
//! successively halved copies of a signal, each kept with absolute sample
//! indices so a reader can address level `d` around `t >> d` for a
//! full-rate index `t`. Decimation is zero-phase, so incremental and bulk
//! propagation produce bit-identical samples. The `parallel` feature
//! spreads long decimations over a rayon thread pool.
//!
//! ```
//! use halfband_rs::{HalfBand, Levels};
//!
//! let filter = HalfBand::design(0.3, 80.0);
//! let sr = 8_000.0f32;
//! let signal: Vec<f32> = (0..16_000)
//!     .map(|i| (std::f32::consts::TAU * 50.0 * i as f32 / sr).sin())
//!     .collect();
//! let mut levels = Levels::new(Some(filter), 3);
//! levels.extend(&signal);
//! levels.propagate();
//!
//! // Level 2 sample n corresponds to full-rate sample 4n.
//! let mut frame = [0.0f32; 1];
//! levels.frame(2, 1_000, &mut frame);
//! assert!((frame[0] - signal[4_000]).abs() < 2e-3);
//! ```
//!
//! Examples: [`octave_frames.rs`](https://github.com/F0rty-Tw0/halfband-rs/blob/master/examples/octave_frames.rs)
//! streams audio-callback-sized chunks and reads aligned frames from every
//! level; [`response.rs`](https://github.com/F0rty-Tw0/halfband-rs/blob/master/examples/response.rs)
//! measures a design's frequency response through the public API.

#![warn(missing_docs)]

mod filter;
mod levels;

pub use filter::HalfBand;
pub use levels::Levels;
