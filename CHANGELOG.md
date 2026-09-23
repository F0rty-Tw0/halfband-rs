# 0.1.0 - 2026-09-23

Extracted from cqt-rs 0.2.0, where these types were private modules.

## Added

- `HalfBand`: Kaiser-windowed half-band decimation filter with `design`,
  `taps_for`, `taps`, `delay` and `decimate_block`.
- `Levels`: multi-rate cascade with absolute indexing: `new`, `clear`,
  `extend`, `extend_zeros`, `reserve`, `propagate`, `frame`,
  `discard_before`, `retained`, `end` and `history_from`.
- `parallel` feature (default) that spreads long decimations over rayon.
