//! Streams a 48 kHz signal into `Levels` in irregular chunks, as an audio
//! callback would, and reads a short frame from every octave level centred
//! on the same full-rate instant.
//!
//! The signal is three tones. Each level keeps only the tones below its
//! pass-band edge, so the centre of every frame should equal the sum of the
//! surviving tones at that instant; the table prints the error. Memory stays
//! bounded because every level is trimmed with `discard_before` to what the
//! next level and the pending reads still need; the last column is the most
//! any level ever held.
//!
//! Run with `cargo run --release --example octave_frames`.

use halfband_rs::{HalfBand, Levels};
use std::f64::consts::TAU;

const SR: f64 = 48_000.0;
const LEVELS: usize = 8;
const HALF: i64 = 4; // frames are 2 * HALF + 1 samples long
/// `(frequency Hz, amplitude)`. 525 Hz and 4.2 kHz sit at 0.35 of the rate
/// of the last level that keeps them (1.5 and 12 kHz), so no level sees a
/// tone inside its transition band.
const TONES: [(f64, f64); 3] = [(30.0, 0.5), (525.0, 0.3), (4_200.0, 0.2)];

fn tone(i: i64, (f, a): (f64, f64)) -> f64 {
    a * (TAU * f * i as f64 / SR).sin()
}

fn main() {
    let filter = HalfBand::design(0.2, 80.0);
    let delay = filter.delay() as i64;
    let mut levels = Levels::new(Some(filter), LEVELS);
    let instants = [50_021i64, 123_457, 170_003];
    let mut errors = [[0.0f64; 3]; LEVELS];
    let (mut next, mut fed, mut seed) = (0, 0i64, 1u32);
    let mut peak = [0usize; LEVELS];

    while fed < 4 * SR as i64 {
        // Irregular chunk sizes between 64 and 1087 from a fixed LCG.
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let chunk: Vec<f32> = (fed..fed + 64 + i64::from(seed >> 22))
            .map(|i| TONES.iter().map(|&t| tone(i, t)).sum::<f64>() as f32)
            .collect();
        fed += chunk.len() as i64;
        levels.extend(&chunk);
        levels.propagate();

        // Read each instant once every level has produced its frame. Level
        // `d` sample `t >> d` sits at full-rate index `(t >> d) << d`.
        while next < instants.len()
            && (0..LEVELS).all(|d| levels.end(d) > (instants[next] >> d) + HALF)
        {
            let t = instants[next];
            let mut frame = [0.0f32; 2 * HALF as usize + 1];
            for (d, row) in errors.iter_mut().enumerate() {
                levels.frame(d, t >> d, &mut frame);
                let rate = SR / (1 << d) as f64;
                let expected: f64 = TONES
                    .iter()
                    .filter(|&&(f, _)| f < 0.4 * rate)
                    .map(|&tn| tone((t >> d) << d, tn))
                    .sum();
                row[next] = (f64::from(frame[HALF as usize]) - expected).abs();
            }
            next += 1;
        }

        // Keep what the next level still needs as filter history (output `n`
        // reads input `2n - delay` onward) and what the next read needs.
        for (d, peak) in peak.iter_mut().enumerate() {
            let mut keep = if d + 1 < LEVELS {
                2 * levels.end(d + 1) - delay
            } else {
                levels.end(d)
            };
            if let Some(&t) = instants.get(next) {
                keep = keep.min((t >> d) - HALF);
            }
            levels.discard_before(d, keep);
            *peak = (*peak).max(levels.retained(d));
        }
    }

    println!("streamed {fed} samples at {SR} Hz; frames centred on t = {instants:?}");
    println!("level   rate Hz  tones kept         |err| @ t1  |err| @ t2  |err| @ t3  peak kept");
    for (d, row) in errors.iter().enumerate() {
        let rate = SR / (1 << d) as f64;
        let kept: Vec<String> = TONES
            .iter()
            .filter(|&&(f, _)| f < 0.4 * rate)
            .map(|&(f, _)| format!("{f}"))
            .collect();
        println!(
            "{d:>5} {rate:>9.1}  {:<18} {:>10.1e}  {:>10.1e}  {:>10.1e}  {:>9}",
            kept.join(","),
            row[0],
            row[1],
            row[2],
            peak[d]
        );
    }
}
