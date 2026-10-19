//! `net::begin_setting_edit` (0x416C26): Enter on a setting of the settings screen starts
//! its editing (`net::begin_edit`, width 0x30C, `palette`): item 1 the player's name
//! (`g_machineName`), item 3 the host's address (`unk_52ABE4`), item 2 the most players as
//! a text (`strings::format_u32` of `dword_52ABD0` into `byte_95E8F8`); any other nothing.
//!
//! In:  nothing (the item is `g_listChoice`)
//! Out: eax = the item, ebx = the text, ecx = 0x30C, edx = `palette` when an edit starts;
//!      for other items ebx = `dword_52ABD0`; no `pusha`
//! Caller: `net::settings_loop`. Its calls through `part` (`strings::append_nul` inline).
//! Shadow mode runs the original alone. Self-test: a copy whose calls go to stand-ins,
//! against the Rust version, on items 0 .. 4.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    CHOICE, FORMAT_U32, HOST_ADDRESS, MACHINE_NAME, MOST_PLAYERS, NUMBER_TEXT, PALETTE, Part,
    Saved, a, answer, dispatch_lobby, dword, g, p, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::begin_setting_edit", rva(0x416C26), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BEGIN_EDIT: u32 = 0x416E1A; // net::begin_edit

/// The Rust version: the registers it leaves.
pub fn begin_setting_edit(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CHOICE);
    r.ebx = match r.eax {
        1 => a(MACHINE_NAME),
        3 => a(HOST_ADDRESS),
        _ => g(MOST_PLAYERS),
    };
    match r.eax {
        1 | 3 => {}
        2 => {
            let item = r.eax;
            r.eax = r.ebx;
            r.ebx = a(NUMBER_TEXT);
            r.ecx = 0;
            part(FORMAT_U32, &mut r)?;
            unsafe { (r.ebx as *mut u8).write(0) };
            r.ebx = a(NUMBER_TEXT);
            r.eax = item;
        }
        _ => return Ok(r),
    }
    r.ecx = 0x30C;
    r.edx = a(PALETTE);
    part(BEGIN_EDIT, &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, begin_setting_edit);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, g_listChoice
    let copy = match crate::net::lobby::copy(
        0x416C26,
        0x50,
        &[0xA1, 0x07, 0xD4, 0x51, 0x00],
        &[0x33, 0x38, 0x4A],
        &[FORMAT_U32, BEGIN_EDIT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(CHOICE),
        dword(MOST_PLAYERS),
        region(NUMBER_TEXT, 0x10),
    ];
    let saved = Saved::new(&all);
    for item in 0..5u32 {
        run_both(
            &mut t,
            &format!("item {item}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                p(CHOICE, item);
                p(MOST_PLAYERS, 6);
                crate::net::lobby::fill(NUMBER_TEXT, 0x10, 5);
                answer(crate::net::draw_setting_value::string_answers());
            },
            &begin_setting_edit,
        );
    }
    drop(saved);
    t.finish()
}
