//! The car's damage and wear model under `physics::damage_step` (stream B, 2026-10-18):
//! `physics::damage_factors` (0x4AB5E6), `physics::damage_wear` (0x4ABEDF),
//! `physics::damage_gauges` (0x4AC817). Their figures: the work car's (`g_car`,
//! 0x719FD8 ..: the damage gauges `unk_71A284` .. `unk_71A33C`, the handling factors
//! `unk_71A410` ..), the model's own (`flt_75B340` .. 0x75B940: shares, tables of four
//! (x0, y0, x1, y1) for `physics::flow::table_lerp`, rates). What each figure is, is a
//! reading. The three are long straight runs: their Rust was generated from the
//! disassembly by idiom (x87 runs as blocks, the clamps as `hold_low` / `hold_high`, the
//! table lerps, the compares as `if`), the rest written by hand.
//!
//! This file: the regions and the shared test.

/// 1.0 (`fld1; fstp`).
pub const ONE: u32 = 0x3F80_0000;
