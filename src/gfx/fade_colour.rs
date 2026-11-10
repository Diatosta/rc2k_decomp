//! `gfx::fade_colour` (0x44A4C5): a colour brought toward another: each of the three low
//! bytes s of ebx becomes s + ((t - s) x ebp >> 8) (`imul`, `sar`), held to 0 .. 0xFF, t
//! the byte of ecx in the same place; the top byte kept.
//!
//! Takes: the colour, the colour faded to, how far (0x100 all the way)
//!
//! Callers: 0x425A23, `race::splash_mud` (0x44A403), 0x44CB15.

/// One byte's step: (the byte, the high dword of the `imul`). Converted:
/// `rc2k_core::gfx::pixels::fade_step`.
use rc2k_core::gfx::pixels::fade_step as step;

/// `colour` (ebx) toward `to` (ecx) by `amount` (ebp);
/// the colour (eax = ebx). Its one caller, `race::splash_mud`, reads nothing else it
/// leaves (esi, edx).
pub fn fade_colour(colour: u32, to: u32, amount: u32) -> u32 {
    let [b0, b1, b2, b3] = colour.to_le_bytes();
    let [t0, t1, t2, _] = to.to_le_bytes();
    let (r0, _) = step(b0.into(), t0.into(), amount);
    let (r1, _) = step(b1.into(), t1.into(), amount);
    let (r2, _) = step(b2.into(), t2.into(), amount);
    u32::from_le_bytes([r0 as u8, r1 as u8, r2 as u8, b3])
}
