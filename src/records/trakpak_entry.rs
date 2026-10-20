//! `records::trakpak_entry` (0x473027): returns the entry of the loaded track pack that a
//! record refers to.
//!
//! In:  edi = record (dword +4 = entry index, dword +8 = track pack id)
//! Out: edi = esi = entry pointer, ebx = index * 0x40, ecx = the record's pack id,
//!      edx = the loaded pack's id; eax and ebp preserved
//! The entries are 0x40 bytes each and start 0x34 bytes into the pack header. If the record's
//! pack id is not the loaded pack's, the original reports "Invalid TrakPak Header" or, when the
//! header's first dword is non-zero, loops forever (it reloads the header address on every
//! pass instead of following the link): that case is left to the original.

use crate::errors::FatalError;
use crate::framework::{
    EAX, EBP, RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("records::trakpak_entry", rva(0x473027), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const HEADER_RVA: usize = rva(0x734BC6); // g_trakPakHeader: +0 link, +4 pack id, +0x34 entries

const ENTRY_SIZE: u32 = rc2k_core::files::records::PACK_ENTRY_LEN as u32;

pub fn loaded_pack_id() -> u32 {
    unsafe { game_read(HEADER_RVA + 4) }
}

// ---- Rust version ----

/// What the original reports when a record belongs to another track pack.
pub const INVALID_HEADER: FatalError = FatalError {
    code: 0x8070_DA00,
    message: "Invalid TrakPak Header",
};

/// Address of entry `index` in the pack header at address `header`.
/// The original loops forever instead of reporting the error when the header's first dword
/// is non-zero; this version reports it in both cases.
pub fn trakpak_entry(
    header: u32,
    index: u32,
    pack_id: u32,
    loaded_pack_id: u32,
) -> Result<u32, FatalError> {
    // Converted: `rc2k_core::files::records::pack_entry`.
    rc2k_core::files::records::pack_entry(index, pack_id, loaded_pack_id)
        .map(|place| header.wrapping_add(place as u32))
        .ok_or(INVALID_HEADER)
}

/// The entry for the record at `record`, in the pack the game has loaded.
pub unsafe fn record_entry(record: *const u8) -> Result<u32, FatalError> {
    let (index, pack_id) = unsafe {
        (
            record.add(4).cast::<u32>().read_unaligned(),
            record.add(8).cast::<u32>().read_unaligned(),
        )
    };
    trakpak_entry(
        game_addr(HEADER_RVA) as u32,
        index,
        pack_id,
        loaded_pack_id(),
    )
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let record = r.edi as *const u8;
    let (index, pack_id) = unsafe {
        (
            record.add(4).cast::<u32>().read_unaligned(),
            record.add(8).cast::<u32>().read_unaligned(),
        )
    };
    let loaded = loaded_pack_id();

    let mine = match trakpak_entry(game_addr(HEADER_RVA) as u32, index, pack_id, loaded) {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };

    if !shadow() {
        r.edi = mine;
        r.esi = mine;
        r.ebx = index.wrapping_mul(ENTRY_SIZE);
        r.ecx = pack_id;
        r.edx = loaded;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.edi != mine
        || o.esi != mine
        || o.ebx != index.wrapping_mul(ENTRY_SIZE)
        || o.ecx != pack_id
        || o.edx != loaded
    {
        HOOK.mismatch(|| {
            format!(
                "index={index} pack_id={pack_id:#x}: original edi={:#x} esi={:#x} ebx={:#x} ecx={:#x} edx={:#x} rust={mine:#x}",
                o.edi, o.esi, o.ebx, o.ecx, o.edx
            )
        });
    }
    (r.edi, r.esi, r.ebx, r.ecx, r.edx) = (o.edi, o.esi, o.ebx, o.ecx, o.edx);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Always the loaded pack's own id, so the original never takes its error path. The result
    // is only computed, never read through, so any index is safe.
    let loaded = loaded_pack_id();
    let header = game_addr(HEADER_RVA) as u32;
    let indices = (0..=300u32).chain([
        0x0100_0000,
        0x03FF_FFFF,
        0x0400_0000,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFF,
    ]);

    for index in indices {
        let mut record = [0xAAu8; 12];
        record[4..8].copy_from_slice(&index.to_le_bytes());
        record[8..12].copy_from_slice(&loaded.to_le_bytes());
        let before = Regs {
            edi: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };

        let mine = trakpak_entry(header, index, loaded, loaded).unwrap_or(0);
        let preserved = ro.preserved_except(&before, !(EAX | EBP));
        let ok = ro.edi == mine
            && ro.esi == mine
            && ro.ebx == index.wrapping_mul(ENTRY_SIZE)
            && ro.ecx == loaded
            && ro.edx == loaded
            && preserved;
        t.check(ok, || {
            format!(
                "index={index:#x}: original edi={:#x} esi={:#x} ebx={:#x} ecx={:#x} edx={:#x} rust={mine:#x} preserved={preserved}",
                ro.edi, ro.esi, ro.ebx, ro.ecx, ro.edx
            )
        });
    }
    t.finish()
}
