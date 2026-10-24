//! `scene::set_view_window` (0x44E47B): the view's window, scale and depths from its
//! size and the scene's settings.
//!
//! In:  nothing
//! Out: eax = the low half and edx = the high half of `dword_63DC54` times 0x10000,
//!      ebx = 0x10000; everything else preserved
//!
//! From the window's width and height (`dword_62408C`, `dword_624090`), its offsets
//! (`dword_623FB0`, `dword_623FB4`) and `dword_60A87C` (how far is seen, a reading):
//! - `dword_63DC58` = `dword_63DC54` = `dword_60A87C` times 0x20;
//! - the middle (`dword_624094`, `dword_624098`) = half of each, unsigned;
//! - the clip rectangle as whole numbers `dword_6241D4` (top), `D8` (bottom), `DC`
//!   (left), `E0` (right): 0, 0, width, height with `g_inRace` at 1, else each less its
//!   offset;
//! - the scale `flt_624088` = the screen's width times `dword_606294` times
//!   `dword_624080` (a whole number) times `flt_623DCC`;
//! - `flt_624224` = (100 less `dword_60AC80`, held to 90 .. 100 by the differences'
//!   signs) times `flt_5C5CFC`; the farthest depth `flt_6241C8` = `dword_63DC54` times
//!   0x10000 (its low half, signed) over that, `flt_6241CC` = 1 over it,
//!   `dword_6241D0` = it times `flt_624228` over `flt_624220` over 65536, and
//!   `g_clipFarthest` = it times `flt_62422C`, both rounded;
//! - as floats: the offsets (`flt_6241F4`, `F8`), the clip rectangle
//!   (`g_clipScreenLeft` ..), the middle (`flt_62409C`, `flt_6240A0`), and the width
//!   and height less `flt_5C5D38` (`flt_6D18E8`, `EC`).
//!
//! `dword_62400C` is left holding the low half.
//! Twelve callers.
//! Self-test: 60 settings by lot on the game's variables (the screen's width as it is), at
//! the test precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, EBX, EDX, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::screens::harness::Left;

// ---- game data ----

const REACH_RVA: usize = rva(0x60A87C); // dword_60A87C
const DEPTH_RVA: usize = rva(0x63DC54); // dword_63DC54, dword_63DC58
pub const SIZE_RVA: usize = rva(0x62408C); // dword_62408C, dword_624090
const MIDDLE_RVA: usize = rva(0x624094); // dword_624094, dword_624098
const MIDDLE_FLOATS_RVA: usize = rva(0x62409C); // flt_62409C, flt_6240A0
const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const OFFSETS_RVA: usize = rva(0x623FB0); // dword_623FB0, dword_623FB4
const EDGES_RVA: usize = rva(0x6241D4); // dword_6241D4 top, D8 bottom, DC left, E0 right
const EDGE_FLOATS_RVA: usize = rva(0x6241E4); // g_clipScreenTop, Bottom, Left, Right
const OFFSET_FLOATS_RVA: usize = rva(0x6241F4); // flt_6241F4, flt_6241F8
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCALE_A_RVA: usize = rva(0x606294); // dword_606294, a float
const SCALE_B_RVA: usize = rva(0x624080); // dword_624080
const SCALE_C_RVA: usize = rva(0x623DCC); // flt_623DCC
const SCALE_RVA: usize = rva(0x624088); // flt_624088
const NEARER_RVA: usize = rva(0x60AC80); // dword_60AC80
const NEARER_FACTOR_RVA: usize = rva(0x5C5CFC); // flt_5C5CFC
const DIVISOR_RVA: usize = rva(0x624224); // flt_624224
const FARTHEST_RVA: usize = rva(0x6241C8); // flt_6241C8, flt_6241CC, dword_6241D0
const FADE_A_RVA: usize = rva(0x624228); // flt_624228
const FADE_B_RVA: usize = rva(0x624220); // flt_624220
const CLIP_FACTOR_RVA: usize = rva(0x62422C); // flt_62422C
const CLIP_FARTHEST_RVA: usize = rva(0x624230); // g_clipFarthest
const TURN_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0
const ONE_RVA: usize = rva(0x5C5D38); // flt_5C5D38
const LESS_ONE_RVA: usize = rva(0x6D18E8); // flt_6D18E8, flt_6D18EC

// ---- Rust version ----

pub unsafe fn set_view_window() -> Left {
    unsafe {
        let depth = game_read::<u32>(REACH_RVA).wrapping_mul(0x20);
        game_write(DEPTH_RVA, [depth, depth]);
        let [width, height]: [u32; 2] = game_read(SIZE_RVA);
        let middle = [width >> 1, height >> 1];
        game_write(MIDDLE_RVA, middle);
        let offsets: [u32; 2] = game_read(OFFSETS_RVA);
        // top, bottom, left, right
        let edges = if game_read::<u32>(IN_RACE_RVA) == 1 {
            [0, height, 0, width]
        } else {
            [
                offsets[1].wrapping_neg(),
                height.wrapping_sub(offsets[1]),
                offsets[0].wrapping_neg(),
                width.wrapping_sub(offsets[0]),
            ]
        };
        game_write(EDGES_RVA, edges);
        game_write(
            SCALE_RVA,
            x87::int_float_int_float(
                game_read(SCREEN_WIDTH_RVA),
                game_read(SCALE_A_RVA),
                game_read(SCALE_B_RVA),
                game_read(SCALE_C_RVA),
            ),
        );
        // `cmp; jns` and `cmp; js`: by the differences' signs.
        let mut nearer = 100u32.wrapping_sub(game_read(NEARER_RVA));
        if (nearer.wrapping_sub(0x5A) as i32) < 0 {
            nearer = 0x5A;
        }
        if (nearer.wrapping_sub(0x64) as i32) >= 0 {
            nearer = 0x64;
        }
        let divisor = x87::int_times(nearer as i32, game_read(NEARER_FACTOR_RVA));
        game_write(DIVISOR_RVA, divisor);
        let product = depth as u64 * 0x1_0000;
        let low = product as u32;
        game_write(CHECKED_RVA, low);
        let farthest = x87::int_over(low as i32, divisor);
        game_write(
            FARTHEST_RVA,
            [
                farthest,
                x87::reciprocal(farthest),
                x87::mul_div_div_to_int(
                    farthest,
                    game_read(FADE_A_RVA),
                    game_read(FADE_B_RVA),
                    game_read(TURN_RVA),
                ) as u32,
            ],
        );
        game_write(
            CLIP_FARTHEST_RVA,
            x87::mul_to_int(farthest, game_read(CLIP_FACTOR_RVA)),
        );
        let float = |value: u32| x87::from_int(value as i32);
        game_write(OFFSET_FLOATS_RVA, offsets.map(float));
        game_write(EDGE_FLOATS_RVA, edges.map(float));
        game_write(MIDDLE_FLOATS_RVA, middle.map(float));
        let one: u32 = game_read(ONE_RVA);
        game_write(
            LESS_ONE_RVA,
            [width, height].map(|size| x87::int_less(size as i32, one)),
        );
        Left::nothing()
            .with(EAX, low)
            .with(EBX, 0x1_0000)
            .with(EDX, (product >> 32) as u32)
    }
}

/// The converted `set_view_window` (plain floats): the hook's. The x87 version above
/// stays for its Rust callers in modules not converted (race, `choose_view_window`,
/// `prepare_menu_scene`).
// Converted: `rc2k_core::scene::view_window::window_settings`
pub unsafe fn set_view_window_plain() -> Left {
    use rc2k_core::scene::view_window::{WindowInputs, window_settings};
    unsafe {
        let f = |r: usize| f32::from_bits(game_read(r));
        let i = WindowInputs {
            reach: game_read(REACH_RVA),
            size: game_read(SIZE_RVA),
            offsets: game_read(OFFSETS_RVA),
            in_race: game_read::<u32>(IN_RACE_RVA) == 1,
            screen_width: game_read(SCREEN_WIDTH_RVA),
            scale: (f(SCALE_A_RVA), game_read(SCALE_B_RVA), f(SCALE_C_RVA)),
            nearer: game_read(NEARER_RVA),
            nearer_factor: f(NEARER_FACTOR_RVA),
            fade: [f(FADE_A_RVA), f(FADE_B_RVA)],
            turn: f(TURN_RVA),
            clip_factor: f(CLIP_FACTOR_RVA),
            one: f(ONE_RVA),
        };
        let w = window_settings(&i);
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
        game_write(DEPTH_RVA, [w.depth, w.depth]);
        game_write(MIDDLE_RVA, w.middle);
        game_write(EDGES_RVA, w.edges);
        game_write(SCALE_RVA, w.scale.to_bits());
        game_write(DIVISOR_RVA, w.divisor.to_bits());
        let low = w.product as u32;
        game_write(CHECKED_RVA, low);
        game_write(
            FARTHEST_RVA,
            [
                w.farthest.to_bits(),
                w.reciprocal.to_bits(),
                w.fade_depth as u32,
            ],
        );
        game_write(CLIP_FARTHEST_RVA, w.clip_farthest);
        let put = |rva: usize, v: &[f32]| {
            for (n, x) in bits(v).into_iter().enumerate() {
                game_write(rva + 4 * n, x);
            }
        };
        put(OFFSET_FLOATS_RVA, &w.offset_floats);
        put(EDGE_FLOATS_RVA, &w.edge_floats);
        put(MIDDLE_FLOATS_RVA, &w.middle_floats);
        put(LESS_ONE_RVA, &w.less_one);
        Left::nothing()
            .with(EAX, low)
            .with(EBX, 0x1_0000)
            .with(EDX, (w.product >> 32) as u32)
    }
}
