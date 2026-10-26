//! `race::throw_mud_texel` (0x44A2A2): one throw of mud onto the texture worked on
//! (`dword_623D5C`, n): while its count `dword_623D80[n]` is below 0x60000, the texel of
//! that number (`race::muddy_texel`, the places in their shuffled order) and the count
//! one more; then `dword_623D30` + 0x10000000, and when that carries (every 16th) a texel
//! at a random number too (`math::next_random`).
//!
//! In:  nothing
//! Out: ebx = n when the count is used up; else the registers as the last `muddy_texel`
//!      leaves them (ebx n again after the first when there is no second); no flags read
//!
//! See `race/mud.rs`. One caller, `race::update_mud_texture`, once per thrown unit.
//! Refuses what `muddy_texel` refuses (a texture past 3, no mud) before anything, and its
//! later refusals (the counts and the random state put back). Self-test: the original and
//! the Rust version on the mud tests' set-up, counts at and below the end, sums that carry
//! and not; the surface, the counts and the random state compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::pixel_format::SCRATCH_RVA;
use crate::math::next_random::{STATE_RVA as RANDOM_RVA, next_random, own_random};
use crate::physics::flow::guard;
use crate::race::mud::has_format;
use crate::race::muddy_texel::{
    STRENGTH_RVA, TEXTURE_RVA, each_mud_case, muddy_texel, texture_and_mud,
};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::throw_mud_texel", rva(0x44A2A2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const THROWN_RVA: usize = rva(0x623D80); // dword_623D80: each texture's count
pub use rc2k_core::race::mud::LAST_THROWN;
pub const EXTRA_SUM_RVA: usize = rva(0x623D30); // dword_623D30

/// The counts, the sum and the random state: what a throw writes besides the texels.
pub fn throw_regions() -> Vec<Region> {
    vec![
        (game_addr(THROWN_RVA) as u32, 0x10),
        at(EXTRA_SUM_RVA),
        at(RANDOM_RVA),
        at(STRENGTH_RVA),
    ]
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn throw_mud_texel(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::mud::{LAST_THROWN, extra_throw}`.
    let (n, _) = texture_and_mud()?;
    guard(&throw_regions(), || unsafe {
        let mut r = Regs { ebx: n, ..*entry };
        let count_at = THROWN_RVA + 4 * n as usize;
        let count: u32 = game_read(count_at);
        if count >= LAST_THROWN {
            return Ok(r);
        }
        game_write(count_at, count + 1);
        r.eax = count;
        r = muddy_texel(&r)?;
        r.ebx = n;
        let (sum, carried) = rc2k_core::race::mud::extra_throw(game_read(EXTRA_SUM_RVA));
        game_write(EXTRA_SUM_RVA, sum);
        if !carried {
            return Ok(r);
        }
        r.eax = next_random();
        muddy_texel(&r)
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { throw_mud_texel(r) }?))
}

/// What one throw may write: the counts, the sum, the random state, the strength, the
/// format's scratch, and the texture's pixels (all of `g_surface`: the random texel's
/// place is not known before).
fn regions() -> Vec<Region> {
    let mut v = throw_regions();
    v.push(at(SCRATCH_RVA));
    v.push(crate::race::muddy_texel::surface_region());
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    own_random(|| {
        let kept = throw_regions();
        let saved = capture(&kept);
        let mut case = 0u32;
        each_mud_case(&mut |what, pixels, reset| {
            for (count, sum) in [
                (0u32, 0u32),
                (0x1234, 0xF000_0000),
                (LAST_THROWN - 1, 0xEFFF_FFFF),
                (LAST_THROWN, 0xF000_0000),
                (0xFFFF_FFFF, 0),
            ] {
                case += 1;
                let seed = 0x0BAD_5EEDu32 ^ case.wrapping_mul(0x9E37_79B9);
                check_state(
                    &mut t,
                    &format!("{what}, count {count:#x}, sum {sum:#x}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        reset();
                        let n: u32 = game_read(TEXTURE_RVA);
                        game_write(THROWN_RVA + 4 * n as usize, count);
                        game_write(EXTRA_SUM_RVA, sum);
                        game_write(RANDOM_RVA, seed);
                    },
                    &[&throw_regions()[..], &[pixels, at(SCRATCH_RVA)]].concat(),
                    &rust,
                );
            }
        });
        restore(&kept, &saved);
    });
    t.finish()
}
