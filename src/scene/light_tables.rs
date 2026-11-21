//! The tables of 0x100 colours the lighting picks from, and the two functions that fill one:
//! shared by their three functions.

/// `scene::fill_light_ramp`: entry n = the first colour plus n over 255 of the way to the
/// second, each of three parts a byte, 0xFF above them. Logic: `rc2k_core::scene::light::ramp`.
pub fn fill_light_ramp(c: [u32; 6], table: &mut [u32; 0x100]) {
    *table = rc2k_core::scene::light::ramp([c[0], c[1], c[2]], [c[3], c[4], c[5]]);
}

/// `scene::fill_light_scale`: entry n = the colour times n over 255. Logic:
/// `rc2k_core::scene::light::scale`.
pub fn fill_light_scale(c: [u32; 3], table: &mut [u32; 0x100]) {
    *table = rc2k_core::scene::light::scale(c);
}
