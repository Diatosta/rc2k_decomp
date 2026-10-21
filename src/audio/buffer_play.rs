//! What `audio::is_playing`, `rewind_buffer`, `play_buffer`, `stop_buffer` and
//! `commit_3d` share: a voice's buffer started, stopped and asked after, one COM call each
//! (`audio/com.rs`), and the self-test of the four on a voice's buffer.

use crate::audio::buffer_settings::a3d;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, shadow};
use crate::screens::harness::CallOriginal;

pub const GET_STATUS: u32 = 0x24;
pub const PLAY: u32 = 0x30;
pub const SET_CURRENT_POSITION: u32 = 0x34;
pub const STOP: u32 = 0x48;
pub const COMMIT: u32 = 0x44;
pub const LISTENER_3D_RVA: usize = crate::framework::rva(0x506A90); // g_soundListener
/// The zero flag of a `Regs` (`framework` has its setter only).
pub fn zero(r: &Regs) -> bool {
    r.eflags & 0x40 != 0
}

/// `GetStatus`: the buffer plays.
pub const PLAYING: u32 = 1;

fn buffer(v: &Voice, a3d_name: &'static str) -> Result<u32, Refused> {
    if a3d() {
        return Err(BadInput(a3d_name).into());
    }
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `GetStatus` (+0x24) into a dword: the status.
pub unsafe fn is_playing(v: &Voice) -> Result<u32, Refused> {
    let b = buffer(v, "A3D (`sub_40CE81`)")?;
    let mut status = 0u32;
    unsafe {
        let f: unsafe extern "system" fn(u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, GET_STATUS));
        f(b, &mut status as *mut u32 as u32);
    }
    Ok(status)
}

/// `SetCurrentPosition` (+0x34) with 0.
pub unsafe fn rewind_buffer(v: &Voice) -> Result<i32, Refused> {
    let b = buffer(v, "A3D (`sub_40CE96`)")?;
    Ok(unsafe { crate::audio::com::call1(b, SET_CURRENT_POSITION, 0) })
}

/// `Play` (+0x30) with 0, 0 and the looping flag for a looping voice (kind 1). Failing: the
/// carry, and `DSERR_BUFFERLOST` sets the flag at `lost`. Returns eax and the carry.
pub unsafe fn play_buffer(v: &Voice, lost: u32) -> Result<(i32, bool), Refused> {
    let b = buffer(v, "A3D (`sub_40CEA0`)")?;
    // Converted: `rc2k_core::audio::buffers::{play_flags, lost}`.
    let flags = rc2k_core::audio::buffers::play_flags(v);
    let r = unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, PLAY));
        f(b, 0, 0, flags)
    };
    if r != 0 && rc2k_core::audio::buffers::lost(r) {
        unsafe { write(lost, 1) };
    }
    Ok((r, r != 0))
}

pub unsafe fn play_buffer_in_game(v: &Voice) -> Result<(i32, bool), Refused> {
    unsafe { play_buffer(v, game_addr(BUFFER_LOST_RVA) as u32) }
}

/// `Stop` (+0x48).
pub unsafe fn stop_buffer(v: &Voice) -> Result<i32, Refused> {
    let b = buffer(v, "A3D (`sub_40CEBB`)")?;
    Ok(unsafe { crate::audio::com::call(b, STOP) })
}

/// The 3D listener's `CommitDeferredSettings` (+0x44); `listener`: where it is kept.
pub unsafe fn commit_3d(listener: u32) -> Result<i32, Refused> {
    let object = unsafe { read(listener) };
    if object == 0 {
        return Err(BadInput("no 3D listener").into());
    }
    Ok(unsafe { crate::audio::com::call(object, COMMIT) })
}

/// What a leaf leaves: eax, and the carry or the zero flag where callers read one.
pub struct Leaves {
    pub eax: u32,
    pub carry: Option<bool>,
    pub zero: Option<bool>,
}

/// The whole `dispatch` of a leaf: in shadow mode the original alone; else the Rust
/// version (ecx and edx are left as they were: the COM method leaves them as it likes,
/// and no caller reads them).
pub fn dispatch_leaf(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: unsafe fn(&Regs) -> Result<Leaves, Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { rust(r) } {
        Ok(left) => {
            r.eax = left.eax;
            if let Some(c) = left.carry {
                r.set_carry(c);
            }
            if let Some(z) = left.zero {
                r.set_zero(z);
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// The self-test of a leaf on a voice: the original and the Rust version on voices of the
/// test's whose buffer is the test DirectSound's, under each of `results`; the calls
/// noted, eax and the flags compared. `g_soundBufferLost` is put back afterwards.
pub fn check_leaf(
    t: &mut SelfTest,
    call_original: CallOriginal,
    rust: unsafe fn(&Regs) -> Result<Leaves, Refused>,
    kinds: &[u32],
    results: &[crate::audio::test_sound::Results],
) {
    if a3d() {
        t.check(true, String::new);
        return;
    }
    let lost = [crate::service::harness::at(BUFFER_LOST_RVA)];
    let saved = crate::service::harness::capture(&lost);
    let sound = crate::audio::test_sound::TestSound::new();
    for &kind in kinds {
        for &res in results {
            let v = Voice {
                buffer: sound.buffer(1),
                kind,
                ..Voice::default()
            };
            let before = Regs {
                esi: &v as *const Voice as u32,
                ..Regs::SENTINEL
            };
            unsafe { write(game_addr(BUFFER_LOST_RVA) as u32, 0) };
            sound.begin(res);
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let theirs = (sound.noted(), unsafe {
                read(game_addr(BUFFER_LOST_RVA) as u32)
            });
            unsafe { write(game_addr(BUFFER_LOST_RVA) as u32, 0) };
            sound.begin(res);
            let mine = unsafe { rust(&before) };
            let noted = (sound.noted(), unsafe {
                read(game_addr(BUFFER_LOST_RVA) as u32)
            });
            let ok = match &mine {
                Ok(left) => {
                    left.eax == ro.eax
                        && left.carry.is_none_or(|c| c == ro.carry())
                        && left.zero.is_none_or(|z| z == zero(&ro))
                }
                Err(_) => false,
            };
            t.check(ok && theirs == noted, || {
                format!(
                    "kind {kind}, status {:#x}, play {:#x}: original eax {:#x} carry {} zero {} {theirs:x?}; rust {:?} {noted:x?}",
                    res.status,
                    res.play,
                    ro.eax,
                    ro.carry(),
                    zero(&ro),
                    mine.as_ref().map(|l| (l.eax, l.carry, l.zero))
                )
            });
        }
    }
    crate::service::harness::restore(&lost, &saved);
}
