//! `install::list_drives` (0x413C27): the drives that can be installed to found and their
//! menu built. Each letter from `dword_51BAE4` ('c') to 'z' that is not a CD-ROM (5) or a
//! RAM disk (6) and whose free space can be read (`files::drive_free_space_kb`) is recorded
//! at `dword_51BA10[letter-'a']` (count 1, free KB). Then for each recorded letter a text is
//! built in `byte_52A30F` (0x10 bytes each; 'a' to 'y': drive Z is found but never listed,
//! the game's slip): the letter, a space, the free space as KB
//! (id 0xA6), MB (0xA7) or GB (0xA8, with ">=" before it), NUL. The number is in
//! `dword_51BAE0`; the drive menu list is chosen (`off_51D7F8` up to 8 drives, `off_51D8B0`
//! up to 16, else `off_51D9E8`) into `off_51D5F8` with its item count set to the number
//! plus two.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. Its two probes through `part` (`files::drive_type`,
//! `files::drive_free_space_kb`). Shadow mode runs the original alone (the free space can
//! move between the two probes). Self-test: a copy whose probes go to stand-ins (the drives
//! scripted), against the Rust version; the tables and the chosen menu are compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    DRIVE_COUNT, DRIVE_LETTERS, DRIVE_TYPE, DRIVES, FIRST_DRIVE, FREE_SPACE, Part, a, at, both, g,
    p, quiet, region, wb, wd,
};
use crate::race::viewer::{Saved, dispatch_parts, regs_entry};
use rc2k_core::install::drives::{self, DriveMenu};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::list_drives", rva(0x413C27), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MENU_8: usize = 0x51D7F8; // off_51D7F8
const MENU_16: usize = 0x51D8B0; // off_51D8B0
const MENU_MORE: usize = 0x51D9E8; // off_51D9E8
const CHOSEN_MENU: usize = 0x51D5F8; // off_51D5F8

pub fn list_drives(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let drives = a(DRIVES);
    // Register for register: ecx the letter, eax given it at each probe, ebx the last slot.
    let mut r = *entry;
    r.ecx = g(FIRST_DRIVE);
    loop {
        r.eax = r.ecx;
        part(at(DRIVE_TYPE), &mut r)?;
        // Converted: `rc2k_core::install::drives` (`offered`, `LISTED`, `free`, `text`,
        // `menu`, `items`).
        if drives::offered(r.eax) {
            r.eax = r.ecx;
            part(at(FREE_SPACE), &mut r)?;
            if !r.carry() {
                r.ebx = drives + 8 * r.ecx.wrapping_sub(b'a' as u32);
                wd(r.ebx, g_at(r.ebx).wrapping_add(1));
                wd(r.ebx + 4, r.eax);
            }
        }
        r.ecx = r.ecx.wrapping_add(1);
        if r.ecx > b'z' as u32 {
            break;
        }
    }
    p(DRIVE_COUNT, 0);
    let mut ebx = a(DRIVE_LETTERS);
    // 'a' .. 'y': the loop's `cmp ecx, 7Ah; jb` leaves 'z' out (the game's slip, section 16)
    for letter in drives::LISTED {
        let slot = drives + 8 * (letter - b'a') as u32;
        if g_at(slot) == 0 {
            continue;
        }
        p(DRIVE_COUNT, g(DRIVE_COUNT) + 1);
        let row = ebx;
        wb(row, letter);
        let free = drives::free(g_at(slot + 4));
        let mut unit = Vec::new();
        unsafe {
            crate::strings::append_string_by_id::append_string_by_id(free.unit, &mut |from, n| {
                unit.extend_from_slice(std::slice::from_raw_parts(from, n));
            });
        }
        // the original writes the letter, skips a byte (`add ebx, 2`), then the rest
        for (n, &b) in drives::text(free, &unit).iter().enumerate() {
            wb(row + 2 + n as u32, b);
        }
        ebx = row + 0x10;
    }
    let count = g(DRIVE_COUNT);
    let menu = match drives::menu(count) {
        DriveMenu::Eight => MENU_8,
        DriveMenu::Sixteen => MENU_16,
        DriveMenu::More => MENU_MORE,
    };
    p(CHOSEN_MENU, a(menu));
    wb(a(menu) + 1, drives::items(count));
    Ok(*entry)
}

fn g_at(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        list_drives(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        list_drives(r, &mut crate::install::installer::game_part)
    })
}

// ---- self-test ----

static SCRIPT: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov ecx, dword_51BAE4
    let copy = match crate::install::installer::copy(
        0x413C27,
        0xED,
        &[0x60, 0x8B, 0x0D, 0xE4, 0xBA, 0x51, 0x00],
        &[
            (0x9, 0x4031D8),
            (0x1A, 0x40317D),
            (0x69, 0x401F29),
            (0xA2, 0x40204A),
            (0xA9, 0x401FA3),
            (0xAE, 0x401F16),
        ],
        &[0x4031D8, 0x40317D],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let regions = [
        region(DRIVES, 8 * 26),
        region(DRIVE_COUNT, 4),
        region(DRIVE_LETTERS, 0x10 * 26),
        region(CHOSEN_MENU, 4),
        region(MENU_8 + 1, 1),
        region(MENU_16 + 1, 1),
        region(MENU_MORE + 1, 1),
    ];
    let saved = Saved::new(&regions);
    // (kind by letter a..z, free KB by letter)
    let scenes: [&[(u8, u32, u32)]; 3] = [
        &[
            (b'c', 2, 0x200),
            (b'd', 5, 0),
            (b'e', 3, 0x80000),
            (b'f', 3, 0x20000000),
        ],
        &[(b'c', 3, 0)],
        &[(b'g', 6, 0), (b'h', 0, 0x1234)],
    ];
    for scene in scenes {
        let prepare = || {
            crate::install::installer::fill(a(DRIVES), 8 * 26, 0);
            crate::install::installer::fill(a(DRIVE_LETTERS), 0x10 * 26, 0xAA);
            p(FIRST_DRIVE, b'c' as u32);
            let mut s = Vec::new();
            for letter in b'c'..=b'z' {
                let found = scene.iter().find(|&&(l, _, _)| l == letter);
                let (kind, free) = found.map(|&(_, k, f)| (k, f)).unwrap_or((3, 0x100));
                // free fails when the scene lists it with free 0 and kind not 5/6
                s.push((kind, free));
            }
            *SCRIPT.lock().unwrap() = s;
            crate::install::installer::answer(Box::new(move |place, r| {
                let mut s = SCRIPT.lock().unwrap();
                if place == a(0x4031D8) {
                    // drive_type: by the letter in eax
                    let n = (r.eax as u8).wrapping_sub(b'c') as usize;
                    r.eax = s.get(n).map(|&(k, _)| k).unwrap_or(3);
                } else if place == a(0x40317D) {
                    let n = (r.eax as u8).wrapping_sub(b'c') as usize;
                    let free = s.get(n).map(|&(_, f)| f).unwrap_or(0x100);
                    r.eax = free;
                    r.set_carry(free == 0);
                }
                let _ = &mut s;
            }));
        };
        both(
            &mut t,
            &format!("scene {scene:?}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| list_drives(r, part),
        );
    }
    drop(saved);
    t.finish()
}
