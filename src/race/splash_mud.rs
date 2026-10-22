//! `race::splash_mud` (0x44A35F): one splash of mud onto the texture worked on
//! (`dword_623D5C`, n): its count `dword_623D90[n]` one more; a random place (x the low
//! byte, y the next of `math::next_random`); unless the mud map's strength there (as
//! `race::muddy_texel`'s) is below 0x20, or a random byte is above it, the 4 x 4 texels
//! from that place are faded (`gfx::fade_colour`, by each colour's top byte) toward one of
//! 16 patterns of `g_mudColours` (4 x 4 colours each, picked at random), their top bytes set,
//! and put back (`race::mud_texel`, `race::put_mud_texel`; untouched without the top bit).
//! The columns and rows counted in `dword_623D38`, `dword_623D3C` (4 each after).
//!
//! In:  nothing
//! Out: ebx = n, then the strength, then each tinted texel's colour; eax (y / 2) x 128
//!      (the strength's address's last part), the masked random byte, or the last texel's
//!      call; ecx = x, edx = y (+ 4 after the splash);
//!      esi the strength's address, then past the pattern; ebp kept; no flags read
//!
//! See `race/mud.rs`. One caller, `race::update_mud_texture`, once per splash. Refuses
//! what `muddy_texel` refuses before anything, and `put_mud_texel`'s later refusals (the
//! counts, the counters and the random state put back). Self-test: the original and the
//! Rust version on the mud tests' set-up with patterns of the test's, random states by lot;
//! the surface, the counts and the random state compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::fade_colour::fade_colour;
use crate::gfx::pixel_format::SCRATCH_RVA;
use crate::math::next_random::{STATE_RVA as RANDOM_RVA, next_random, own_random};
use crate::physics::flow::guard;
use crate::race::mud::{COLOURS_RVA, has_format};
use crate::race::mud_texel::mud_texel;
use crate::race::muddy_texel::{
    TEXTURE_RVA, each_mud_case, strength_at, surface_region, texture_and_mud,
};
use crate::race::put_mud_texel::put_mud_texel;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::splash_mud", rva(0x44A35F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const SPLASHED_RVA: usize = rva(0x623D90); // dword_623D90: each texture's count
pub const LAST_SPLASHED: u32 = 0x1000;
const COLUMN_RVA: usize = rva(0x623D38); // dword_623D38
const ROW_RVA: usize = rva(0x623D3C); // dword_623D3C

/// What a splash writes besides the texels and the format's scratch.
pub fn splash_regions() -> Vec<Region> {
    vec![
        (game_addr(SPLASHED_RVA) as u32, 0x10),
        at(RANDOM_RVA),
        (game_addr(COLUMN_RVA) as u32, 8),
    ]
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn splash_mud(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::mud::{splash_place, splash_lands, splash_pattern}`.
    use rc2k_core::race::mud::{
        MUD_TOP, SPLASH_SIDE, muddy, splash_lands, splash_pattern, splash_place, strength_row,
    };
    let (n, mud) = texture_and_mud()?;
    guard(&splash_regions(), || unsafe {
        let mut r = Regs { ebx: n, ..*entry };
        let count_at = SPLASHED_RVA + 4 * n as usize;
        game_write(count_at, game_read::<u32>(count_at).wrapping_add(1));
        (r.ecx, r.edx) = splash_place(next_random());
        r.esi = strength_at(mud, n, r.ecx, r.edx);
        r.ebx = u32::from(*(r.esi as *const u8));
        // The address's making leaves eax its last part, (y / 2) x 128.
        r.eax = strength_row(r.edx);
        let lands = splash_lands(r.ebx, || {
            r.eax = next_random() & 0xFF;
            r.eax
        });
        if !lands {
            return Ok(r);
        }
        r.eax = splash_pattern(next_random());
        r.esi = (game_addr(COLOURS_RVA) as u32).wrapping_add(r.eax);
        game_write(ROW_RVA, 0u32);
        for row in 0..SPLASH_SIDE {
            game_write(COLUMN_RVA, 0u32);
            for column in 0..SPLASH_SIDE {
                r.eax = mud_texel(r.eax, r.ecx, r.edx);
                if muddy(r.eax) {
                    let colour = (r.esi as *const u32).read_unaligned();
                    let faded = fade_colour(&Regs {
                        ebx: r.eax,
                        ecx: colour,
                        ebp: colour >> 24,
                        ..r
                    });
                    r.ebx = faded.ebx;
                    r.eax = put_mud_texel(&Regs {
                        eax: faded.eax | MUD_TOP,
                        ..r
                    })?;
                }
                r.esi = r.esi.wrapping_add(4);
                r.ecx = r.ecx.wrapping_add(1);
                game_write(COLUMN_RVA, column + 1);
            }
            r.ecx = r.ecx.wrapping_sub(4);
            r.edx = r.edx.wrapping_add(1);
            game_write(ROW_RVA, row + 1);
        }
        Ok(r)
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { splash_mud(r) }?))
}

/// What one splash may write: its own, the format's scratch, all of `g_surface`.
fn regions() -> Vec<Region> {
    let mut v = splash_regions();
    v.push(at(SCRATCH_RVA));
    v.push(surface_region());
    v
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// 256 colours of the test's in `g_mudColours` (the 16 patterns), until put back.
pub fn test_colours() -> Region {
    (game_addr(COLOURS_RVA) as u32, 0x400)
}

pub fn fill_test_colours() {
    for k in 0..0x100u32 {
        let alpha = [0x00u32, 0x40, 0x80, 0xFF][(k % 4) as usize];
        let colour = (alpha << 24) | (k.wrapping_mul(0x01_0F_37) & 0xFF_FFFF);
        unsafe { game_write(COLOURS_RVA + 4 * k as usize, colour) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    own_random(|| {
        let kept = [&splash_regions()[..], &[test_colours()]].concat();
        let saved = capture(&kept);
        fill_test_colours();
        let mut x = 0x7F4A_7C15u32;
        each_mud_case(&mut |what, pixels, reset| {
            for count in [0u32, 0xFFF] {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                let seed = x;
                check_state(
                    &mut t,
                    &format!("{what}, count {count:#x}, random state {seed:#x}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        reset();
                        let n: u32 = game_read(TEXTURE_RVA);
                        game_write(SPLASHED_RVA + 4 * n as usize, count);
                        game_write(RANDOM_RVA, seed);
                    },
                    &[&splash_regions()[..], &[pixels, at(SCRATCH_RVA)]].concat(),
                    &rust,
                );
            }
        });
        restore(&kept, &saved);
    });
    t.finish()
}
