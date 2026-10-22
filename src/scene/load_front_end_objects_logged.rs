//! `scene::load_front_end_objects_logged` (0x4433DA): `scene::load_front_end_objects`,
//! then its log: a rule, "Front end objects loaded", "total points" `g_objectPointCount`,
//! "total pointmem" `dword_634FA4`, "total faces" `dword_5CD67C`, "total facemem"
//! `dword_634FA8` (each label without a line's end, its value in hex8 with one), a rule.
//!
//! In:  nothing
//! Out: the registers as `load_front_end_objects` leaves them (the log calls are
//!      `pusha`'d)
//!
//! Caller: `scene::set_up_front_end_scene`. Its calls through `part`: 0 the loading (in
//! the game at its address: its hook chooses), 1 `log_write_l1_nl`, 2 `log_write_l1`, 3
//! `log_hex8_l1_nl` (eax the text or value; in the game the Rust log functions). Shadow
//! mode runs the original alone (it loads files and logs). Manual patch with a private
//! copy (the first call starts at the first byte; its 12 calls relocated). Self-test: a
//! copy whose calls go to stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::load_front_end_objects_logged",
    rva(0x4433DA),
    thunk,
    &ORIGINAL,
)
.private_copy(LEN)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x91;
const CALLS: [usize; 12] = [
    0x0, 0xC, 0x19, 0x26, 0x32, 0x3F, 0x4B, 0x58, 0x64, 0x71, 0x7D, 0x8A,
];
const CALLEES: [usize; 4] = [
    0x425CA9, // 0 scene::load_front_end_objects
    0x40142B, // 1 logfile::log_write_l1_nl
    0x401427, // 2 logfile::log_write_l1
    0x401626, // 3 logfile::log_hex8_l1_nl
];
const RULE: usize = 0x4DBC2F;
const LOADED: usize = 0x4DBC4B;
const RULE_2: usize = 0x4DBC9A;
/// (label, value)
const TOTALS: [(usize, usize); 4] = [
    (0x4DBC64, 0x5CD680), // "total points", g_objectPointCount
    (0x4DBC71, 0x634FA4), // "total pointmem"
    (0x4DBC80, 0x5CD67C), // "total faces"
    (0x4DBC8C, 0x634FA8), // "total facemem"
];

/// The Rust version: the registers it leaves.
pub unsafe fn load_front_end_objects_logged(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(0, &mut r)?;
    let mut line = |place: u32, eax: u32| part(place, &mut Regs { eax, ..r });
    let text = |a: usize| game_addr(rva(a)) as u32;
    line(1, text(RULE))?;
    line(1, text(LOADED))?;
    for (label, value) in TOTALS {
        line(2, text(label))?;
        line(3, unsafe { game_read(rva(value)) })?;
    }
    line(1, text(RULE_2))?;
    Ok(r)
}

/// In the game: the loading at its address, the log lines by the Rust log functions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_hex8::log_hex8, log_write::log_write};
    unsafe {
        match place {
            0 => crate::calls::call(rva(CALLEES[0]), r),
            1 => log_write(r.eax as *const u8, true, 1),
            2 => log_write(r.eax as *const u8, false, 1),
            _ => log_hex8(r.eax, true, 1),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { load_front_end_objects_logged(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4433DA,
        LEN,
        &[0xE8, 0xCA, 0x28, 0xFE, 0xFF],
        &CALLS,
        &CALLEES,
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::menu::parts::answers(&[(0, &[crate::menu::parts::Answer::eax(0x1234)])]);
    run_both(
        &mut t,
        "once",
        &copy,
        &[],
        &|| {},
        Regs::SENTINEL,
        false,
        &|r| unsafe { load_front_end_objects_logged(r, &mut test_part) },
    );
    crate::menu::parts::answers(&[]);
    t.finish()
}
