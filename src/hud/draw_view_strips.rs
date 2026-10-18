//! `hud::draw_view_strips` (0x45B7B1): unless `dword_5BEC94`, for each player's view block
//! (`g_viewBlocks`, `+4` for the second when `dword_6238A4` is not 0) whose +0xFA8 is 0 and
//! +0x728 is set: while its flash is on (+0x730 at most 0, or +0x72C modulo +0x730 below
//! half of +0x730), `hud::draw_view_strip` for view `dword_6238A4` (the second's plus 3)
//! with texture `dword_6240BC` 0xC4 (0xC5); then the strips' shared tail (the original
//! jumps to 0x45B9AA).
//!
//! In:  nothing
//! Out: as the path leaves them (esi the last block looked at)
//!
//! Caller: `sub_45A893` (0x45A92E). The strips and the tail through `part` (1 the strip,
//! 2 the tail), in the game their Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose strip calls and tail jump go to stand-ins, against the
//! Rust version, blocks by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::hud::draw_view_strip::TEXTURE_RVA;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_view_strips", rva(0x45B7B1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const OFF_RVA: usize = rva(0x5BEC94); // dword_5BEC94
const BLOCKS_RVA: usize = rva(0x63DBB0); // g_viewBlocks
const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Whether esi's block's strip is shown (see the header); edx / ebx as the test leaves them.
unsafe fn flashing(g: &mut Regs) -> bool {
    let b = g.esi;
    let get = |o: u32| unsafe { ((b + o) as *const u32).read_unaligned() };
    if get(0xFA8) != 0 || get(0x728) == 0 {
        return false;
    }
    let period = get(0x730);
    if (period as i32) <= 0 {
        return true;
    }
    g.eax = get(0x72C);
    g.edx = g.eax % period;
    g.eax /= period;
    g.ebx = period >> 1;
    g.edx < g.ebx
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_view_strips(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(OFF_RVA) == 0 {
            g.esi = game_read(BLOCKS_RVA);
            if flashing(&mut g) {
                g.ebp = game_read(PLAYERS_RVA);
                game_write(TEXTURE_RVA, 0xC4u32);
                part(1, &mut g)?;
            }
            g.ebp = game_read(PLAYERS_RVA);
            if g.ebp != 0 {
                g.esi = game_read(BLOCKS_RVA + 4);
                if flashing(&mut g) {
                    g.ebp = game_read::<u32>(PLAYERS_RVA).wrapping_add(3);
                    game_write(TEXTURE_RVA, 0xC5u32);
                    part(1, &mut g)?;
                }
            }
        }
        part(2, &mut g)?;
    }
    Ok(g)
}

/// In the game: the strip's and the tail's Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::draw_view_strip as vs;
    unsafe {
        match place {
            1 => *r = vs::draw_view_strip(r, &mut vs::game_part)?,
            _ => vs::overlay_tail(r, &mut (&mut vs::game_part as vs::Part))?,
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_view_strips(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_5BEC94, 0
    let entry: &[u8] = &[0x83, 0x3D, 0x94, 0xEC, 0x5B, 0x00, 0x00];
    let copy = match copy(
        0x45B7B1,
        0xBD,
        entry,
        &[0x57, 0xB3, 0xB8],
        &[(0x45B86E, 1), (0x45B9AA, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut blocks = vec![0u32; 2 * 0x400];
    let [b0, b1] = [
        blocks.as_mut_ptr() as u32,
        blocks.as_mut_ptr() as u32 + 0x1000,
    ];
    let regions = vec![at(TEXTURE_RVA)];
    let mut vars = regions.clone();
    vars.extend([OFF_RVA, BLOCKS_RVA, BLOCKS_RVA + 4, PLAYERS_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..8u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(OFF_RVA, u32::from(case == 7));
                game_write(BLOCKS_RVA, b0);
                game_write(BLOCKS_RVA + 4, b1);
                game_write(PLAYERS_RVA, case & 1);
                for (n, b) in [b0, b1].into_iter().enumerate() {
                    let put = |o: u32, v: u32| ((b + o) as *mut u32).write_unaligned(v);
                    let k = case + n as u32;
                    put(0xFA8, u32::from(k % 5 == 4));
                    put(0x728, u32::from(k % 4 != 3));
                    put(0x730, [0u32, 10, 7, 0xFFFF_FFF0][(k % 4) as usize]);
                    put(0x72C, 13 * k);
                }
            },
            &|r| unsafe { draw_view_strips(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(blocks);
    t.finish()
}
