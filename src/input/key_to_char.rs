//! `input::key_to_char` (0x407E43): turns a key (a scan code) into the character it types
//! with the current keyboard layout, shift state and pending dead key.
//!
//! In:  al = key
//! Out: al = the character, 0 if the key types none; carry = (al == 0), zero = (al == 1);
//!      ebx, ecx, edx, esi, edi, ebp preserved. The rest of eax: untouched for a key release
//!      (bit 7 set); else zero, except ah = the dead key that was stored or used.
//! The layout is a block of four pointers (`dword_4F07D4` points at the current one): +0 a
//! list of dead keys {key, plain, shifted} ended by a 0 key, +4 the plain table, +8 the
//! shifted one, +0xC the one used while `byte_4F6B40` is down (read as AltGr). A dead key is
//! stored in `byte_4F6AD0`; the next key is then looked up, by its plain character, in the
//! list of {dead key, base, plain result, shifted result} at 0x4F8000. Keys whose bit is set
//! in the bitmap at 0x4F7FF0 type nothing. In layout mode 2 (`byte_4F07D2`), Alt or AltGr
//! with a shift key switches the layout: left shift picks 0x4F07F8, right shift 0x4F080C.
//! Refuses: no layout; a dead key pending with no plain table (the original reads through
//! null); lists without an end within 256 entries.

use crate::errors::BadInput;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::key_to_char", rva(0x407E43), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const LAYOUT_MODE_RVA: usize = rva(0x4F07D2); // byte_4F07D2
pub const LAYOUT_RVA: usize = rva(0x4F07D4); // dword_4F07D4: the current layout block
pub const LAYOUT_A_RVA: usize = rva(0x4F07F8); // dword_4F07F8
const LAYOUT_B_RVA: usize = rva(0x4F080C); // unk_4F080C
pub const DEAD_KEY_RVA: usize = rva(0x4F6AD0); // byte_4F6AD0
// The key-down table starts at the pending key, 0x4F6ADC, and is indexed by scan code.
pub const LEFT_SHIFT_RVA: usize = rva(0x4F6B06); // byte_4F6B06 (key 0x2A)
pub const RIGHT_SHIFT_RVA: usize = rva(0x4F6B12); // byte_4F6B12 (key 0x36)
pub const ALT_RVA: usize = rva(0x4F6B14); // byte_4F6B14 (key 0x38)
pub const ALT_GR_RVA: usize = rva(0x4F6B40); // byte_4F6B40
const SILENT_KEYS_RVA: usize = rva(0x4F7FF0); // dword_4F7FF0: one bit per key
const COMPOSE_RVA: usize = rva(0x4F8000); // unk_4F8000

use crate::bridge::Raw;

unsafe fn dword(addr: u32) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}

fn down(rva: usize) -> bool {
    unsafe { game_read::<u8>(rva) != 0 }
}

/// True if a keyboard layout is set.
pub fn has_layout() -> bool {
    unsafe { game_read::<u32>(LAYOUT_RVA) != 0 }
}

/// What the function may write.
pub fn regions() -> [Region; 2] {
    [at(LAYOUT_RVA), (game_addr(DEAD_KEY_RVA) as u32, 1)]
}

// ---- Rust version ----

/// eax as the original leaves it; the character is its low byte.
pub unsafe fn key_to_char(eax: u32) -> Result<u32, BadInput> {
    use rc2k_core::keyboard::{self, Layout, Switch};
    let key = eax as u8;
    if key & 0x80 != 0 {
        return Ok(eax & !0xFF);
    }
    // Converted: `rc2k_core::keyboard`; here the layout block's pointers as tables, and
    // the switch and the pending key stored.
    unsafe {
        let switched = keyboard::switch(
            game_read(LAYOUT_MODE_RVA),
            down(ALT_RVA),
            down(ALT_GR_RVA),
            down(LEFT_SHIFT_RVA),
            down(RIGHT_SHIFT_RVA),
        );
        let layout: u32 = match switched {
            Some(Switch::First) => game_addr(LAYOUT_A_RVA) as u32,
            Some(Switch::Second) => game_addr(LAYOUT_B_RVA) as u32,
            None => game_read(LAYOUT_RVA),
        };
        if layout == 0 {
            return Err(BadInput("no keyboard layout"));
        }
        let table = |at: u32| {
            let p = dword(layout + at);
            (p != 0).then(|| Raw::new(p as *const u8))
        };
        let tables = [table(0), table(4), table(8), table(0xC)];
        fn as_bytes(t: &Option<Raw>) -> Option<&dyn rc2k_core::bytes::Bytes> {
            t.as_ref().map(|t| t as &dyn rc2k_core::bytes::Bytes)
        }
        let typed = keyboard::key_to_char(
            key,
            &Layout {
                dead_keys: as_bytes(&tables[0]),
                plain: as_bytes(&tables[1]),
                shifted: as_bytes(&tables[2]),
                alt_gr: as_bytes(&tables[3]),
            },
            &Raw::new(game_addr(COMPOSE_RVA) as *const u8),
            &Raw::new(game_addr(SILENT_KEYS_RVA) as *const u8),
            down(LEFT_SHIFT_RVA) || down(RIGHT_SHIFT_RVA),
            down(ALT_GR_RVA),
            game_read(DEAD_KEY_RVA),
        )?;
        if switched.is_some() {
            game_write(LAYOUT_RVA, layout);
        }
        if let Some(pending) = typed.pending {
            game_write(DEAD_KEY_RVA, pending);
        }
        Ok(typed.eax)
    }
}

// ---- hook ----

fn set_result(r: &mut Regs, eax: u32) {
    r.eax = eax;
    r.set_carry(eax as u8 == 0);
    r.set_zero(eax as u8 == 1);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        match unsafe { key_to_char(r.eax) } {
            Ok(eax) => set_result(r, eax),
            Err(_) => unsafe { call_original(r) },
        }
        return;
    }

    // Shadow: the Rust version first; what it wrote is put back before the original runs.
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { key_to_char(r.eax) };
    let state = capture(&regions);
    restore(&regions, &before);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if let Ok(eax) = mine {
        let theirs = capture(&regions);
        if o.eax != eax || theirs != state || !o.preserved_except(r, EAX) {
            HOOK.mismatch(|| {
                format!(
                    "eax={:#x} state {before:x?}: original eax={:#x} {theirs:x?}; rust eax={eax:#x} {state:x?}",
                    r.eax, o.eax
                )
            });
        }
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test: every key with and without shift, AltGr and a pending dead key ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let layout: u32 = unsafe { game_read(LAYOUT_RVA) };
    if layout == 0 || unsafe { dword(layout + 4) } == 0 {
        return t.skip("no keyboard layout is loaded");
    }
    let keys = [
        (game_addr(LEFT_SHIFT_RVA) as u32, 1),
        (game_addr(RIGHT_SHIFT_RVA) as u32, 1),
        (game_addr(ALT_RVA) as u32, 1),
        (game_addr(ALT_GR_RVA) as u32, 1),
    ];
    let saved = (capture(&regions()), capture(&keys));

    // Dead keys to leave pending: the first few different ones of the compose list.
    let mut pendings = vec![0u8];
    let mut entry = game_addr(COMPOSE_RVA) as u32;
    for _ in 0..256 {
        let dead = unsafe { (entry as *const u8).read() };
        if dead == 0 || pendings.len() >= 4 {
            break;
        }
        if !pendings.contains(&dead) {
            pendings.push(dead);
        }
        entry += 4;
    }
    // In layout mode 2 a held AltGr with shift switches to a layout that may be empty, where
    // a pending dead key makes the original read through null: AltGr stays up there.
    let alt_grs: &[u8] = if unsafe { game_read::<u8>(LAYOUT_MODE_RVA) } == 2 {
        &[0]
    } else {
        &[0, 1]
    };

    for &pending in &pendings {
        for shift in [0u8, 1] {
            for &alt_gr in alt_grs {
                for key in 0u32..=255 {
                    let prepare = || unsafe {
                        game_write(LAYOUT_RVA, layout);
                        game_write(DEAD_KEY_RVA, pending);
                        game_write(LEFT_SHIFT_RVA, shift);
                        game_write(RIGHT_SHIFT_RVA, 0u8);
                        game_write(ALT_RVA, 0u8);
                        game_write(ALT_GR_RVA, alt_gr);
                    };
                    let before = Regs {
                        eax: 0x5EED_AB00 | key,
                        ..Regs::SENTINEL
                    };
                    prepare();
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    let theirs = capture(&regions());
                    let (carry_o, zero_o) = (ro.carry(), ro.eflags & 0x40 != 0);

                    prepare();
                    let mine = unsafe { key_to_char(before.eax) };
                    let state = capture(&regions());

                    let ok = mine == Ok(ro.eax)
                        && theirs == state
                        && carry_o == (ro.eax as u8 == 0)
                        && zero_o == (ro.eax as u8 == 1)
                        && ro.preserved_except(&before, EAX);
                    t.check(ok, || {
                        format!(
                            "key={key:#x} shift={shift} alt_gr={alt_gr} pending={pending:#x}: original eax={:#x} carry={carry_o} zero={zero_o} {theirs:x?}; rust {mine:x?} {state:x?}",
                            ro.eax
                        )
                    });
                }
            }
        }
    }

    restore(&regions(), &saved.0);
    restore(&keys, &saved.1);
    t.finish()
}
