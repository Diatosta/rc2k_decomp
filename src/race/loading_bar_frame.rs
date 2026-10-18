//! `race::loading_bar_frame` (0x4467C7): one frame of the loading bar while a stage loads
//! (the loading's callback `dword_623704`), with `dword_71C3E0` only: `dword_6365CC` 8;
//! nothing while quitting; with the window active and the surfaces not lost
//! (`dword_518380` not 1): `scene::begin_scene`, the view window chosen as unframed
//! (`dword_621514` 0 for `scene::choose_view_window`, put back), `race::loading_bar_quad`,
//! `scene::end_scene`, `scene::lock_frame`, an empty function, `scene::unlock_frame`,
//! `scene::present_frame`, carry clear. A failed step (or the window inactive, or the
//! surfaces lost) instead, unless quitting, restores the surfaces (`sub_405B10`, the
//! platform's) and leaves carry set when they were neither lost before nor after.
//!
//! In:  nothing
//! Out: CF / ZF as the path leaves them (see the code); the registers as its calls leave
//!      them
//!
//! Caller: the stage's loading (through `dword_623704`). The window is its Rust version;
//! the rest through `part` (1 begin, 2 the bar, 3 end, 4 lock, 5 unlock, 6 show, 7 the
//! restore), in the game their Rust versions (the restore the platform's call). Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose calls go to stand-ins
//! setting carry by the case, against the Rust version, every way out.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::loading_bar_frame", rva(0x4467C7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ON_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const MARK_RVA: usize = rva(0x6365CC); // dword_6365CC
const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const ACTIVE_RVA: usize = rva(0x518374); // g_windowActive
const LOST_RVA: usize = rva(0x518380); // dword_518380
const FRAMED_RVA: usize = rva(0x621514); // dword_621514
const RESTORE_RVA: usize = rva(0x405B10); // sub_405B10

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn flags(g: &mut Regs, carry: bool, zero: bool) {
    g.set_carry(carry);
    g.set_zero(zero);
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn loading_bar_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(ON_RVA) == 0 {
            flags(&mut g, false, true);
            return Ok(g);
        }
        game_write(MARK_RVA, 8u32);
        if game_read::<u32>(QUITTING_RVA) == 1 {
            flags(&mut g, false, true);
            return Ok(g);
        }
        let drawn = 'frame: {
            if game_read::<u32>(ACTIVE_RVA) == 0 || game_read::<u32>(LOST_RVA) == 1 {
                break 'frame false;
            }
            part(1, &mut g)?;
            if g.carry() {
                break 'frame false;
            }
            let framed: u32 = game_read(FRAMED_RVA);
            game_write(FRAMED_RVA, 0u32);
            crate::scene::choose_view_window::choose_view_window().apply(&mut g);
            game_write(FRAMED_RVA, framed);
            part(2, &mut g)?;
            part(3, &mut g)?;
            part(4, &mut g)?;
            if g.carry() {
                break 'frame false;
            }
            part(5, &mut g)?;
            part(6, &mut g)?;
            !g.carry()
        };
        if drawn {
            g.set_carry(false);
            return Ok(g);
        }
        if game_read::<u32>(QUITTING_RVA) == 1 {
            flags(&mut g, false, true);
            return Ok(g);
        }
        g.ebx = game_read(LOST_RVA);
        part(7, &mut g)?;
        g.ebx = (g.ebx ^ 1) | game_read::<u32>(LOST_RVA);
        let ebx = g.ebx;
        flags(&mut g, ebx < 1, ebx == 1);
    }
    Ok(g)
}

/// In the game: the calls' Rust versions (the restore the platform's).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::*;
    unsafe {
        match place {
            1 => begin_scene::begin_scene(r, &mut |r| call_game(r, game_addr(rva(0x41ABC7)))),
            2 => {
                *r = crate::race::loading_bar_quad::loading_bar_quad(
                    r,
                    &mut crate::race::loading_bar_quad::game_part,
                )?
            }
            3 => end_scene::end_scene(r, &mut |r| call_game(r, game_addr(rva(0x41AB8A)))),
            4 => *r = lock_frame::lock_frame(r, &mut lock_frame::game_part)?,
            5 => *r = unlock_frame::unlock_frame(r, &mut unlock_frame::game_part)?,
            6 => *r = present_frame::present_frame(r, &mut present_frame::game_part)?,
            _ => call_game(r, game_addr(RESTORE_RVA)),
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
    match unsafe { loading_bar_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The places whose stand-ins set carry in this case (a bit each); the restore's sets
/// `dword_518380` to bit 8.
static CARRY: AtomicU32 = AtomicU32::new(0);

fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    let c = CARRY.load(Ordering::SeqCst);
    r.set_carry(c & (1 << place) != 0);
    if place == 7 {
        unsafe { game_write(LOST_RVA, (c >> 8) & 1) };
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::take_called;
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_71C3E0, 0
    let entry: &[u8] = &[0x83, 0x3D, 0xE0, 0xC3, 0x71, 0x00, 0x00];
    let copy = match copy(
        0x4467C7,
        0x9A,
        entry,
        &[0x32, 0x49, 0x54, 0x59, 0x5E, 0x65, 0x6A, 0x6F, 0x87],
        &[
            (0x44E8A1, 1),
            (0x446861, 2),
            (0x44E952, 3),
            (0x44E837, 4),
            (0x44E851, 5),
            (0x44E895, 6),
            (0x405B10, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let mut regions: Vec<Region> = [MARK_RVA, FRAMED_RVA, LOST_RVA].map(at).to_vec();
    regions.extend(crate::scene::choose_view_window::regions());
    let mut vars = regions.clone();
    vars.extend([ON_RVA, QUITTING_RVA, ACTIVE_RVA].map(at));
    vars.extend(crate::scene::set_view_window::inputs());
    let saved = capture(&vars);
    // (on, quitting, active, lost, carry bits)
    let cases: [(u32, u32, u32, u32, u32); 9] = [
        (0, 0, 1, 0, 0),
        (1, 1, 1, 0, 0),
        (1, 0, 1, 0, 0),
        (1, 0, 0, 0, 0),
        (1, 0, 1, 1, 0x100),
        (1, 0, 1, 0, 1 << 1),
        (1, 0, 1, 0, 1 << 4),
        (1, 0, 1, 0, (1 << 6) | 0x100),
        (1, 0, 0, 1, 0),
    ];
    let regs = |r: &Regs| {
        (
            [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
            r.carry(),
            r.eflags & 0x40 != 0,
        )
    };
    for (n, (on, quitting, active, lost, carry)) in cases.into_iter().enumerate() {
        // Drawn: the zero flag is the one `scene::set_view_window`'s `mul` leaves
        // (undefined on x86; no caller reads it): only the carry is compared there.
        let drawn = on == 1 && quitting == 0 && active == 1 && lost == 0 && carry == 0;
        let regs = |r: &Regs| {
            let (all, c, z) = regs(r);
            (all, c, z && !drawn)
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(ON_RVA, on);
            game_write(QUITTING_RVA, quitting);
            game_write(ACTIVE_RVA, active);
            game_write(LOST_RVA, lost);
            game_write(FRAMED_RVA, 1u32);
            CARRY.store(carry, Ordering::SeqCst);
        };
        prepare();
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            loading_bar_frame(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {n}: original {:x?} {:x?}; rust {:x?} {:x?}; same state {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2 == mine.2
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
