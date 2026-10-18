//! `race::loading_bar_quad` (0x446861): the loading bar, a red (`dword_623DE8` 0xFF0000)
//! flat rectangle along the bottom: x from 0 to the screen's width times `dword_623700`
//! (held to 0 .. 100) over 100, y from 0x24D to 0x258 scaled from 600 to the screen's
//! height; `dword_6240C4` -1; `race::draw_screen_quad`.
//!
//! In:  nothing
//! Out: eax the right edge, ebx 100, ecx 0x258, edx the rest of its division; then as the
//!      quad leaves them (its callers return at once)
//!
//! Caller: `race::loading_bar_frame`. The quad through `part` (1), in the game its Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose quad
//! goes to a stand-in, against the Rust version, shares by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::loading_bar_quad", rva(0x446861), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const QUAD_IN_RVA: usize = rva(0x623DE8); // colour; x at +0x28 / +0x2C, y at +0x38 / +0x3C
pub const SHARE_RVA: usize = rva(0x623700); // dword_623700: the share loaded, 0 .. 100
const SWITCH_RVA: usize = rva(0x6240C4); // dword_6240C4
const SCREEN_RVA: [usize; 2] = [rva(0x4F494C), rva(0x4F4950)];

/// A call by place: 1 `draw_screen_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `v` times `size` over `of` (`mul`, `div`): the quotient and the rest.
fn scaled(v: u32, size: u32, of: u32) -> (u32, u32) {
    let n = u64::from(v) * u64::from(size);
    ((n / u64::from(of)) as u32, (n % u64::from(of)) as u32)
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn loading_bar_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let [w, h] = SCREEN_RVA.map(|r| game_read::<u32>(r));
        let q = |o: usize, v: u32| game_write(QUAD_IN_RVA + o, v);
        let (left, _) = scaled(0, w, 0x320);
        q(0x28, left);
        q(0, 0xFF_0000);
        q(0x38, scaled(0x24D, h, 0x258).0);
        q(0x3C, scaled(0x258, h, 0x258).0);
        let share: u32 = game_read(SHARE_RVA);
        let share = if (share as i32) < 0 { 0 } else { share };
        let share = if (share as i32).wrapping_sub(0x64) < 0 {
            share
        } else {
            0x64
        };
        let (right, rest) = scaled(w, share, 0x64);
        g.eax = right.wrapping_add(left);
        q(0x2C, g.eax);
        [g.ebx, g.ecx, g.edx] = [0x64, 0x258, rest];
        game_write(SWITCH_RVA, 0xFFFF_FFFFu32);
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `draw_screen_quad`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::race::frame_draw::draw_screen_quad(r, &mut crate::race::frame_draw::quad_part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { loading_bar_quad(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, 0
    let entry: &[u8] = &[0xB8, 0x00, 0x00, 0x00, 0x00];
    let copy = match copy(0x446861, 0x9A, entry, &[0x94], &[(0x44B364, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (crate::framework::game_addr(QUAD_IN_RVA) as u32, 0x40),
        at(SWITCH_RVA),
    ];
    let mut vars = regions.clone();
    vars.push(at(SHARE_RVA));
    let saved = capture(&vars);
    for share in [0u32, 37, 100, 150, 0xFFFF_FFF6] {
        run_both(
            &mut t,
            &format!("share {share}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SHARE_RVA, share);
            },
            &|r| unsafe { loading_bar_quad(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
