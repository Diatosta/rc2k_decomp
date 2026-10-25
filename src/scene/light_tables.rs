//! The tables of 0x100 colours the lighting picks from, and the two functions that fill one:
//! shared by their three functions.

unsafe fn store(table: *mut [u32; 0x100], entries: &[u32; 0x100]) {
    unsafe { table.write_unaligned(*entries) };
}

/// `scene::fill_light_ramp` (0x44F169): entry n = the first colour plus n over 255 of
/// the way to the second, each of three parts a byte, 0xFF above them. Converted:
/// `rc2k_core::scene::light::ramp`.
pub unsafe fn fill_light_ramp(colours: *const [u32; 6], table: *mut [u32; 0x100]) {
    unsafe {
        let c = colours.read_unaligned();
        let entries = rc2k_core::scene::light::ramp([c[0], c[1], c[2]], [c[3], c[4], c[5]]);
        store(table, &entries);
    }
}

/// `scene::fill_light_scale` (0x44F1BC): entry n = the colour times n over 255.
/// Converted: `rc2k_core::scene::light::scale`.
pub unsafe fn fill_light_scale(colour: *const [u32; 3], table: *mut [u32; 0x100]) {
    unsafe {
        let c = colour.read_unaligned();
        let entries = rc2k_core::scene::light::scale(c);
        store(table, &entries);
    }
}
