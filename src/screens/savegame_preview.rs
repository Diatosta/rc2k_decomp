//! What the two panels of the load-game screen's preview need loaded (their tests check
//! it). The drawing is `rc2k_core::screens::savegame_preview` (converted 2026-10-20).

/// The fonts both panels draw with: rows, title, row numbers.
pub const FONTS: [usize; 3] = [0x51E584, 0x51E5E4, 0x51E5A4];
/// The headings, "-" and "n/d".
pub const STRINGS: [u32; 5] = [0x4E5, 0x4E6, 0x4E7, 0x57A, 0x578];
