//! Measures the frequency response of a [`HalfBand`] design empirically,
//! through the public `decimate_block`: a sine at each frequency of a sweep
//! is decimated, and the output RMS over the input RMS is the gain.
//!
//! Every sine completes a whole number of cycles over the measured block, so
//! the RMS carries no windowing bias; the floor of the measurement is the
//! f32 rounding of the samples themselves, around -130 dB.
//!
//! Run with `cargo run --release --example response -- [transition]
//! [attenuation_db] [--svg path]`; the defaults are `0.2 80`.

use halfband_rs::HalfBand;
use std::f64::consts::PI;
use std::fmt::Write as _;

/// Decimated samples per measurement.
const N: usize = 8_192;
/// Plotted gain range in dB.
const FLOOR: f64 = -140.0;
const CEIL: f64 = 5.0;

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

/// Gain in dB at `f` (a fraction of the input Nyquist frequency).
fn gain_db(filter: &HalfBand, f: f64, odd: &mut Vec<f32>) -> f64 {
    let first = filter.delay();
    let input: Vec<f32> = (0..first + 2 * N + filter.delay())
        .map(|i| (PI * f * i as f64 + 0.3).sin() as f32)
        .collect();
    let mut out = vec![0.0; N];
    filter.decimate_block(&input, first, &mut out, odd);
    20.0 * (rms(&out) / rms(&input[first..first + 2 * N])).log10()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut numbers, mut svg) = (Vec::new(), None);
    while let Some(arg) = args.next() {
        if arg == "--svg" {
            svg = args.next();
        } else {
            numbers.push(
                arg.parse::<f64>()
                    .expect("numeric transition / attenuation"),
            );
        }
    }
    let transition = numbers.first().copied().unwrap_or(0.2);
    let attenuation = numbers.get(1).copied().unwrap_or(80.0);
    let filter = HalfBand::design(transition, attenuation);
    let (pass, stop) = ((1.0 - transition) / 2.0, (1.0 + transition) / 2.0);

    // f = k / N gives k whole cycles over the block. k stays odd so it is
    // never N / 2, which would alias the output exactly onto its Nyquist
    // frequency, where the RMS depends on the phase.
    let mut odd = Vec::new();
    let sweep: Vec<(f64, f64)> = (1..N)
        .step_by(4)
        .map(|k| k as f64 / N as f64)
        .map(|f| (f, gain_db(&filter, f, &mut odd)))
        .collect();
    let ripple = sweep
        .iter()
        .filter(|p| p.0 <= pass)
        .map(|p| p.1.abs())
        .fold(0.0, f64::max);
    let (worst_f, worst) = sweep
        .iter()
        .filter(|p| p.0 >= stop)
        .map(|&(f, g)| (f, -g))
        .fold((0.0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });

    println!("HalfBand::design({transition}, {attenuation})");
    println!("taps {}  delay {}", filter.taps(), filter.delay());
    println!("pass band  f <= {pass:.3}  max ripple  {ripple:.2e} dB");
    println!("stop band  f >= {stop:.3}  min attenuation {worst:.1} dB (at f = {worst_f:.3})");
    println!(
        "({} frequencies, f as a fraction of the input Nyquist frequency)",
        sweep.len()
    );

    if let Some(path) = svg {
        std::fs::write(
            &path,
            plot(&sweep, pass, stop, &filter, transition, attenuation),
        )
        .expect("write svg");
        println!("wrote {path}");
    }
}

/// Hand-written SVG: transparent background, mid-grey axes and text, one
/// strong line colour, so it reads on light and dark pages alike.
fn plot(sweep: &[(f64, f64)], pass: f64, stop: f64, filter: &HalfBand, t: f64, a: f64) -> String {
    let (w, h, left, right, top, bottom) = (720.0, 380.0, 64.0, 20.0, 36.0, 48.0);
    let x = |f: f64| left + f * (w - left - right);
    let y = |db: f64| top + (CEIL - db.clamp(FLOOR, CEIL)) / (CEIL - FLOOR) * (h - top - bottom);
    let mut s = String::new();
    let _ = write!(
        s,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" font-family="sans-serif" font-size="12" fill="#888">
<rect x="{:.1}" y="{top}" width="{:.1}" height="{:.1}" fill="#2da44e" fill-opacity="0.14"/>
<rect x="{:.1}" y="{top}" width="{:.1}" height="{:.1}" fill="#cf222e" fill-opacity="0.12"/>
<text x="{:.1}" y="{:.1}">pass band</text><text x="{:.1}" y="{:.1}">stop band</text>
"##,
        x(0.0),
        x(pass) - x(0.0),
        h - top - bottom,
        x(stop),
        x(1.0) - x(stop),
        h - top - bottom,
        x(0.0) + 6.0,
        top + 30.0,
        x(stop) + 6.0,
        top + 30.0,
    );
    for i in 0..=10 {
        let f = f64::from(i) / 10.0;
        let _ = writeln!(
            s,
            r##"<line x1="{0:.1}" y1="{top}" x2="{0:.1}" y2="{1:.1}" stroke="#888" stroke-opacity="0.25"/><text x="{0:.1}" y="{2:.1}" text-anchor="middle">{f:.1}</text>"##,
            x(f),
            h - bottom,
            h - bottom + 16.0
        );
    }
    for db in (-140..=0).step_by(20) {
        let db = f64::from(db);
        let _ = writeln!(
            s,
            r##"<line x1="{left}" y1="{0:.1}" x2="{1:.1}" y2="{0:.1}" stroke="#888" stroke-opacity="0.25"/><text x="{2:.1}" y="{3:.1}" text-anchor="end">{db}</text>"##,
            y(db),
            w - right,
            left - 6.0,
            y(db) + 4.0
        );
    }
    let points: Vec<String> = sweep
        .iter()
        .map(|&(f, g)| format!("{:.1},{:.1}", x(f), y(g)))
        .collect();
    let _ = write!(
        s,
        r##"<line x1="{left}" y1="{0:.1}" x2="{1:.1}" y2="{0:.1}" stroke="#888" stroke-dasharray="4 4"/>
<rect x="{left}" y="{top}" width="{2:.1}" height="{3:.1}" fill="none" stroke="#888"/>
<polyline points="{4}" fill="none" stroke="#e8590c" stroke-width="1.5" stroke-linejoin="round"/>
<text x="{5:.1}" y="{6:.1}" text-anchor="middle">frequency (fraction of input Nyquist)</text>
<text transform="translate(16 {7:.1}) rotate(-90)" text-anchor="middle">gain (dB)</text>
<text x="{left}" y="22" font-size="13">HalfBand::design({t}, {a}): {8} taps, measured through decimate_block (dashed: -{a} dB)</text>
</svg>
"##,
        y(-a),
        w - right,
        w - left - right,
        h - top - bottom,
        points.join(" "),
        (left + w - right) / 2.0,
        h - 10.0,
        (top + h - bottom) / 2.0,
        filter.taps(),
    );
    s
}
