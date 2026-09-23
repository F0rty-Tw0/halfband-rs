//! Throughput of the half-band decimator against a plain FIR of the same
//! length, and of the full `Levels` cascade in bulk and in streamed chunks.
//!
//! Run with `cargo bench` (rayon) or `cargo bench --no-default-features`
//! (serial).

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use halfband_rs::{HalfBand, Levels};
use std::f64::consts::PI;
use std::hint::black_box;

const SR: usize = 48_000;

fn signal(len: usize) -> Vec<f32> {
    (0..len)
        .map(|i| {
            let t = i as f32 / SR as f32;
            0.5 * (std::f32::consts::TAU * 440.0 * t).sin()
                + 0.2 * (std::f32::consts::TAU * 9_000.0 * t).sin()
        })
        .collect()
}

/// Hamming-windowed sinc low-pass at half the input Nyquist frequency. Only
/// its length matters for timing.
fn windowed_sinc(taps: usize) -> Vec<f32> {
    let mid = (taps - 1) as f64 / 2.0;
    (0..taps)
        .map(|k| {
            let n = k as f64 - mid;
            let sinc = if n == 0.0 {
                0.5
            } else {
                (PI * n / 2.0).sin() / (PI * n)
            };
            let window = 0.54 - 0.46 * (2.0 * PI * k as f64 / (taps - 1) as f64).cos();
            (sinc * window) as f32
        })
        .collect()
}

/// The textbook approach: filter at the full rate with every tap, then keep
/// every second output. The tap-outer loop vectorizes like `decimate_block`
/// does, so the comparison measures the algorithm, not the auto-vectorizer.
fn naive_decimate(h: &[f32], input: &[f32], full: &mut [f32], out: &mut [f32]) {
    full.fill(0.0);
    for (k, &c) in h.iter().enumerate() {
        for (y, &x) in full.iter_mut().zip(&input[k..]) {
            *y += c * x;
        }
    }
    for (o, &y) in out.iter_mut().zip(full.iter().step_by(2)) {
        *o = y;
    }
}

fn decimate(c: &mut Criterion) {
    let filter = HalfBand::design(0.2, 80.0);
    let delay = filter.delay();
    // One second of input, plus the filter's reach on both sides.
    let input = signal(SR + 2 * delay);
    let mut out = vec![0.0f32; SR / 2];
    let mut odd = Vec::new();
    let h = windowed_sinc(filter.taps());
    let mut full = vec![0.0f32; SR];

    let mut group = c.benchmark_group("decimate 1 s at 48 kHz");
    group.throughput(Throughput::Elements(SR as u64));
    group.bench_function(
        format!("HalfBand::decimate_block ({} taps)", filter.taps()),
        |b| {
            b.iter(|| {
                filter.decimate_block(black_box(&input), delay, &mut out, &mut odd);
                black_box(&out);
            })
        },
    );
    group.bench_function(format!("naive FIR then drop odd ({} taps)", h.len()), |b| {
        b.iter(|| {
            naive_decimate(&h, black_box(&input), &mut full, &mut out);
            black_box(&out);
        })
    });
    group.finish();
}

fn cascade(c: &mut Criterion) {
    const LEN: usize = 10 * SR;
    const LEVELS: usize = 8;
    let input = signal(LEN);
    let mut levels = Levels::new(Some(HalfBand::design(0.2, 80.0)), LEVELS);

    let mut group = c.benchmark_group("Levels 10 s at 48 kHz, 8 levels");
    group.throughput(Throughput::Elements(LEN as u64));
    group.bench_function("extend + propagate at once", |b| {
        b.iter(|| {
            levels.clear();
            levels.extend(black_box(&input));
            levels.propagate();
            black_box(levels.end(LEVELS - 1));
        })
    });
    group.bench_function("streamed in 512-sample chunks", |b| {
        b.iter(|| {
            levels.clear();
            for chunk in black_box(&input).chunks(512) {
                levels.extend(chunk);
                levels.propagate();
            }
            black_box(levels.end(LEVELS - 1));
        })
    });
    group.finish();
}

criterion_group!(benches, decimate, cascade);
criterion_main!(benches);
