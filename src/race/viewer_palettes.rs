//! `race::viewer_palettes` (0x41D8BF): for the viewer's page, kind 0x35's and 0x2A's first
//! texture and slot (`g_kindFirstTextures` / `g_kindFirstSlots` +0x350, +0x324) into
//! `dword_5BECB0` .. `dword_5BECBC`, and the six fonts' palettes pointed at
//! `unk_960FA0`, which gets the fonts' 256 colours (`g_fontPalettes`) taken apart in the
//! screen's format (`gfx::pixel_to_components`, `g_surfaceFormat`), made opaque and put
//! together in the texture format (`gfx::components_to_pixel`, `[off_5ABAC0 + 0xC] + 8`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_load_stage`. Shadow mode compares. Self-test: the game's fonts and
//! formats as they are (skipped without fonts or a texture format), against the original;
//! what it writes put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::gfx::pixel_format::PixelFormat;
use crate::physics::flow::a;
use crate::race::viewer::{get, put};

const FONT_PALETTES: usize = 0x51DEB0; // g_fontPalettes
const COLOURS: usize = 0x960FA0; // unk_960FA0: 256 dwords
const SURFACE_FORMAT: usize = 0x4F496C; // g_surfaceFormat
const TEXTURE_KINDS: usize = 0x5ABACC; // off_5ABAC0 + 0xC
/// (to, from): the textures and slots kept.
const KEPT: [(usize, usize); 4] = [
    (0x5BECB0, 0x5D0E38), // g_kindFirstTextures + 0x350
    (0x5BECB4, 0x5D1E38), // g_kindFirstSlots + 0x350
    (0x5BECB8, 0x5D0E0C), // g_kindFirstTextures + 0x324
    (0x5BECBC, 0x5D1E0C), // g_kindFirstSlots + 0x324
];
/// The palette fields pointed at the colours, in the original's order.
const PALETTES: [usize; 6] = [0x51EAF8, 0x51EB58, 0x51EB18, 0x51EB78, 0x51EAD8, 0x51EB38];

/// The Rust version.
pub unsafe fn viewer_palettes() {
    for (to, from) in KEPT {
        put(to, get(from));
    }
    for at in PALETTES {
        put(at, a(COLOURS));
    }
    unsafe {
        let screen = (a(SURFACE_FORMAT) as *const PixelFormat).read_unaligned();
        let texture = (get(TEXTURE_KINDS).wrapping_add(8) as *const PixelFormat).read_unaligned();
        // Converted: `rc2k_core::race::viewer_flow::{font_colour, FONT_COLOURS}` (colour
        // by colour, as the original).
        use rc2k_core::race::viewer_flow::{FONT_COLOURS, font_colour};
        let from = get(FONT_PALETTES) as *const u32;
        let to = a(COLOURS) as *mut u32;
        for n in 0..FONT_COLOURS {
            let colour = font_colour(
                from.add(n).read_unaligned(),
                |c| crate::gfx::pixel_to_components::pixel_to_components(&screen, c),
                |p| crate::gfx::components_to_pixel::components_to_pixel(&texture, p),
            );
            to.add(n).write_unaligned(colour);
        }
    }
}
