//! `race::save_viewed_replay` (0x41ED1C): the viewed (and edited) replay saved as replay
//! ebx: with blocks to save, the header made ("REPL", 0, version 0x50400, the date
//! `system::packed_local_time`, the slot's name: 32 bytes of `byte_528934`); the last
//! race's file (`var\tmprpl`) opened to read (mode 0, `dword_975508`) and the slot's
//! (`race::replay_path`) made (mode 1, `dword_97550C`); the undo emptied
//! (`dword_9673F4` 0, its places zeroed: they become the new index); the header written,
//! then each block in the viewer's order: read from its place (`files::seek`,
//! `drive::read_packed` into `dword_9613A4`, neither checked), its first 8 bytes the
//! block's cameras, written packed (`race::write_replay_chunk`), its offset and size
//! (negative when packed) into the new index; then "RCIB" and the count (16 bytes) and the
//! new index (0x3840 bytes). Both closed; the undo emptied again.
//!
//! In:  ebx the slot (`race::replay_path`'s number)
//! Out: carry set when it failed (nothing to save, an open or a write failing); ZF when it
//!      worked; every register kept (`pusha` / `popa`)
//!
//! Callers: `race::viewer_action` (0x15 back into a saved slot, 0x16 into a chosen one),
//! `race::save_last_replay`. Its calls through `part`: 1 `system::packed_local_time`
//! (-> eax), 2 `files::open_by_mode` (eax path, ebx mode -> eax handle, carry), 3
//! `files::write` (eax, ebx, ecx -> eax, carry), 4 `files::seek` (eax, ecx distance, ebx
//! method), 5 `drive::read_packed` (eax, ebx, ecx), 6 `race::write_replay_chunk` (eax, ebx,
//! ecx -> eax, carry), 7 `files::close` (eax); in the game their Rust versions
//! (`replay_path` and `system::zero_memory` are called as they are). Saving into slot 222
//! would make the file it reads from anew (the original's way, kept). Shadow mode runs the
//! original alone (it writes a file). Self-test: a copy whose calls go to stand-ins,
//! against the Rust version: nothing to save, either open failing, writes failing at each
//! place, three blocks saved (packed and not).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::replay_path::{LAST_RACE_PATH, replay_path};
use crate::race::viewer::{
    BLOCK_DATA, BLOCKS, CAMERAS, CF, HEADER, INDEX, PLACES, Part, Saved, TABLE_LEN, UNDO_BLOCKS,
    UNDO_PLACES, ZF, answer, dispatch_parts, dword, get, move_dwords, put, region, run_both,
    test_part,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::save_viewed_replay", rva(0x41ED1C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const NAMES: usize = 0x528934; // byte_528934: 32 bytes a slot
const FROM: usize = 0x975508; // dword_975508
const TO: usize = 0x97550C; // dword_97550C
const OFFSET: usize = 0x5C2528; // dword_5C2528: the new file's length so far

/// The Rust version: the registers and flags it leaves.
pub fn save_viewed_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut out = *entry;
    let done = save(entry, part)?;
    put(UNDO_BLOCKS, 0);
    out.set_carry(!done);
    out.set_zero(done);
    Ok(out)
}

/// Whether it was saved (`edx` 1).
fn save(entry: &Regs, part: Part) -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{has_blocks, SAVED_HEADER, name_offset,
    // next_offset}`.
    use rc2k_core::race::viewer_flow::{
        BLOCK_LEN, CAMERA_DWORDS, HEADER_LEN, INDEX_HEAD_LEN, NAME_LEN, SAVED_HEADER, has_blocks,
        name_offset, next_offset,
    };
    let mut r = Regs { edx: 0, ..*entry };
    if !has_blocks(get(BLOCKS)) {
        return Ok(false);
    }
    for (k, word) in SAVED_HEADER.into_iter().enumerate() {
        put(HEADER + 4 * k, word);
    }
    part(1, &mut r)?;
    put(HEADER + 0xC, r.eax);
    let name = NAMES.wrapping_add(name_offset(r.ebx) as usize);
    unsafe {
        std::ptr::copy(a(name) as *const u8, a(HEADER + 0x10) as *mut u8, NAME_LEN);
    }
    r.ecx = 0;
    r.esi = a(name) + NAME_LEN as u32;
    r.edi = a(HEADER + 0x10) + NAME_LEN as u32;
    let slot = r.ebx;
    r.eax = a(LAST_RACE_PATH);
    r.ebx = 0;
    part(2, &mut r)?;
    r.ebx = slot;
    if r.carry() {
        return Ok(false);
    }
    put(FROM, r.eax);
    r = replay_path(&r)?;
    r.ebx = 1;
    part(2, &mut r)?;
    if r.carry() {
        close(part, &mut r, false)?;
        return Ok(false);
    }
    put(TO, r.eax);
    put(UNDO_BLOCKS, 0);
    unsafe {
        crate::system::zero_memory::zero_memory(a(UNDO_PLACES) as *mut u8, TABLE_LEN as u32)
    }?;
    let write = |part: Part, r: &mut Regs, buffer: u32, len: u32| -> Result<bool, Refused> {
        r.eax = get(TO);
        r.ebx = buffer;
        r.ecx = len;
        part(3, r)?;
        Ok(!r.carry())
    };
    put(OFFSET, HEADER_LEN);
    if !write(part, &mut r, a(HEADER), HEADER_LEN)? {
        close(part, &mut r, true)?;
        return Ok(false);
    }
    let mut block = 0u32;
    loop {
        r.ebp = block;
        r.eax = get(FROM);
        r.ebx = 0;
        r.ecx = get(PLACES + 8 * block as usize);
        part(4, &mut r)?;
        r.eax = get(FROM);
        r.ebx = a(BLOCK_DATA);
        r.ecx = get(PLACES + 8 * block as usize + 4);
        part(5, &mut r)?;
        move_dwords(BLOCK_DATA, CAMERAS + 8 * block as usize, CAMERA_DWORDS);
        r.ecx = 0;
        r.esi = a(CAMERAS) + 8 * block + 8;
        r.edi = a(BLOCK_DATA) + 8;
        put(UNDO_PLACES + 8 * block as usize, get(OFFSET));
        r.eax = get(TO);
        r.ebx = a(BLOCK_DATA);
        r.ecx = BLOCK_LEN;
        part(6, &mut r)?;
        if r.carry() {
            close(part, &mut r, true)?;
            return Ok(false);
        }
        put(UNDO_PLACES + 8 * block as usize + 4, r.eax);
        put(OFFSET, next_offset(get(OFFSET), r.eax));
        block += 1;
        if block == get(BLOCKS) {
            break;
        }
    }
    r.ebp = block;
    let done = write(part, &mut r, a(INDEX), INDEX_HEAD_LEN)?
        && write(part, &mut r, a(UNDO_PLACES), TABLE_LEN as u32)?;
    if done {
        r.edx = 1;
    }
    close(part, &mut r, true)?;
    Ok(done)
}

/// The two files closed (the new one only when it was made).
fn close(part: Part, r: &mut Regs, both: bool) -> Result<(), Refused> {
    if both {
        r.eax = get(TO);
        part(7, r)?;
    }
    r.eax = get(FROM);
    part(7, r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::files::handle;
    unsafe {
        match place {
            1 => r.eax = crate::system::packed_local_time::packed_local_time(),
            2 => crate::race::open_viewed_replay::game_part(2, r)?,
            3 => {
                let (n, ok) = handle::write(r.eax, r.ebx as *const u8, r.ecx);
                r.eax = n;
                r.set_carry(!ok);
            }
            4 => crate::race::open_viewed_replay::game_part(4, r)?,
            5 => {
                let (eax, carry, _) = crate::drive::read_packed::read_packed(r.eax, r.ebx, r.ecx)?;
                r.eax = eax;
                r.set_carry(carry);
            }
            6 => {
                *r =
                    crate::race::replay::write_replay_chunk(r, &mut crate::race::replay::game_part)?
            }
            _ => {
                let ok = handle::close(r.eax);
                r.eax = ok as u32;
                r.set_carry(!ok);
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        save_viewed_replay(r, &mut game_part)
    });
}

// ---- self-test ----

fn regions() -> Vec<Region> {
    vec![
        region(HEADER, 0x30),
        dword(FROM),
        dword(TO),
        dword(UNDO_BLOCKS),
        region(UNDO_PLACES, TABLE_LEN),
        dword(OFFSET),
        region(BLOCK_DATA, 0x10),
        region(0x94E8B0, 0x100),
    ]
}

/// Which call fails: (place, its how-many-th call), or none.
fn act(fails: (u32, u32), counts: &mut [u32; 8], place: u32, r: &mut Regs) {
    counts[place as usize] += 1;
    let fail = fails == (place, counts[place as usize]);
    match place {
        1 => r.eax = 0x2A3B_4C5D,
        2 => {
            r.eax = if fail { u32::MAX } else { 0x100 + counts[2] };
            r.set_carry(fail);
        }
        3 => {
            r.eax = if fail { r.ecx / 2 } else { r.ecx };
            r.set_carry(fail);
        }
        5 => unsafe {
            for n in 0..0x10u32 {
                ((r.ebx + n) as *mut u8).write(0x60 + n as u8 + counts[5] as u8);
            }
        },
        6 => {
            // Packed every other block.
            r.eax = if counts[6].is_multiple_of(2) {
                0x0123u32.wrapping_neg()
            } else {
                0x2810
            };
            r.set_carry(fail);
        }
        _ => {}
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; xor edx, edx; cmp dword_5BECD0, 0
    let copy = match crate::race::viewer::copy(
        0x41ED1C,
        0x199,
        &[0x60, 0x33, 0xD2, 0x83, 0x3D, 0xD0, 0xEC, 0x5B, 0x00, 0x00],
        &[
            0x2A, 0x56, 0x67, 0x71, 0x98, 0xB5, 0xD3, 0xEA, 0x11F, 0x157, 0x16E, 0x17B, 0x185,
        ],
        &[
            (0x410F99, 1),
            (0x402DE8, 2),
            (0x402E57, 3),
            (0x402E75, 4),
            (0x41EEB5, 5),
            (0x41EEE4, 6),
            (0x402E23, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = regions();
    all.extend([
        dword(BLOCKS),
        region(PLACES, 0x20),
        region(CAMERAS, 0x20),
        region(NAMES, 0x80),
    ]);
    let saved = Saved::new(&all);
    for (what, blocks, slot, fails) in [
        ("nothing to save", 0u32, 2u32, (0u32, 0u32)),
        ("three blocks", 3, 2, (0, 0)),
        ("a game's own slot", 3, 101, (0, 0)),
        ("the source not opened", 3, 2, (2, 1)),
        ("the slot not made", 3, 2, (2, 2)),
        ("the header not written", 3, 2, (3, 1)),
        ("a block not written", 3, 2, (6, 2)),
        ("the count not written", 3, 2, (3, 2)),
        ("the index not written", 3, 2, (3, 3)),
    ] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                ebx: slot,
                ..Regs::SENTINEL
            },
            &regions(),
            CF | ZF,
            &|| {
                for (at, len) in regions() {
                    unsafe { std::ptr::write_bytes(at as *mut u8, 0xAA, len) };
                }
                put(BLOCKS, blocks);
                crate::race::viewer::fill(PLACES, 0x20, 18);
                crate::race::viewer::fill(CAMERAS, 0x20, 19);
                crate::race::viewer::fill(NAMES, 0x80, 20);
                let mut counts = [0u32; 8];
                answer(Box::new(move |place, r| act(fails, &mut counts, place, r)));
            },
            &|r| save_viewed_replay(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
