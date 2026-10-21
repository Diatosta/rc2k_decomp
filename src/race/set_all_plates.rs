//! `race::set_all_plates` (0x42518B): the texture kinds' pictures for the stage
//! (`LOADLEVEL_INIT : setplatesall`), then the system and slot textures reloaded.
//!
//! In:  nothing
//! Out: the registers as `scene::reload_slot_textures` leaves them
//!
//! - The stage prefs' (`localdat`) picture lists copied to `g_kindPictures` (0x5D7968), 0x32C14
//!   bytes below: 0x60A57C .. 0x60A64C, 0x60A67C .. 0x60A73C, 0x60A77C .. 0x60A79C (the
//!   original in 4-dword `rep movsd` pieces; esi, edi and ecx left as its last).
//! - 23 kinds' pictures fixed (`FIXED_PICTURES`).
//! - With cars, each car's eight from 0x5D7D68: its kind's picture
//!   (`scene::known_kind_picture` of `dword_623858`[car]) and the seven after it. Leaves edi
//!   past them, ecx the cars, eax the last + 1, ebx 0.
//! - `scene::reload_system_textures`, `scene::reload_slot_textures`.
//!
//! Callers: `sub_44426F` (loadlevel_init) and the code at 0x425B29. Refuses more than 8 cars.
//! Shadow mode runs the original alone (it loads textures and logs). Self-test: a
//! redirected copy whose two reloads note themselves, the pictures and cars of the
//! test's, the game's put back.

use crate::errors::Refused;
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
const PICTURES_LEN: usize = 4 * rc2k_core::race::stage_objects::PICTURE_KINDS;
/// The prefs' picture lists (`PREF_PIECES`' kinds) start here, 0x32C14 bytes above the
/// pictures.
const PREF_LISTS: usize = 0x60A57C;
const PREFS_PART: (usize, usize) = (0x60A57C, 0x60A79C - 0x60A57C);
const CARS_RVA: usize = rva(0x63DB90); // g_raceCars
const CAR_KINDS_RVA: usize = rva(0x623858); // g_raceCarKinds
const MOST_CARS: u32 = rc2k_core::race::stage_objects::CAR_BLOCKS;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `race::set_all_plates` (see the header); part 0 the system textures' reload, 1 the
/// slots'.
pub unsafe fn set_all_plates(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::stage_objects::{check_picture_cars, PREF_PIECES,
    // FIXED_PICTURES, car_pictures}`.
    use rc2k_core::race::stage_objects::{
        CAR_PICTURE_KINDS, FIXED_PICTURES, PICTURES_A_CAR, PREF_PIECES, car_pictures,
        check_picture_cars,
    };
    let mut g = *entry;
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        check_picture_cars(cars)?;
        let pictures = game_addr(PICTURES_RVA);
        for (kind, dwords) in PREF_PIECES {
            let from = game_addr(rva(PREF_LISTS)) + 4 * kind;
            let to = pictures + 4 * kind;
            std::ptr::copy(from as *const u32, to as *mut u32, dwords);
            g.esi = (from + 4 * dwords) as u32;
            g.edi = (to + 4 * dwords) as u32;
        }
        g.ecx = 0;
        for (kind, picture) in FIXED_PICTURES {
            game_write(PICTURES_RVA + 4 * kind, picture);
        }
        if cars != 0 {
            g.edi = (pictures + 4 * CAR_PICTURE_KINDS) as u32;
            for car in 0..cars {
                g.ecx = car;
                let first = crate::scene::known_kind_picture::known_kind_picture(game_read(
                    CAR_KINDS_RVA + 4 * car as usize,
                ));
                for picture in car_pictures(first) {
                    (g.edi as *mut u32).write_unaligned(picture);
                    g.edi += 4;
                }
                g.eax = first.wrapping_add(PICTURES_A_CAR as u32);
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
