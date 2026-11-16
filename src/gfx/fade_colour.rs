//! A colour brought toward another: each of the three low bytes s of the colour becomes s + ((t -
//! s) x amount >> 8), held to 0 .. 0xFF, t the byte of the target in the same place; the top byte
//! kept. `amount` 0x100 goes all the way.

/// One byte's step: (the byte, the high dword of the product).
use rc2k_core::gfx::pixels::fade_step as step;

/// `colour` toward `to` by `amount`; returns the new colour. Its one caller, `race::splash_mud`,
/// reads nothing else.
pub fn fade_colour(colour: u32, to: u32, amount: u32) -> u32 {
    let [b0, b1, b2, b3] = colour.to_le_bytes();
    let [t0, t1, t2, _] = to.to_le_bytes();
    let (r0, _) = step(b0.into(), t0.into(), amount);
    let (r1, _) = step(b1.into(), t1.into(), amount);
    let (r2, _) = step(b2.into(), t2.into(), amount);
    u32::from_le_bytes([r0 as u8, r1 as u8, r2 as u8, b3])
}
