//! `menu::check_cheat_name` (0x480CBF): the fourth player's name taken as a cheat's: the
//! name coded by rot13 into the text buffer (`strings::rot13_into_text`) is looked for in
//! the list at `aNepnqrNpgvba` (0x7585E0: each entry a coded name, its NUL and the cheat's
//! bit number; a NUL where a name would start ends it), each by `strings::same_text`
//! (case aside). Found: the bit set in `g_cheats`, the menu's sound 8, `menu::apply_cheats`,
//! and the player's name made string 0x18F.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `menu::player4_details_loop` once the name's editing ends. The sound and
//! `apply_cheats` go through `part` (in the game the Rust versions; `apply_cheats` logs);
//! shadow mode runs the original alone. Refuses (before anything) a missing fourth player
//! or name, a list without its end in 64 entries, a bit past 31 (the original's `bts`
//! would reach past `g_cheats`), and what the coding refuses. Self-test: a copy whose two
//! calls go to stand-ins, against the Rust version: names of the list (coded back),
//! another, case changed.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use crate::menu::player::{PLAYERS_RVA, player_at};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::rot13_into_text::{rot13, rot13_into_text};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::check_cheat_name", rva(0x480CBF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIST_RVA: usize = rva(0x7585E0); // aNepnqrNpgvba
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const HIDDEN_NAME: u32 = 0x18F;
const CALLEES: [usize; 2] = [0x40F059, 0x480AC1]; // sound::play_menu_sound, menu::apply_cheats

/// The list's entry and the cheat's bit for the coded name in the text buffer, if it is in
/// the list.
unsafe fn find(coded: *const u8) -> Result<Option<(u32, u32)>, Refused> {
    let list = game_addr(LIST_RVA) as *const u8;
    let coded = unsafe { std::ffi::CStr::from_ptr(coded.cast()) }.to_bytes();
    // Converted: `rc2k_core::menu::cheat_names::find`.
    let found = rc2k_core::menu::cheat_names::find(
        &unsafe { crate::bridge::Raw::new(list) },
        coded,
        &crate::strings::same_text::case_table(),
    )?;
    Ok(found.map(|f| (list as u32 + f.at as u32, f.bit as u32)))
}

/// The Rust version: the registers it leaves.
pub unsafe fn check_cheat_name(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let player = player_at(3);
    if player == 0 {
        return Err(BadInput("no fourth player").into());
    }
    let name = unsafe { ((player + 0x18) as *const u32).read_unaligned() };
    unsafe {
        let coded = rot13_into_text(name as *const u8)?;
        let Some((found, bit)) = find(coded as *const u8)? else {
            return Ok(*entry);
        };
        // Converted: `rc2k_core::menu::cheat_names::with_cheat`.
        let cheats = rc2k_core::menu::cheat_names::with_cheat(game_read(CHEATS_RVA), bit as u8)?;
        game_write(CHEATS_RVA, cheats);
        // The registers at the calls: ebx the entry, ecx the bit, edi the record.
        let mut r = Regs {
            eax: 8,
            ebx: found,
            ecx: bit,
            edi: player,
            ..*entry
        };
        part(0, &mut r)?;
        part(1, &mut r)?;
        let mut hidden = Vec::new();
        crate::strings::append_string_by_id::push_string(&mut hidden, HIDDEN_NAME);
        hidden.push(0);
        std::ptr::copy_nonoverlapping(hidden.as_ptr(), name as *mut u8, hidden.len());
    }
    Ok(*entry)
}

/// In the game: the menu sound's and `apply_cheats`' Rust versions (its log lines too).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 0 {
        crate::sound::play_menu_sound::play_menu_sound_in_game(r);
    } else {
        unsafe {
            crate::menu::apply_cheats::apply_cheats(&mut |line| {
                crate::menu::text_entry::log_l1(line, true)
            })
        }?;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { check_cheat_name(r, part) },
        game_part,
    );
}

/// The list's names, decoded.
fn listed_names() -> Vec<Vec<u8>> {
    let mut names = Vec::new();
    let mut at = game_addr(LIST_RVA) as *const u8;
    unsafe {
        for _ in 0..rc2k_core::menu::cheat_names::MOST_ENTRIES {
            if at.read() == 0 {
                break;
            }
            let mut name = Vec::new();
            while at.read() != 0 {
                name.push(rot13(at.read()));
                at = at.add(1);
            }
            names.push(name);
            at = at.add(2);
        }
    }
    names
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x480CBF,
        0x76,
        &[0x60, 0x50, 0xB8, 0x03, 0x00, 0x00, 0x00],
        &[0x12, 0x1D, 0x24, 0x34, 0x4A, 0x4F, 0x6A, 0x6F],
        &CALLEES,
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    if crate::strings::string_table::blob().is_null() {
        return t.skip("string table not loaded");
    }
    let mut names = listed_names();
    if names.is_empty() {
        return t.skip("no cheat names");
    }
    names.truncate(3);
    let mut upper = names[0].clone();
    upper.iter_mut().for_each(|b| *b = b.to_ascii_uppercase());
    names.push(upper);
    names.push(b"Colin McRae".to_vec());
    let mut record = [0u32; 0x40];
    let mut name = [0u8; 0x40];
    let (player, at_name) = (record.as_mut_ptr() as u32, name.as_mut_ptr() as u32);
    let inputs = [at(PLAYERS_RVA + 12)];
    let regions: [Region; 3] = [at(CHEATS_RVA), (at_name, 0x40), (text() as u32, TEXT_LEN)];
    let saved = (capture(&inputs), capture(&[regions[0], regions[2]]));
    for (n, typed) in names.iter().enumerate() {
        for cheats in [0u32, 0x8000_0001] {
            run_both(
                &mut t,
                &format!(
                    "name {n} ({}), cheats {cheats:#x}",
                    String::from_utf8_lossy(typed)
                ),
                &copy,
                &regions,
                &|| unsafe {
                    std::ptr::write_bytes(at_name as *mut u8, 0x5A, 0x40);
                    std::ptr::copy_nonoverlapping(typed.as_ptr(), at_name as *mut u8, typed.len());
                    (at_name as *mut u8).add(typed.len()).write(0);
                    ((player + 0x18) as *mut u32).write_unaligned(at_name);
                    game_write(PLAYERS_RVA + 12, player);
                    game_write(CHEATS_RVA, cheats);
                    std::ptr::write_bytes(text(), 0xAA, TEXT_LEN);
                },
                Regs::SENTINEL,
                false,
                &|r| unsafe { check_cheat_name(r, &mut test_part) },
            );
        }
    }
    restore(&inputs, &saved.0);
    restore(&[regions[0], regions[2]], &saved.1);
    let _ = (&mut record, &mut name);
    t.finish()
}
