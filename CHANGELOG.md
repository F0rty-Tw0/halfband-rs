# Unreleased

## Added

- `examples/octave_frames.rs`: streams audio in irregular chunks and reads
  aligned frames from every level.
- `examples/response.rs`: measures a design's frequency response through the
  public API and can plot it as SVG.
- Criterion benchmarks comparing `decimate_block` with a plain FIR, and the
  `Levels` cascade in bulk and streamed.

## Changed

- Minimum supported Rust version lowered from 1.98 to 1.85.
- `HalfBand::design` documents that the delivered stop-band attenuation can
  fall about 1 dB short of the requested value.

# 0.1.0 - 2026-09-23

Extracted from cqt-rs 0.2.0, where these types were private modules.

## Added

- `HalfBand`: Kaiser-windowed half-band decimation filter with `design`,
  `taps_for`, `taps`, `delay` and `decimate_block`.
- `Levels`: multi-rate cascade with absolute indexing: `new`, `clear`,
  `extend`, `extend_zeros`, `reserve`, `propagate`, `frame`,
  `discard_before`, `retained`, `end` and `history_from`.
- `parallel` feature (default) that spreads long decimations over rayon.
