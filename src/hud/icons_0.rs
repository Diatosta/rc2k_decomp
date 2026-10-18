//! `hud::icons_0` (0x4B57D8): layout 0's icon: no shift, the scale `dword_75C0B8`, the first entry at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them
//!
//! The icons through `part` (1), in the game `draw_icon`'s Rust version. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose calls go to a stand-in, against
//! the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_icon::{SCALE_RVA, SHIFT_RVA};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::icons_0", rva(0x4B57D8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `draw_icon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn icons_0(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let icon = |g: &mut Regs, ebx: u32, edi: u32, part: &mut Part| {
            [g.ebx, g.edi] = [ebx, edi];
            part(1, g)
        };
        let mut part = part;
        game_write(SHIFT_RVA, 0u32);
        game_write(SCALE_RVA, game_read::<u32>(rva(0x75C0B8)));
        icon(&mut g, 0xC8, 0, &mut part)?;
    }
    Ok(g)
}

/// In the game: `draw_icon`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::draw_icon as di;
    *r = unsafe { di::draw_icon(r, &mut crate::hud::add_quad::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { icons_0(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0xC7, 0x05, 0x70, 0xC0, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(0x4B57D8, 0x26, entry, &[0x20], &[(0x4B587E, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = [SCALE_RVA, SHIFT_RVA, TEXTURE_RVA].map(at).to_vec();
    let saved = capture(&regions);
    run_both(
        &mut t,
        "once",
        &copy,
        &regions,
        &|| unsafe {
            restore(&regions, &saved);
            game_write(TEXTURE_RVA, 7u32);
        },
        &|r| unsafe { icons_0(r, &mut test_part) },
    );
    restore(&regions, &saved);
    t.finish()
}
