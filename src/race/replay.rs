//! The replay file (`var\tmprpl`, IDA 2026-10-17): opened with its header at the race's
//! start (`race::open_replay`, the header sealed by `race::seal_replay_header`), frames
//! gathered (`race::add_replay_frame`), written block by block (`race::write_replay_block`
//! through `race::write_replay_chunk`, which packs by runs), and closed with its index
//! ("RCIB", `race::close_replay`). The steps call each other (a failed block closes the
//! file, which writes the last frames as a block), so all go through a `Part` by place
//! (`PLACES`): in the game the Rust versions (the file primitives `files::handle`'s).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::service::harness::{Region, at};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The calls by place, as IDA shows them.
pub const PLACES: [usize; 15] = [
    0,        // 0 (not used: the stand-ins' place 0 is special)
    0x402E57, // 1 files::write: eax handle, ebx buffer, ecx bytes -> eax written, CF failed
    0x402E23, // 2 files::close: eax handle
    0x402DE8, // 3 files::open_by_mode: eax path, ebx mode -> eax handle, CF failed
    0x403175, // 4 files::free_space_kb -> eax
    0x410F99, // 5 system::packed_local_time -> eax
    0x41EFEC, // 6 race::pack_runs
    0x41F29D, // 7 race::seal_stage_time
    0x402735, // 8 race::seal_block
    0x4017F2, // 9 strings::copy_printable
    0x41F0A3, // 10 race::seal_replay_header
    0x41E85A, // 11 race::close_replay
    0x41E8DE, // 12 race::add_replay_frame
    0x41E9A3, // 13 race::write_replay_block
    0x41EEE4, // 14 race::write_replay_chunk
];

pub const PLAIN_RVA: usize = rva(0x5BEC80); // dword_5BEC80: 1 = chunks written unpacked
pub const OPEN_RVA: usize = rva(0x5BEC88); // dword_5BEC88: the file is open
pub const DONE_RVA: usize = rva(0x5BEC84); // dword_5BEC84: a replay was closed
pub const HANDLE_RVA: usize = rva(0x5BEC8C); // h
pub const PACKED_RVA: usize = rva(0x93E8B0); // g_pngInfo: the packing's buffer
pub const INDEX_RVA: usize = rva(0x5BECC8); // "RCIB": the index, 0x3850 bytes
pub const INDEX_LEN: u32 = 0x3850;
pub const BLOCKS_RVA: usize = rva(0x5BECD0); // its block count
pub const FRAMES_RVA: usize = rva(0x5BECD4); // its frame count
pub const PLACES_RVA: usize = rva(0x5BECD8); // its (offset, size) per block
pub const MOST_BLOCKS: u32 = 0x708;
pub const OFFSET_RVA: usize = rva(0x5C2528); // dword_5C2528: the file's length so far
pub const IN_BLOCK_RVA: usize = rva(0x5C252C); // dword_5C252C: frames in the block
pub const LAST_BLOCK_RVA: usize = rva(0x5C2530); // dword_5C2530
pub const PAD_FRAME_RVA: usize = rva(0x5C2518); // unk_5C2518: the frame padding a block
pub const CURSOR_RVA: usize = rva(0x9613A0); // dword_9613A0: where the next frame goes
pub const BLOCK_RVA: usize = rva(0x9613A4); // dword_9613A4: the block, 0x2810 bytes
pub const BLOCK_LEN: u32 = 0x2810;
pub const HEADER_RVA: usize = rva(0x528504); // dword_528504: the file's header, 0x430
pub const HEADER_LEN: u32 = 0x430;
pub const CAR_HEADER_RVA: usize = rva(0x719B14); // dword_719B14: 0x400 bytes
pub const SEALED_RVA: usize = rva(0x719B54); // dword_719B54: the sealed part, 0x100
const PATH_RVA: usize = rva(0x5295F0); // "var\tmprpl"
const MOST_FREE_KB: u32 = 0x7800;

unsafe fn get(a: usize) -> u32 {
    unsafe { (game_addr(a) as *const u32).read_unaligned() }
}

unsafe fn set(a: usize, v: u32) {
    unsafe { (game_addr(a) as *mut u32).write_unaligned(v) };
}

/// `race::seal_replay_header` (0x41F0A3): the replay's sealed header part (`dword_719B54`,
/// 0x40 dwords): version byte 6, the player's name (`strings::copy_printable`, 0x20), the
/// settings and options copied (`HEADER_MOVES`), a checksum (from dword 1: add, `rol` 5)
/// into dword 0 and `dword_4F66F9`; then its 32 blocks of 8 chained through
/// `race::seal_block` with the key "halfmoon" (the game's key put back after). Every
/// register kept.
pub unsafe fn seal_replay_header(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    unsafe {
        game_write::<u8>(rva(0x719B58), 6);
        let mut r = Regs {
            ecx: 0x20,
            eax: game_addr(rva(0x755784)) as u32, // g_playerNames
            ebx: game_addr(rva(0x719B61)) as u32,
            ..g
        };
        part(9, &mut r)?;
        for &(size, to, from) in &HEADER_MOVES {
            let (to, from) = (game_addr(rva(to)), game_addr(rva(from)));
            std::ptr::copy(from as *const u8, to as *mut u8, size);
        }
        let mut sum = 0u32;
        for n in 1..0x40 {
            sum = sum.wrapping_add(get(SEALED_RVA + 4 * n)).rotate_left(5);
        }
        set(SEALED_RVA, sum);
        set(rva(0x4F66F9), sum);
        let key = [get(rva(0x4E052C)), get(rva(0x4E0530))];
        set(rva(0x4E052C), 0x666C_6168);
        set(rva(0x4E0530), 0x6E6F_6F6D);
        let (mut a, mut b) = (0u32, 0u32);
        for n in 0..0x20u32 {
            let at = SEALED_RVA + 8 * n as usize;
            set(rva(0x4E0534), get(at) ^ a);
            set(rva(0x4E0538), get(at + 4) ^ b);
            // The registers at the call: eax the second dword's mix, ebx the block, ecx
            // its number, esi and edi the chain.
            let mut r = Regs {
                eax: get(at + 4) ^ b,
                ebx: game_addr(at) as u32,
                ecx: n,
                esi: a,
                edi: b,
                ..g
            };
            part(8, &mut r)?;
            a = get(rva(0x4E053C));
            set(at, a);
            b = get(rva(0x4E0540));
            set(at + 4, b);
        }
        set(rva(0x4E0530), key[1]);
        set(rva(0x4E052C), key[0]);
    }
    Ok(g)
}

/// (bytes, to, from), in the original's order (made from the disassembly by script).
const HEADER_MOVES: [(usize, usize, usize); 31] = [
    (1, 0x719B81, 0x4E07A0),
    (4, 0x719B59, 0x528510),
    (1, 0x719B86, 0x75ACE8),
    (1, 0x719B87, 0x75ACD4),
    (1, 0x719B88, 0x75ACF0),
    (1, 0x719B89, 0x75ACD8),
    (1, 0x719B8A, 0x75AE1C),
    (1, 0x719C8C, 0x75AE1C),
    (1, 0x719B8B, 0x75AE24),
    (4, 0x719B5D, 0x5BECAC),
    (4, 0x719B82, 0x75870C),
    (1, 0x719B94, 0x75AE2C),
    (1, 0x719B95, 0x75AD30),
    (1, 0x719B96, 0x75AD1C),
    (1, 0x719B97, 0x75AD20),
    (1, 0x719B98, 0x75AD2C),
    (1, 0x719B99, 0x75AD24),
    (1, 0x719B9A, 0x75AD28),
    (1, 0x719B9B, 0x71C6D0),
    (1, 0x719B9C, 0x71C6D4),
    (1, 0x719B9D, 0x71C6D8),
    (2, 0x719B9E, 0x71C6DC),
    (4, 0x719BA0, 0x71C6E0),
    (2, 0x719BA4, 0x71C6E4),
    (2, 0x719BA6, 0x71C6E8),
    (2, 0x719BA8, 0x71C6EC),
    (2, 0x719BAA, 0x71C6F0),
    (2, 0x719BAC, 0x71C6F4),
    (2, 0x719BAE, 0x71C6F8),
    (2, 0x719BB0, 0x71C6FC),
    (2, 0x719BB2, 0x71C700),
];

/// `race::write_replay_chunk` (0x41EEE4): ecx bytes at ebx written to file eax; unless
/// `dword_5BEC80` is 1 they are packed (`race::pack_runs` into `g_pngInfo`, rounded up to
/// a dword) and the packing written when smaller (eax then the bytes written negated).
/// Leaves eax that, carry for a failed write (the rest kept: `pusha` / `popa`).
pub unsafe fn write_replay_chunk(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if get(PLAIN_RVA) == 1 {
            part(1, &mut g)?;
            return Ok(g);
        }
        let mut r = Regs {
            esi: g.ebx,
            edi: game_addr(PACKED_RVA) as u32,
            edx: g.ecx,
            ..g
        };
        part(6, &mut r)?;
        r.ecx = r.ecx.wrapping_add(3) & !3;
        if r.ecx >= r.edx {
            r.ecx = r.edx;
            part(1, &mut r)?;
        } else {
            r.ebx = r.edi;
            part(1, &mut r)?;
            if !r.carry() {
                r.eax = r.eax.wrapping_neg();
                r.set_carry(false);
                r.set_zero(r.eax == 0);
            }
        }
        let flags = r.eflags;
        g.eax = r.eax;
        g.eflags = flags;
    }
    Ok(g)
}

/// `race::write_replay_block` (0x41E9A3): the block (`dword_9613A4`, 0x2810 bytes) written
/// as a chunk; its offset and size (negative when packed) into the index; the file's
/// length on; the block count on. A failed write or the index full (0x708 blocks) closes
/// the replay (`race::close_replay`) and sets carry. Every register kept.
pub unsafe fn write_replay_block(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let n = get(BLOCKS_RVA);
        if n >= MOST_BLOCKS {
            return Err(BadInput("the replay's index is full already").into());
        }
        set(PLACES_RVA + 8 * n as usize, get(OFFSET_RVA));
        let mut r = Regs {
            edx: n,
            eax: get(HANDLE_RVA),
            ebx: game_addr(BLOCK_RVA) as u32,
            ecx: BLOCK_LEN,
            ..g
        };
        part(14, &mut r)?;
        let failed = if r.carry() {
            true
        } else {
            set(PLACES_RVA + 8 * n as usize + 4, r.eax);
            let size = if (r.eax as i32) < 0 {
                r.eax.wrapping_neg()
            } else {
                r.eax
            };
            r.eax = size;
            set(OFFSET_RVA, get(OFFSET_RVA).wrapping_add(size));
            set(BLOCKS_RVA, n + 1);
            if n + 1 == MOST_BLOCKS {
                true
            } else {
                g.set_carry(false);
                g.set_zero(size == 0);
                false
            }
        };
        if failed {
            part(11, &mut r)?;
            g.eflags = r.eflags;
            g.set_carry(true);
        }
    }
    Ok(g)
}

/// `race::add_replay_frame` (0x41E8DE): with the replay open, ecx bytes at ebx (a frame
/// when 0x10: the frame counts on) put at the cursor (`rep movsd`, ecx / 4 dwords); at 100
/// frames the block written (`race::write_replay_block`) and the cursor and count back.
/// Every register kept. Refuses frames past the block's room (before anything).
pub unsafe fn add_replay_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    unsafe {
        if get(OPEN_RVA) != 1 {
            return Ok(g);
        }
        let cursor = get(CURSOR_RVA);
        let end = game_addr(BLOCK_RVA) as u32 + BLOCK_LEN;
        let dwords = g.ecx >> 2;
        if cursor < game_addr(BLOCK_RVA) as u32 || cursor.wrapping_add(4 * dwords) > end {
            return Err(BadInput("a frame past the replay block's room").into());
        }
        if g.ecx == 0x10 {
            set(FRAMES_RVA, get(FRAMES_RVA).wrapping_add(1));
            set(IN_BLOCK_RVA, get(IN_BLOCK_RVA).wrapping_add(1));
        }
        std::ptr::copy(g.ebx as *const u8, cursor as *mut u8, 4 * dwords as usize);
        set(CURSOR_RVA, cursor + 4 * dwords);
        if get(IN_BLOCK_RVA) >= 100 {
            let mut r = Regs {
                esi: g.ebx + 4 * dwords,
                edi: cursor + 4 * dwords,
                ecx: 0,
                ..g
            };
            part(13, &mut r)?;
            set(CURSOR_RVA, game_addr(BLOCK_RVA) as u32);
            set(IN_BLOCK_RVA, 0);
        }
    }
    Ok(g)
}

/// `race::close_replay` (0x41E85A): with the replay open, the block filled up with the
/// padding frame (`race::add_replay_frame`, which writes it at 100); the last block's
/// frame count (`dword_5C252C`: 100, or for the last block the frames modulo 100 plus
/// one); the time sealed; the index ("RCIB", 0x3850 bytes) written; the file closed;
/// `dword_5BEC88` 0, `dword_5BEC84` 1. Every register kept.
pub unsafe fn close_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    unsafe {
        if get(OPEN_RVA) != 1 {
            return Ok(g);
        }
        let mut r = g;
        while get(IN_BLOCK_RVA) != 0 {
            r.ebx = game_addr(PAD_FRAME_RVA) as u32;
            r.ecx = 0x10;
            part(12, &mut r)?;
        }
        r.ebx = 100;
        r.eax = get(LAST_BLOCK_RVA).wrapping_add(1);
        if r.eax == get(BLOCKS_RVA) {
            r.eax = get(FRAMES_RVA);
            r.edx = r.eax % 100;
            r.eax /= 100;
            r.ebx = r.edx + 1;
        }
        set(IN_BLOCK_RVA, r.ebx);
        part(7, &mut r)?;
        r.eax = get(HANDLE_RVA);
        r.ebx = game_addr(INDEX_RVA) as u32;
        r.ecx = INDEX_LEN;
        part(1, &mut r)?;
        r.eax = get(HANDLE_RVA);
        part(2, &mut r)?;
        set(OPEN_RVA, 0);
        set(DONE_RVA, 1);
    }
    Ok(g)
}

/// `race::open_replay` (0x41E6EC): `dword_5BEC84` 0; with replays on (`dword_5BEC78` 1,
/// `dword_5BEC7C` 0, `dword_52A910` not 1, `dword_75ACDC` not 1) and 30 MB free: the
/// header ("REPL", version 0x50400, the date, "tmpreplay", the race's settings), the
/// car's header (`seal_replay_header`) copied into it, the index "RCIB" cleared, the
/// cursor, counts and length 0; `var\tmprpl` created; the header written. A failed open
/// or write closes it (`race::close_replay`) and sets carry. Every register kept.
pub unsafe fn open_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        set(DONE_RVA, 0);
        if get(rva(0x5BEC78)) != 1
            || get(rva(0x5BEC7C)) != 0
            || get(rva(0x52A910)) == 1
            || get(rva(0x75ACDC)) == 1
        {
            return Ok(g);
        }
        let mut r = g;
        part(4, &mut r)?;
        if r.eax < MOST_FREE_KB {
            return Ok(g);
        }
        for k in 0..0x10C {
            set(HEADER_RVA + 4 * k, 0);
        }
        set(HEADER_RVA, 0x4C50_4552);
        set(HEADER_RVA + 8, 0x50400);
        // After `rep stosd`: eax and ecx 0, edi past the header.
        r.eax = 0;
        r.ecx = 0;
        r.edi = game_addr(HEADER_RVA) as u32 + HEADER_LEN;
        part(5, &mut r)?;
        set(HEADER_RVA + 0xC, r.eax);
        set(HEADER_RVA + 0x10, 0x7270_6D74);
        set(HEADER_RVA + 0x14, 0x616C_7065);
        set(HEADER_RVA + 0x18, 0x79);
        set(rva(0x719B34), get(rva(0x514F78)));
        set(rva(0x719B38), get(rva(0x75AE20)));
        set(rva(0x719B3C), get(rva(0x75AF8C)));
        set(rva(0x719C54), 6);
        std::ptr::copy(
            game_addr(rva(0x71C6D0)) as *const u8,
            game_addr(rva(0x719C58)) as *mut u8,
            0xD * 4,
        );
        r.eax = get(rva(0x75AF8C));
        r.ecx = 0;
        r.esi = game_addr(rva(0x71C6D0)) as u32 + 0x34;
        r.edi = game_addr(rva(0x719C58)) as u32 + 0x34;
        part(10, &mut r)?;
        std::ptr::copy(
            game_addr(CAR_HEADER_RVA) as *const u8,
            game_addr(HEADER_RVA + 0x30) as *mut u8,
            0x400,
        );
        std::ptr::write_bytes(game_addr(INDEX_RVA) as *mut u8, 0, INDEX_LEN as usize);
        r.esi = game_addr(CAR_HEADER_RVA) as u32 + 0x400;
        r.edi = game_addr(INDEX_RVA) as u32 + INDEX_LEN;
        r.ecx = 0;
        set(INDEX_RVA, 0x4249_4352);
        set(CURSOR_RVA, game_addr(BLOCK_RVA) as u32);
        set(IN_BLOCK_RVA, 0);
        set(OFFSET_RVA, 0);
        r.eax = game_addr(PATH_RVA) as u32;
        r.ebx = 1;
        part(3, &mut r)?;
        let failed = r.carry() || {
            set(HANDLE_RVA, r.eax);
            set(OPEN_RVA, 1);
            r.eax = get(HANDLE_RVA);
            r.ebx = game_addr(HEADER_RVA) as u32;
            r.ecx = HEADER_LEN;
            set(OFFSET_RVA, get(OFFSET_RVA).wrapping_add(HEADER_LEN));
            part(1, &mut r)?;
            r.carry()
        };
        if failed {
            part(11, &mut r)?;
            g.eflags = r.eflags;
            g.set_carry(true);
        } else {
            g.set_carry(false);
            g.set_zero(r.eax == 0);
        }
    }
    Ok(g)
}

/// The calls in the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::files::handle;
    unsafe {
        match place {
            1 => {
                let (n, ok) = handle::write(r.eax, r.ebx as *const u8, r.ecx);
                r.eax = n;
                r.set_carry(!ok);
                r.set_zero(n == r.ecx);
            }
            2 => {
                let ok = handle::close(r.eax);
                r.eax = ok as u32;
                r.set_carry(!ok);
                r.set_zero(ok);
            }
            3 => {
                let h = handle::open_by_mode(r.eax as *const u8, r.ebx)?;
                r.eax = h.unwrap_or(handle::NO_HANDLE);
                r.set_carry(h.is_none());
            }
            4 => r.eax = crate::logfile::start_log::free_space_kb(),
            5 => r.eax = crate::system::packed_local_time::packed_local_time(),
            6 => r.ecx = crate::race::save_seal::pack_runs(r.esi, r.ecx, r.edi),
            7 => crate::race::race_shell::seal_stage_time(),
            8 => crate::race::save_seal::seal_block(),
            9 => crate::strings::copy_printable::copy_printable(
                r.eax as *const u8,
                r.ebx as *mut u8,
                r.ecx,
            )?,
            10 => *r = seal_replay_header(r, &mut game_part)?,
            11 => *r = close_replay(r, &mut game_part)?,
            12 => *r = add_replay_frame(r, &mut game_part)?,
            13 => *r = write_replay_block(r, &mut game_part)?,
            _ => *r = write_replay_chunk(r, &mut game_part)?,
        }
    }
    Ok(())
}

/// What the replay's functions write, for saving and comparing.
pub fn regions() -> Vec<Region> {
    let mut r = vec![
        (game_addr(INDEX_RVA) as u32, INDEX_LEN as usize),
        (game_addr(BLOCK_RVA) as u32, BLOCK_LEN as usize),
        (game_addr(HEADER_RVA) as u32, HEADER_LEN as usize),
        (game_addr(SEALED_RVA) as u32, 0x100),
        (game_addr(rva(0x719B34)) as u32, 12),
        (game_addr(rva(0x719C54)) as u32, 4 + 0xD * 4),
        (game_addr(rva(0x719C8C)) as u32, 1),
        (game_addr(rva(0x4E052C)) as u32, 0x18),
        (game_addr(rva(0x4E04AC)) as u32, 0x80),
        (game_addr(rva(0x4F66F9)) as u32, 4),
    ];
    r.extend(
        [
            PLAIN_RVA,
            OPEN_RVA,
            DONE_RVA,
            HANDLE_RVA,
            OFFSET_RVA,
            IN_BLOCK_RVA,
            LAST_BLOCK_RVA,
            CURSOR_RVA,
        ]
        .map(at),
    );
    r
}

// ---- for the self-tests ----

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static CALLED: Mutex<Vec<(u32, [u32; 6])>> = Mutex::new(Vec::new());
/// The stand-ins' answers: a write that fails, an open that fails, a chunk that fails,
/// the free space, the packing's length, the chunk's result.
pub static FAIL_WRITE: AtomicBool = AtomicBool::new(false);
pub static FAIL_OPEN: AtomicBool = AtomicBool::new(false);
pub static FAIL_CHUNK: AtomicBool = AtomicBool::new(false);
pub static FREE_KB: AtomicU32 = AtomicU32::new(0x10000);
pub static PACKED: AtomicU32 = AtomicU32::new(0x100);
pub static CHUNK: AtomicU32 = AtomicU32::new(0xFFFF_FF00);

/// The stand-ins: each call noted with its registers, then the answers above (the frame's
/// stand-in empties the block's count, so `close_replay`'s loop ends).
pub fn note(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi]));
    match place {
        1 => {
            let fail = FAIL_WRITE.load(Ordering::SeqCst);
            r.eax = if fail { 0 } else { r.ecx };
            r.set_carry(fail);
            r.set_zero(!fail);
        }
        2 => {
            r.eax = 1;
            r.set_carry(false);
            r.set_zero(true);
        }
        3 => {
            let fail = FAIL_OPEN.load(Ordering::SeqCst);
            r.eax = if fail { u32::MAX } else { 0x5A5A_0001 };
            r.set_carry(fail);
        }
        4 => r.eax = FREE_KB.load(Ordering::SeqCst),
        5 => r.eax = 0x07E9_1234,
        6 => r.ecx = PACKED.load(Ordering::SeqCst),
        12 => unsafe { set(IN_BLOCK_RVA, 0) },
        14 => {
            let fail = FAIL_CHUNK.load(Ordering::SeqCst);
            r.eax = CHUNK.load(Ordering::SeqCst);
            r.set_carry(fail);
        }
        _ => {}
    }
}

pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

pub fn take_called() -> Vec<(u32, [u32; 6])> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// A copy whose calls (and, for `write_replay_chunk`, its jump to `files::write`) go to
/// the stand-ins by place.
pub fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    jump_out: Option<usize>,
) -> Result<crate::redirect::RedirectedCopy, &'static str> {
    crate::install::stand_ins::set_handler(note);
    let copy = unsafe {
        crate::redirect::RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            PLACES
                .iter()
                .position(|&k| k != 0 && rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        })
    }?;
    if let Some(offset) = jump_out {
        unsafe { copy.retarget(offset + 2, crate::install::stand_ins::stand_in(1)) };
    }
    Ok(copy)
}

/// One case: the copy and the Rust version from `before` after `prepare`; the calls with
/// their registers, the registers left (and the carry, with `carry`: only where callers
/// read it), and `regions` compared.
pub fn run_both(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &crate::redirect::RedirectedCopy,
    before: Regs,
    prepare: &dyn Fn(),
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
    carry: bool,
) {
    use crate::service::harness::capture;
    let regions = regions();
    let regs = |r: &Regs| {
        (
            [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
            carry && r.carry(),
        )
    };
    prepare();
    take_called();
    let mut ro = before;
    unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
    prepare();
    let mine = rust(&before);
    let mine = (
        mine.as_ref().ok().map(regs),
        take_called(),
        capture(&regions),
    );
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}

/// Puts the replay's variables in a test's state: everything 0x5A but the switches and
/// counts given.
pub fn prepare(open: u32, plain: u32, blocks: u32, frames: u32, in_block: u32) {
    unsafe {
        for &(a, len) in &regions() {
            std::ptr::write_bytes(a as *mut u8, 0x5A, len);
        }
        set(OPEN_RVA, open);
        set(PLAIN_RVA, plain);
        set(BLOCKS_RVA, blocks);
        set(FRAMES_RVA, frames);
        set(IN_BLOCK_RVA, in_block);
        set(CURSOR_RVA, game_addr(BLOCK_RVA) as u32 + 0x40);
        set(HANDLE_RVA, 0x5A5A_0001);
        set(OFFSET_RVA, 0x1000);
        set(LAST_BLOCK_RVA, blocks.wrapping_sub(1));
    }
}
