//! `race::set_all_plates` (0x42518B): the texture kinds' pictures for the stage
//! (`LOADLEVEL_INIT : setplatesall`), then the system and slot textures reloaded.
//!
//! In:  nothing
//! Out: the registers as `scene::reload_slot_textures` leaves them
//!
//! - The stage prefs' (`localdat`) picture lists copied to `g_kindPictures` (0x5D7968), 0x32C14
//!   bytes below: 0x60A57C .. 0x60A64C, 0x60A67C .. 0x60A73C, 0x60A77C .. 0x60A79C (the
//!   original in 4-dword `rep movsd` pieces; esi, edi and ecx left as its last).
//! - 23 kinds' pictures fixed (`FIXED`).
//! - With cars, each car's eight from 0x5D7D68: its kind's picture
//!   (`scene::known_kind_picture` of `dword_623858`[car]) and the seven after it. Leaves edi
//!   past them, ecx the cars, eax the last + 1, ebx 0.
//! - `scene::reload_system_textures`, `scene::reload_slot_textures`.
//!
//! Callers: `sub_44426F` (loadlevel_init) and the code at 0x425B29. Refuses more than 8 cars.
//! Shadow mode runs the original alone (it loads textures and logs). Self-test: a
//! redirected copy whose two reloads note themselves, the pictures and cars of the
//! test's, the game's put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_all_plates", rva(0x42518B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PICTURES_RVA: usize = rva(0x5D7968); // g_kindPictures
const PICTURES_LEN: usize = 0x5D82C8 - 0x5D7968;
const BELOW: usize = 0x32C14;
/// The pieces copied: their start in the prefs and their dwords.
const PIECES: [(usize, u32); 3] = [(0x60A57C, 0x34), (0x60A67C, 0x30), (0x60A77C, 8)];
const PREFS_PART: (usize, usize) = (0x60A57C, 0x60A79C - 0x60A57C);
const FIXED: [(usize, u32); 23] = [
    (0x5D7C74, 0x7A8),
    (0x5D7C88, 0x7CF),
    (0x5D7CB4, 0x708),
    (0x5D7CB8, 0x71A),
    (0x5D7C90, 0x384),
    (0x5D7C78, 0x6FE),
    (0x5D7C7C, 0x6FF),
    (0x5D7C8C, 0x700),
    (0x5D7CB0, 0x722),
    (0x5D7CD8, 0x70C),
    (0x5D7CDC, 0x70E),
    (0x5D7CE0, 0x710),
    (0x5D7CE4, 0x712),
    (0x5D7CE8, 0x714),
    (0x5D7CEC, 0x716),
    (0x5D81D8, 0x71E),
    (0x5D81DC, 0x720),
    (0x5D7C68, 0x708),
    (0x5D7C6C, 0x70A),
    (0x5D7C70, 0x706),
    (0x5D82C4, 0x71C),
    (0x5D7C80, 0x73A),
    (0x5D7C84, 0x73C),
];
const CARS_RVA: usize = rva(0x63DB90); // g_raceCars
const CAR_KINDS_RVA: usize = rva(0x623858); // g_raceCarKinds
const CAR_PICTURES: u32 = 0x5D7D68;
const MOST_CARS: u32 = 8;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `race::set_all_plates` (see the header); part 0 the system textures' reload, 1 the
/// slots'.
pub unsafe fn set_all_plates(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        if cars > MOST_CARS {
            return Err(BadInput("more than 8 cars").into());
        }
        for (from, dwords) in PIECES {
            let from = game_addr(rva(from));
            std::ptr::copy(
                from as *const u32,
                (from - BELOW) as *mut u32,
                dwords as usize,
            );
            g.esi = (from + 4 * dwords as usize) as u32;
            g.edi = g.esi - BELOW as u32;
        }
        g.ecx = 0;
        for (at, value) in FIXED {
            game_write(rva(at), value);
        }
        if cars != 0 {
            g.edi = game_addr(rva(CAR_PICTURES as usize)) as u32;
            for car in 0..cars {
                g.ecx = car;
                g.eax = crate::scene::known_kind_picture::known_kind_picture(game_read(
                    CAR_KINDS_RVA + 4 * car as usize,
                ));
                for _ in 0..8 {
                    (g.edi as *mut u32).write_unaligned(g.eax);
                    g.edi += 4;
                    g.eax = g.eax.wrapping_add(1);
                }
                g.ebx = 0;
            }
            g.ecx = cars;
        }
        part(0, &mut g)?;
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: the two reloads in Rust.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if place == 0 {
            crate::scene::reload_system_textures::reload_system_textures_in_game(r)
        } else {
            crate::scene::reload_slot_textures::reload_slot_textures_in_game(r)
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_all_plates(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    use crate::service::harness::{Region, at, capture, restore};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x42518B,
        0x329,
        &[0x8D, 0x3D, 0x68, 0x79, 0x5D, 0x00],
        &[0x302, 0x31E, 0x323],
        &[(0x4254B4, 0), (0x425669, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let pictures: Region = (game_addr(PICTURES_RVA) as u32, PICTURES_LEN);
    let prefs: Region = (game_addr(rva(PREFS_PART.0)) as u32, PREFS_PART.1);
    let kinds: Region = (game_addr(CAR_KINDS_RVA) as u32, 4 * MOST_CARS as usize);
    let all = [pictures, prefs, kinds, at(CARS_RVA)];
    let saved = capture(&all);
    for cars in [0u32, 1, 8] {
        run_both(
            &mut t,
            &format!("{cars} cars"),
            &copy,
            &[pictures],
            &|| unsafe {
                restore(&all, &saved);
                for n in 0..PREFS_PART.1 / 4 {
                    game_write(rva(PREFS_PART.0) + 4 * n, 0x0700_0000 + n as u32 * 3);
                }
                for n in 0..MOST_CARS {
                    game_write(CAR_KINDS_RVA + 4 * n as usize, (n * 5 + 2) as i32 - 3);
                }
                game_write(CARS_RVA, cars);
            },
            &|r| unsafe { set_all_plates(r, &mut test_part) },
        );
    }
    restore(&all, &saved);
    t.finish()
}
