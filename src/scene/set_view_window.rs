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

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, EDX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats,
    dispatch_state_floats,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_view_window", rva(0x44E47B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

/// The float places: the scale, the divisor, the farthest depth and its reciprocal, the
/// sizes less one; the two depths rounded from floats (`fistp`).
fn float_places() -> Vec<(Region, Tolerance)> {
    let a = |rva: usize| game_addr(rva) as u32;
    vec![
        (at(SCALE_RVA), FLOAT),
        (at(DIVISOR_RVA), FLOAT),
        ((a(FARTHEST_RVA), 8), FLOAT),
        ((a(FARTHEST_RVA) + 8, 4), ROUNDED),
        (at(CLIP_FARTHEST_RVA), ROUNDED),
        ((a(LESS_ONE_RVA), 8), FLOAT),
    ]
}

// ---- hook ----

/// Everything it writes.
pub fn regions() -> [Region; 8] {
    let block = |rva: usize, len: usize| (game_addr(rva) as u32, len);
    [
        block(DEPTH_RVA, 8),
        block(SCALE_RVA, 4),
        block(MIDDLE_RVA, 0x10),
        // The farthest depth's three, the edges, the edges as floats, the offsets.
        block(FARTHEST_RVA, 0x34),
        at(DIVISOR_RVA),
        at(CLIP_FARTHEST_RVA),
        block(LESS_ONE_RVA, 8),
        at(CHECKED_RVA),
    ]
}

/// What it reads and a test sets.
pub fn inputs() -> [Region; 9] {
    let block = |rva: usize, len: usize| (game_addr(rva) as u32, len);
    [
        at(REACH_RVA),
        block(SIZE_RVA, 8),
        at(IN_RACE_RVA),
        block(OFFSETS_RVA, 8),
        at(SCALE_A_RVA),
        at(SCALE_B_RVA),
        at(SCALE_C_RVA),
        at(NEARER_RVA),
        // flt_624220, flt_624224 (written), flt_624228, flt_62422C
        block(FADE_B_RVA, 0x10),
    ]
}

/// Settings of a test's, by lot.
pub unsafe fn set_inputs(seed: &mut u32, case: usize) {
    let mut lot = || {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *seed >> 8
    };
    let mut float = |around: f32| {
        match lot() % 9 {
            0 => 0.0,
            1 => -around,
            _ => around * (0.25 + (lot() % 4000) as f32 / 1000.0),
        }
        .to_bits()
    };
    unsafe {
        let fades = [float(1.0), 0, float(0.5), float(0.9)];
        game_write(FADE_B_RVA, fades);
        game_write(SCALE_A_RVA, float(1.0));
        game_write(SCALE_C_RVA, float(0.002));
        game_write(
            SCALE_B_RVA,
            [0x100u32, 0x80, 0, 0xFFFF_FF00][lot() as usize % 4],
        );
        game_write(
            REACH_RVA,
            match case % 7 {
                0 => 0,
                1 => 0x0800_0001,
                2 => 0x400,
                _ => lot() % 0x300,
            },
        );
        game_write(
            SIZE_RVA,
            match case % 5 {
                0 => [0x320u32, 0x258],
                1 => [0x8000_0001, 0xFFFF_FFFF],
                _ => [lot() % 0x800, lot() % 0x600],
            },
        );
        game_write(IN_RACE_RVA, [0u32, 1, 1, 2][lot() as usize % 4]);
        game_write(OFFSETS_RVA, [lot() % 0x200, (lot() % 0x200).wrapping_neg()]);
        game_write(
            NEARER_RVA,
            [0u32, 5, 10, 11, 100, 0xFFFF_FFFF, 0x8000_0000, 0x7FFF_FFFF][lot() as usize % 8],
        );
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { set_view_window_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (regions, inputs) = (regions(), inputs());
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x1B87_3593u32;
    let places = float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    for case in 0..60 {
        unsafe { set_inputs(&mut seed, case) };
        let set = capture(&inputs);
        at_test_precisions(&tolerant, &mut |word, held| {
            check_state_floats(
                &mut t,
                "scene::set_view_window",
                &format!("control word {word:#x}, case {case}: inputs {set:x?}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    restore(&regions, &vec![0xAA; saved.0.len()]);
                    restore(&inputs, &set);
                },
                &regions,
                held,
                &rust,
            );
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
