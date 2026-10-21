//! `system::check_developer_machine` (0x416292): whether the computer is called "andrew".
//!
//! In:  nothing
//! Out: ebx = the text buffer; ax = the last two characters looked at, in lower case; the
//!      rest preserved
//! `dword_529D94` = 0; the computer's name is asked for into the text buffer
//! (`GetComputerNameA`, through `sub_410848`; if that fails the buffer is looked at as it
//! is); if its first six characters are "andrew" in any case, `dword_529D94` = 1.
//! (`race::tyre_change_gauge` reads it.) One caller, `start`.
//! Shadow mode asks twice. Self-test: a redirected copy whose name comes from the test.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use crate::strings::text_from_ids::text;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::WindowsProgramming::GetComputerNameA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::check_developer_machine",
    rva(0x416292),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DEVELOPER_RVA: usize = rva(0x529D94); // dword_529D94
const NAME_ROOM: u32 = 0x40;

// ---- Rust version ----

/// `sub_410848`: the computer's name at `to` (0x40 bytes), if it can be had.
pub unsafe fn computer_name(to: *mut u8) -> bool {
    let mut room = NAME_ROOM;
    unsafe { GetComputerNameA(to, &mut room) != 0 }
}

/// `name` writes the computer's name at the place it is given, if it can. Returns the
/// low word the original leaves in eax.
pub unsafe fn check_developer_machine(name: &mut dyn FnMut(*mut u8)) -> u16 {
    unsafe {
        game_write(DEVELOPER_RVA, 0u32);
        name(text());
        // Converted: `rc2k_core::system::machine::developer`.
        let (yes, last) =
            rc2k_core::system::machine::developer((text() as *const [u8; 6]).read_unaligned());
        if yes {
            game_write(DEVELOPER_RVA, 1u32);
        }
        last
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [at(DEVELOPER_RVA), (text() as u32, NAME_ROOM as usize)]
}

fn left(r: &Regs, last: u16) -> Left {
    Left::nothing()
        .with(EBX, text() as u32)
        .with(EAX, r.eax & 0xFFFF_0000 | last as u32)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let last = unsafe {
        check_developer_machine(&mut |to| {
            computer_name(to);
        })
    };
    Ok(left(r, last))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: names of the test's ----

const LEN: usize = 0x49;
const ENTRY_BYTES: [u8; 8] = [0xC7, 0x05, 0x94, 0x9D, 0x52, 0x00, 0x00, 0x00];
const CALLS: [usize; 1] = [0x10];

/// The name the stand-in gives, or none if the asking fails.
static NAME: Mutex<Option<&'static [u8]>> = Mutex::new(None);

fn give_name(_: u32, regs: &mut Regs) {
    if let Some(name) = *NAME.lock().unwrap() {
        unsafe { std::ptr::copy(name.as_ptr(), regs.ebx as *mut u8, name.len()) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x416292)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    let names: [Option<&'static [u8]>; 10] = [
        Some(b"ANDREW\0"),
        Some(b"andrew-pc\0"),
        Some(b"AnDrEw2\0"),
        Some(b"andre\0"),
        Some(b"andrea\0"),
        Some(b"xandrew\0"),
        Some(b"AN\x04REW\0"),
        Some(b"\0"),
        Some(b"DESKTOP-1\0"),
        None,
    ];
    for name in names {
        *NAME.lock().unwrap() = name;
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            // What a failed asking leaves to be looked at.
            std::ptr::copy(c"andrew".as_ptr().cast::<u8>(), text(), 7);
            text().add(NAME_ROOM as usize - 1).write(0);
            set_handler(give_name);
            let regs = if rust {
                let last = check_developer_machine(&mut |to| {
                    let mut regs = Regs {
                        ebx: to as u32,
                        ..Regs::default()
                    };
                    act(0, &mut regs);
                });
                let mut after = before;
                left(&before, last).apply(&mut after);
                (after.eax, after.ebx, true)
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                (ro.eax, ro.ebx, ro.preserved_except(&before, EAX | EBX))
            };
            (regs, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.2, || {
            format!(
                "name={:?}: original {:x?} developer={:?}; rust {:x?} developer={:?}",
                name.map(String::from_utf8_lossy),
                theirs.0,
                &theirs.1[..4],
                mine.0,
                &mine.1[..4]
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
