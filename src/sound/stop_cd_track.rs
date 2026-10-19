//! `sound::stop_cd_track` (0x40FCC8): with the CD audio open (`dword_518108` 1),
//! `mciSendCommandA(mciId, MCI_STOP, 0, 0)`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! Called by `sound::stop_cd_audio` (three calls and two more by its `call $+5` into this).
//! The MCI call is the platform's: in the game through the import's thunk (0x4D5E7E); in
//! the standalone program nothing (no CD audio; in this build `dword_518108` never becomes
//! 1: `sound::open_cd_audio`). Shadow mode runs the original alone (the device is stopped
//! once). Self-test: a copy whose two variables are the test's and whose MCI call goes to a
//! stand-in that notes its arguments, against the Rust version, open and not.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow, standalone,
};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stop_cd_track", rva(0x40FCC8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CD_OPEN_RVA: usize = rva(0x518108); // dword_518108: the CD audio is open
const MCI_ID_RVA: usize = rva(0x518118); // mciId
const MCI_SEND_COMMAND: usize = rva(0x4D5E7E); // mciSendCommandA's thunk
const MCI_STOP: u32 = 0x808;

/// `mciSendCommandA(device, message, parameter 1, parameter 2)`.
pub type SendCommand<'a> = &'a mut dyn FnMut(u32, u32, u32, u32) -> u32;

#[derive(Clone, Copy)]
pub struct CdVars {
    pub open: u32,
    pub device: u32,
}

impl CdVars {
    pub fn game() -> Self {
        Self {
            open: game_addr(CD_OPEN_RVA) as u32,
            device: game_addr(MCI_ID_RVA) as u32,
        }
    }
}

// ---- Rust version ----

pub unsafe fn stop_cd_track(v: &CdVars, send: SendCommand) {
    unsafe {
        if (v.open as *const u32).read_unaligned() == 1 {
            send((v.device as *const u32).read_unaligned(), MCI_STOP, 0, 0);
        }
    }
}

/// In the game: the import's thunk (winmm); in the standalone program nothing.
pub fn game_send(device: u32, message: u32, a: u32, b: u32) -> u32 {
    if standalone() {
        return 0;
    }
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32) -> u32 =
            std::mem::transmute(game_addr(MCI_SEND_COMMAND));
        f(device, message, a, b)
    }
}

/// For callers with parts: the game's variables and MCI.
pub fn game_part(_place: u32, _r: &mut Regs) -> Result<(), Refused> {
    unsafe { stop_cd_track(&CdVars::game(), &mut game_send) };
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { stop_cd_track(&CdVars::game(), &mut game_send) };
}

// ---- self-test ----

static NOTED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

extern "system" fn note_send(device: u32, message: u32, a: u32, b: u32) -> u32 {
    NOTED.lock().unwrap().push([device, message, a, b]);
    0
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = vec![0u32; 2];
    let at = vars.as_mut_ptr() as u32;
    let v = CdVars {
        open: at,
        device: at + 4,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40FCC8,
            len: 0x20,
            entry: &[0x60, 0x83, 0x3D, 0x08, 0x81, 0x51, 0x00, 0x01],
            calls: &[0x19],
            jumps_out: &[],
        },
        &|callee| (callee == MCI_SEND_COMMAND).then_some(note_send as *const () as usize),
        &[(0x518108, v.open), (0x518118, v.device)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for open in [0u32, 1, 2] {
        let run = |rust: bool| unsafe {
            let p = at as *mut u32;
            p.write_volatile(open);
            p.add(1).write_volatile(0x0042_0017);
            NOTED.lock().unwrap().clear();
            let preserved = if rust {
                stop_cd_track(&v, &mut |d, m, a, b| note_send(d, m, a, b));
                true
            } else {
                let before = Regs::SENTINEL;
                let mut ro = before;
                call_game(&mut ro, c.addr());
                ro.preserved_except(&before, 0)
            };
            (preserved, std::mem::take(&mut *NOTED.lock().unwrap()))
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs == mine, || {
            format!("open {open}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(vars);
    t.finish()
}
