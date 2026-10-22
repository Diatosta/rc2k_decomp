//! `system::find_running_game` (0x4103BE): `hInstance` (0x5184A0) = `GetModuleHandleA(0)`,
//! then whether a window of the game's class ("MF-SDG R5", 0x4E0080) is open already.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`); carry set (and zero clear) if such a
//!      window was found, else carry and zero as `test eax, eax` leaves them (clear, set)
//! One caller, `start` (0x40107E), which leaves at once on the carry. In the game this
//! never gets a call (the hook DLL is loaded later). The Win32 calls are made from Rust
//! (the same in both programs; in the standalone program its own module's handle, and a
//! running game is found as in the game: its window is not of that class). Manual patch
//! with a private copy (0x20 bytes, both calls relocated): a call starts at +3. Shadow
//! mode: both (they only read), compared. Self-test: the original and the Rust version,
//! `hInstance` and both flags (in the running game the window is found).

use crate::framework::{
    RawHook, Regs, SelfTest, ZERO_FLAG, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::find_running_game", rva(0x4103BE), thunk, &ORIGINAL)
        .private_copy(0x20)
        .relocate_calls(&[0x03, 0x14])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const INSTANCE_RVA: usize = rva(0x5184A0); // hInstance
const CLASS_NAME_RVA: usize = rva(0x4E0080); // ClassName, "MF-SDG R5"

// ---- Rust version ----

/// True if the game runs already.
pub unsafe fn find_running_game() -> bool {
    unsafe {
        game_write(INSTANCE_RVA, GetModuleHandleA(std::ptr::null()) as u32);
        !FindWindowA(game_addr(CLASS_NAME_RVA) as *const u8, std::ptr::null()).is_null()
    }
}

// ---- hook ----

fn set_flags(r: &mut Regs, found: bool) {
    r.set_carry(found);
    r.set_zero(!found);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let found = unsafe { find_running_game() };
    if !shadow() {
        set_flags(r, found);
        return;
    }
    let mine = unsafe { game_read::<u32>(INSTANCE_RVA) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = unsafe { game_read::<u32>(INSTANCE_RVA) };
    if o.carry() != found || mine != theirs {
        HOOK.mismatch(|| {
            format!(
                "original found={} instance={theirs:#x}; rust found={found} instance={mine:#x}",
                o.carry()
            )
        });
    }
    r.eflags = o.eflags;
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let found = unsafe { find_running_game() };
    set_flags(r, found);
    Ok(())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    unsafe {
        let saved = game_read::<u32>(INSTANCE_RVA);
        game_write(INSTANCE_RVA, 0xAAAA_AAAAu32);
        let before = Regs::SENTINEL;
        let mut ro = before;
        call_original(&mut ro);
        let theirs = game_read::<u32>(INSTANCE_RVA);
        game_write(INSTANCE_RVA, 0xAAAA_AAAAu32);
        let mut mine = before;
        set_flags(&mut mine, find_running_game());
        let instance = game_read::<u32>(INSTANCE_RVA);
        let zero = |r: &Regs| r.eflags & ZERO_FLAG != 0;
        let preserved = ro.preserved_except(&before, 0);
        t.check(
            preserved && theirs == instance && ro.carry() == mine.carry() && zero(&ro) == zero(&mine),
            || {
                format!(
                    "original preserved={preserved} instance={theirs:#x} carry={} zero={}; rust instance={instance:#x} carry={} zero={}",
                    ro.carry(),
                    zero(&ro),
                    mine.carry(),
                    zero(&mine)
                )
            },
        );
        game_write(INSTANCE_RVA, saved);
    }
    t.finish()
}
