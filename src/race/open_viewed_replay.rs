//! `race::open_viewed_replay` (0x41E9FF): a replay opened for the viewer: its file
//! (`race::replay_path`; the last race's only when one was recorded, `dword_5BEC84`; a
//! saved one, below 100 after the path's, copied over `var\tmprpl` first,
//! `files::copy_file`) opened (mode 0) into `h`, `dword_5BEC88` 1, block and frame 0;
//! its header (0x430 bytes into `dword_528504`: "REPL", version 0x50400) and its index
//! (the last 0x3850 bytes into "RCIB": its name, the block count held to 0x708, more
//! than 0) read; each block read (`drive::read_replay_frame_0`) for its first 8 bytes,
//! the block's cameras (`unk_963BB4`); the marks round the whole; the first block read
//! again, the cars' header (0x400 bytes from `unk_528534`) into `dword_719B14`, and the
//! replay's physics patches in force (`race::apply_replay_patches`). Anything failing:
//! the replay closed (`race::close_viewed_replay`) and carry.
//!
//! In:  ebx the replay (`race::replay_path`'s number)
//! Out: carry set when it failed; every register kept (`pusha` / `popa`)
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14), `race::save_last_replay`,
//! 0x444014 (`sub_443FxF`'s replay run). Its calls through `part`: 1 `files::copy_file`
//! (eax from, ebx to), 2 `files::open_by_mode` (eax path, ebx mode -> eax handle, carry),
//! 3 `files::read` (eax handle, ebx buffer, ecx bytes -> eax bytes read, carry), 4
//! `files::seek` (eax handle, ecx distance, ebx method -> carry), 5
//! `drive::read_replay_frame_0` (eax block -> carry), 6 `race::close_viewed_replay`, 7
//! `race::apply_replay_patches`; in the game their Rust versions (`replay_path` is called
//! as it is). Shadow mode runs the original alone (a saved replay is copied over the
//! last race's). Self-test: a copy whose calls go to stand-ins that play a file system
//! (headers and indexes good and bad, reads failing at each place), against the Rust
//! version; the variables, the calls (eax, ebx, ecx, edx) and carry compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::replay_path::{LAST_RACE_PATH, replay_path};
use crate::race::viewer::{
    BLOCK, BLOCK_CURSOR, BLOCK_DATA, BLOCKS, CAMERAS, CAR_HEADER, CF, FRAME, HANDLE, HEADER, INDEX,
    MARK_IN, MARK_OUT, OPEN, Part, Saved, TABLE_LEN, answer, dispatch_parts, dword, get, put,
    region, run_both, test_part,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::open_viewed_replay", rva(0x41E9FF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RECORDED: usize = 0x5BEC84; // dword_5BEC84: a replay was recorded
const CARS_FROM: usize = 0x528534; // unk_528534
use rc2k_core::race::replay::{
    HEADER_MAGIC as REPL, HEADER_VERSION as VERSION, INDEX_MAGIC as RCIB,
};
use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};

/// The Rust version: the registers and flags it leaves.
pub fn open_viewed_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{last_race_missing, copied_first,
    // read_good, version_good, index_blocks, opened_marks, opened_zero}`.
    use rc2k_core::race::viewer_flow::{
        Blocks, CAMERA_DWORDS, CAR_HEADER_LEN, copied_first, index_blocks, last_race_missing,
        opened_marks, opened_zero, read_good, version_good,
    };
    let mut out = *entry;
    let failed = |part: Part, out: &mut Regs, r: &Regs| -> Result<Regs, Refused> {
        let mut s = *r;
        part(6, &mut s)?;
        out.set_carry(true);
        Ok(*out)
    };
    let mut r = replay_path(entry)?;
    let is_last_race = r.eflags & crate::framework::ZERO_FLAG != 0;
    if last_race_missing(is_last_race, || get(RECORDED)) {
        return failed(part, &mut out, &r);
    }
    if copied_first(r.ebx) {
        r.ebx = a(LAST_RACE_PATH);
        part(1, &mut r)?;
        if r.carry() {
            return failed(part, &mut out, &r);
        }
        r.eax = a(LAST_RACE_PATH);
    }
    r.ebx = 0;
    part(2, &mut r)?;
    if r.carry() {
        return failed(part, &mut out, &r);
    }
    put(HANDLE, r.eax);
    put(OPEN, 1);
    put(BLOCK, 0);
    put(FRAME, 0);
    let read = |part: Part, r: &mut Regs, buffer: usize, len: u32| -> Result<bool, Refused> {
        r.eax = get(HANDLE);
        r.ebx = a(buffer);
        r.ecx = len;
        part(3, r)?;
        Ok(read_good(
            r.carry(),
            r.eax,
            r.ecx,
            get(buffer),
            buffer == HEADER,
        ))
    };
    if !read(part, &mut r, HEADER, HEADER_LEN)? {
        return failed(part, &mut out, &r);
    }
    // `files::version_differs` (eax the version, kept): on another version the close gets
    // eax 0x50400.
    r.eax = VERSION;
    if !version_good(get(HEADER + 8)) {
        return failed(part, &mut out, &r);
    }
    r.eax = get(HANDLE);
    r.ebx = 2;
    r.ecx = INDEX_LEN.wrapping_neg();
    part(4, &mut r)?;
    if r.carry() || !read(part, &mut r, INDEX, INDEX_LEN)? {
        return failed(part, &mut out, &r);
    }
    match index_blocks(get(BLOCKS)) {
        Blocks::Bad => return failed(part, &mut out, &r),
        Blocks::Held(n) => put(BLOCKS, n),
        Blocks::Kept => {}
    }
    let mut block = 0u32;
    loop {
        r.eax = block;
        part(5, &mut r)?;
        if r.carry() {
            return failed(part, &mut out, &r);
        }
        crate::race::viewer::move_dwords(CAMERAS + 8 * block as usize, BLOCK_DATA, CAMERA_DWORDS);
        r.ecx = 0;
        r.esi = a(BLOCK_DATA) + 8;
        r.edi = a(CAMERAS) + 8 * block + 8;
        block += 1;
        if block == get(BLOCKS) {
            break;
        }
    }
    let (mark_in, mark_out) = opened_marks(get(BLOCKS));
    put(MARK_IN, mark_in);
    put(MARK_OUT, mark_out);
    put(BLOCK_CURSOR, a(BLOCK_DATA));
    r.eax = 0;
    part(5, &mut r)?;
    if r.carry() {
        return failed(part, &mut out, &r);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            a(CARS_FROM) as *const u8,
            a(CAR_HEADER) as *mut u8,
            CAR_HEADER_LEN,
        )
    };
    r.ecx = 0;
    r.esi = a(CARS_FROM) + CAR_HEADER_LEN as u32;
    r.edi = a(CAR_HEADER) + CAR_HEADER_LEN as u32;
    part(7, &mut r)?;
    out.set_carry(false);
    out.set_zero(opened_zero(r.eax));
    Ok(out)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::files::handle;
    unsafe {
        match place {
            1 => {
                let done = crate::files::copy_file::copy_file(r.eax as _, r.ebx as _)?;
                r.set_carry(!done);
            }
            2 => match handle::open_by_mode(r.eax as _, r.ebx)? {
                Some(file) => {
                    r.eax = file;
                    r.set_carry(false);
                }
                None => {
                    r.eax = u32::MAX;
                    r.set_carry(true);
                }
            },
            3 => {
                let (n, ok) = handle::read(r.eax, r.ebx as *mut u8, r.ecx);
                r.eax = n;
                r.set_carry(!ok);
            }
            4 => {
                let at = handle::seek(r.eax, r.ecx as i32, r.ebx);
                r.set_carry(at.is_none());
            }
            5 => {
                let failed = crate::drive::read_replay_frame_0::read_replay_frame_0(r.eax)?;
                r.set_carry(failed);
            }
            6 => {
                *r = crate::race::close_viewed_replay::close_viewed_replay(
                    r,
                    &mut crate::race::close_viewed_replay::game_part,
                )?
            }
            _ => {
                *r = crate::race::replay_patches::apply_replay_patches(
                    r,
                    &mut crate::race::replay_patches::game_part,
                )?
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        open_viewed_replay(r, &mut game_part)
    });
}

// ---- self-test ----

/// What it writes.
pub fn regions() -> Vec<Region> {
    vec![
        dword(HANDLE),
        dword(OPEN),
        dword(BLOCK),
        dword(FRAME),
        region(HEADER, HEADER_LEN as usize),
        region(INDEX, INDEX_LEN as usize),
        region(CAMERAS, TABLE_LEN),
        dword(MARK_IN),
        dword(MARK_OUT),
        dword(BLOCK_CURSOR),
        region(CAR_HEADER, 0x400),
        region(BLOCK_DATA, 8),
        region(0x94E8B0, 0x100),
    ]
}

/// A test's file system: which call fails, what the header and index say.
#[derive(Clone, Copy, Debug)]
struct Files {
    copy_fails: bool,
    open_fails: bool,
    magic: u32,
    version: u32,
    header_short: bool,
    seek_fails: bool,
    index_magic: u32,
    blocks: u32,
    /// The block whose read fails (or none).
    block_fails: u32,
}

const GOOD: Files = Files {
    copy_fails: false,
    open_fails: false,
    magic: REPL,
    version: VERSION,
    header_short: false,
    seek_fails: false,
    index_magic: RCIB,
    blocks: 3,
    block_fails: u32::MAX,
};

fn act(f: Files, place: u32, r: &mut Regs) {
    let fill_at = |at: u32, len: usize, seed: u8| unsafe {
        for n in 0..len {
            ((at as usize + n) as *mut u8).write(seed.wrapping_add(n as u8));
        }
    };
    match place {
        1 => r.set_carry(f.copy_fails),
        2 => {
            r.eax = if f.open_fails { u32::MAX } else { 0x1234 };
            r.set_carry(f.open_fails);
        }
        3 => {
            fill_at(r.ebx, r.ecx as usize, 0x21);
            unsafe {
                if r.ecx == HEADER_LEN {
                    (r.ebx as *mut u32).write_unaligned(f.magic);
                    ((r.ebx + 8) as *mut u32).write_unaligned(f.version);
                } else {
                    (r.ebx as *mut u32).write_unaligned(f.index_magic);
                    ((r.ebx + 8) as *mut u32).write_unaligned(f.blocks);
                }
            }
            r.eax = if f.header_short && r.ecx == HEADER_LEN {
                0x400
            } else {
                r.ecx
            };
            r.set_carry(false);
        }
        4 => r.set_carry(f.seek_fails),
        5 => {
            fill_at(a(BLOCK_DATA), 8, 0x40 + r.eax as u8);
            r.set_carry(r.eax == f.block_fails);
        }
        _ => {}
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call race::replay_path
    let copy = match crate::race::viewer::copy(
        0x41E9FF,
        0x17D,
        &[0x60, 0xE8, 0x22, 0x05, 0x00, 0x00],
        &[
            0x1, 0x20, 0x36, 0x74, 0x98, 0xB2, 0xCD, 0x111, 0x152, 0x16C, 0x175,
        ],
        &[
            (0x402C28, 1),
            (0x402DE8, 2),
            (0x402E3D, 3),
            (0x402E75, 4),
            (0x41EBA1, 5),
            (0x41EB7C, 6),
            (0x41F2C9, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = regions();
    all.push(dword(RECORDED));
    let saved = Saved::new(&all);
    let cases: [(&str, u32, u32, Files); 13] = [
        ("the last race's", 222, 1, GOOD),
        ("the last race's, none recorded", 222, 0, GOOD),
        ("a saved one", 4, 0, GOOD),
        ("a game's own", 103, 0, Files { blocks: 1, ..GOOD }),
        (
            "the copy failing",
            4,
            0,
            Files {
                copy_fails: true,
                ..GOOD
            },
        ),
        (
            "the open failing",
            222,
            1,
            Files {
                open_fails: true,
                ..GOOD
            },
        ),
        (
            "not a replay",
            222,
            1,
            Files {
                magic: 0x1234,
                ..GOOD
            },
        ),
        (
            "another version",
            222,
            1,
            Files {
                version: 0x50300,
                ..GOOD
            },
        ),
        (
            "a short header",
            222,
            1,
            Files {
                header_short: true,
                ..GOOD
            },
        ),
        (
            "the seek failing",
            222,
            1,
            Files {
                seek_fails: true,
                ..GOOD
            },
        ),
        (
            "no index",
            222,
            1,
            Files {
                index_magic: 0,
                ..GOOD
            },
        ),
        ("no blocks", 222, 1, Files { blocks: 0, ..GOOD }),
        (
            "a block failing",
            222,
            1,
            Files {
                block_fails: 1,
                ..GOOD
            },
        ),
    ];
    for (what, number, recorded, files) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                ebx: number,
                ..Regs::SENTINEL
            },
            &regions(),
            CF,
            &|| {
                put(RECORDED, recorded);
                for (at, len) in regions() {
                    unsafe { std::ptr::write_bytes(at as *mut u8, 0xAA, len) };
                }
                answer(Box::new(move |place, r| act(files, place, r)));
            },
            &|r| open_viewed_replay(r, &mut test_part),
        );
    }
    // Past 0x708 blocks, held there (both sides read 0x708 blocks).
    run_both(
        &mut t,
        "too many blocks",
        copy.addr(),
        Regs {
            ebx: 222,
            ..Regs::SENTINEL
        },
        &regions(),
        CF,
        &|| {
            put(RECORDED, 1);
            answer(Box::new(|place, r| {
                act(
                    Files {
                        blocks: 0x709,
                        block_fails: 0x707,
                        ..GOOD
                    },
                    place,
                    r,
                )
            }));
        },
        &|r| open_viewed_replay(r, &mut test_part),
    );
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
