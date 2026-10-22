//! `scene::try_allocate_textures` (0x4224C4): asks for the texture memory every needed
//! texture takes, at the levels they are needed at.
//!
//! In:  nothing
//! Out: carry clear = all of it was had; carry set = not. The original leaves the
//!      registers changed; its callers keep them all. The Rust version keeps them.
//!
//! - Each kind's first slot and first texture (`g_kindFirstSlots`,
//!   `g_kindFirstTextures`) = -1.
//! - The memory asked for is added up for the log: `dword_5D0AE0` over the textures
//!   needed in video memory and `dword_5D0AE4` over those in system memory, by their
//!   levels' sizes; "VMEMREQ <n> SMEMREQ <n>" is logged.
//! - Then for each of the 0x400 textures: if it is needed in video memory, its level
//!   (held to 0 .. 7) is added to both sums again and one video texture is asked for
//!   from `gfx::allocate_textures` (0x4051ED, Direct3D: not ported) with eax = 1,
//!   ebx = 0, cl = the level, edx = the pixel kind (3 for 18 textures by their number,
//!   else 2) and ebp = the mip levels (`dword_623898` times the level's entry of
//!   `dword_5CDABC`; none for textures 0x80 .. 0xFF and from 0x140 on:
//!   `scene::no_mips_for` 0x46512F, inlined); what comes back in eax is the texture's
//!   first slot. If it is needed in system memory: the same with eax = 0, ebx = 1 and
//!   the mip levels as they are; what comes back in ebx is its first texture, and
//!   `dword_5D2AE8` at that number = the texture.
//! - A refusal from `gfx::allocate_textures` ends it with "NO -" in the log and the
//!   carry; else "YES -".
//!
//! 46 call sites in `scene::build_texture_catalogue`.
//! Refuses a level of 13 or more (past the sizes' table).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose allocations and log calls are the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::redirect::RedirectedCopy;
use crate::scene::texture_needs::{
    LEVEL_SIZES_RVA, LEVELS, SYSTEM_LEVELS_RVA, TEXTURES, VIDEO_LEVELS_RVA, levels,
};
use crate::scene::textures::{KIND_SLOT_RVA, KIND_TEXTURE_RVA, test_table, write_table};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::try_allocate_textures",
    rva(0x4224C4),
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

// ---- game data ----

/// `dword_5D0AE0` video, `dword_5D0AE4` system: the memory asked for.
const ASKED_RVA: usize = rva(0x5D0AE0);
const MIPS_RVA: usize = rva(0x623898); // dword_623898
/// Mip levels a level has: the sizes' table from its fifth entry.
const LEVEL_MIPS_RVA: usize = rva(0x5CDABC); // dword_5CDABC
const TEXTURE_OF_RVA: usize = rva(0x5D2AE8); // dword_5D2AE8
const ALLOCATE_RVA: usize = rva(0x4051ED); // gfx::allocate_textures

pub use rc2k_core::scene::textures::{Ask, Call};

// ---- Rust version ----

/// The game's tables.
struct GameMemory;

impl rc2k_core::scene::textures::TextureMemory for GameMemory {
    fn level(&self, video: bool, texture: u32) -> u32 {
        let table = if video {
            VIDEO_LEVELS_RVA
        } else {
            SYSTEM_LEVELS_RVA
        };
        unsafe { game_read(table + texture as usize * 4) }
    }
    fn size(&self, level: u32) -> u32 {
        unsafe { game_read(LEVEL_SIZES_RVA + level as usize * 4) }
    }
    fn mips(&self) -> u32 {
        unsafe { game_read(MIPS_RVA) }
    }
    fn level_mips(&self, level: u32) -> u32 {
        unsafe { game_read(LEVEL_MIPS_RVA + level as usize * 4) }
    }
    fn forget_kinds(&mut self) {
        unsafe {
            std::ptr::write_bytes(game_addr(KIND_SLOT_RVA) as *mut u8, 0xFF, TEXTURES * 4);
            std::ptr::write_bytes(game_addr(KIND_TEXTURE_RVA) as *mut u8, 0xFF, TEXTURES * 4);
        }
    }
    fn set_asked(&mut self, asked: [u32; 2]) {
        unsafe { game_write(ASKED_RVA, asked) }
    }
    fn set_first_slot(&mut self, texture: u32, slot: u32) {
        unsafe { game_write(KIND_SLOT_RVA + texture as usize * 4, slot) }
    }
    fn set_first_texture(&mut self, texture: u32, first: u32) {
        unsafe {
            game_write(TEXTURE_OF_RVA + first as usize * 4, texture);
            game_write(KIND_TEXTURE_RVA + texture as usize * 4, first);
        }
    }
}

/// Whether all the memory was had.
// Converted: `rc2k_core::scene::textures::try_allocate`
pub unsafe fn try_allocate_textures(
    call: &mut dyn FnMut(Call) -> Option<u32>,
) -> Result<bool, BadInput> {
    Ok(rc2k_core::scene::textures::try_allocate(
        &mut GameMemory,
        &mut |c| call(c),
    )?)
}

/// With the game's log and the game's allocation.
pub unsafe fn try_allocate_textures_in_game() -> Result<bool, BadInput> {
    unsafe {
        try_allocate_textures(&mut |call| {
            match call {
                Call::Text(text, newline) => log_write(text.as_ptr().cast(), newline, 1),
                Call::Number(number) => log_int(number as i32, false, 1),
                Call::Allocate(ask) => {
                    let mut r = Regs {
                        eax: ask.video as u32,
                        ebx: !ask.video as u32,
                        ecx: ask.level,
                        edx: ask.pixel_kind,
                        ebp: ask.mips,
                        esi: ask.texture,
                        ..Regs::default()
                    };
                    crate::calls::call(ALLOCATE_RVA, &mut r);
                    if r.carry() {
                        return None;
                    }
                    return Some(if ask.video { r.eax } else { r.ebx });
                }
            }
            Some(0)
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { try_allocate_textures_in_game() } {
        Ok(had) => {
            r.set_carry(!had);
        }
        Err(e) => {
            let e: Refused = e.into();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

// ---- self-test ----

const START: usize = 0x4224C4;
const LEN: usize = 0x287;
const ENTRY_BYTES: [u8; 5] = [0xB8, 0xFF, 0xFF, 0xFF, 0xFF];
const CALL_SITES: [usize; 9] = [
    0x42253E, 0x42254A, 0x422557, 0x422563, 0x42263E, 0x422643, 0x42270B, 0x422734, 0x422743,
];

/// Each call: its place (0 a text, 1 a number, 2 a text and a line's end, 3 an
/// allocation) and what it was given.
static CALLED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());
/// The allocation that is refused (counted from 0), and how many have been made.
static REFUSED: Mutex<(u32, u32)> = Mutex::new((0, 0));

fn text_mark(text: *const u8) -> u32 {
    unsafe { (text as *const u32).read_unaligned() }
}

fn handler(place: u32, r: &mut Regs) {
    let mut called = CALLED.lock().unwrap();
    match place {
        0 | 2 => called.push([place, text_mark(r.eax as *const u8), 0, 0, 0, 0, 0]),
        1 => called.push([1, r.eax, 0, 0, 0, 0, 0]),
        _ => {
            called.push([3, r.eax, r.ebx, r.ecx & 0xFF, r.edx, r.ebp, r.esi]);
            let mut refused = REFUSED.lock().unwrap();
            let number = refused.1;
            refused.1 += 1;
            // What comes back: slots and textures counted up from numbers of the test's.
            r.set_carry(number == refused.0);
            r.eax = 0x100 + number;
            r.ebx = 0x20 + number % 0x300;
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = CALL_SITES.iter().map(|site| site - START).collect();
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(START)),
            LEN,
            &ENTRY_BYTES,
            &calls,
            &|callee| {
                // logfile::log_write_l1, log_int_l1, log_write_l1_nl, gfx::allocate_textures
                [rva(0x401427), rva(0x401472), rva(0x40142B), ALLOCATE_RVA]
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 4] = [
        levels(KIND_SLOT_RVA),
        levels(KIND_TEXTURE_RVA),
        (game_addr(ASKED_RVA) as u32, 8),
        levels(TEXTURE_OF_RVA),
    ];
    let inputs = [
        levels(VIDEO_LEVELS_RVA),
        levels(SYSTEM_LEVELS_RVA),
        at(MIPS_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0009u32;
    for case in 0..16u32 {
        let video = test_table(&mut seed, TEXTURES as u32, 1 + case % 4, LEVELS);
        let system = test_table(&mut seed, TEXTURES as u32, 2 + case % 3, LEVELS);
        let mips = [0u32, 1, 3, 0xFFFF_FFFF][case as usize % 4];
        // Refused never, at once, or somewhere on the way.
        let refuse = [u32::MAX, 0, 1, 37, 300, u32::MAX][case as usize % 6];
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            write_table(VIDEO_LEVELS_RVA, &video);
            write_table(SYSTEM_LEVELS_RVA, &system);
            game_write(MIPS_RVA, mips);
            CALLED.lock().unwrap().clear();
            *REFUSED.lock().unwrap() = (refuse, 0);
            set_handler(handler);
            let had = if rust {
                try_allocate_textures(&mut |call| {
                    let mut r = Regs::default();
                    match call {
                        Call::Text(text, newline) => {
                            r.eax = text.as_ptr() as u32;
                            act(if newline { 2 } else { 0 }, &mut r);
                        }
                        Call::Number(number) => {
                            r.eax = number;
                            act(1, &mut r);
                        }
                        Call::Allocate(ask) => {
                            r.eax = ask.video as u32;
                            r.ebx = !ask.video as u32;
                            r.ecx = ask.level;
                            r.edx = ask.pixel_kind;
                            r.ebp = ask.mips;
                            r.esi = ask.texture;
                            act(3, &mut r);
                            if r.carry() {
                                return None;
                            }
                            return Some(if ask.video { r.eax } else { r.ebx });
                        }
                    }
                    Some(0)
                })
                .ok()
            } else {
                let mut r = Regs::SENTINEL;
                call_game(&mut r, copy.addr());
                Some(!r.carry())
            };
            (had, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "case {case}: mips {mips:#x}, refused at {refuse:#x}: had {:?} / {:?}; {} / {} calls, the first that differs {first:?} ({:x?} / {:x?}); memory same {}",
                theirs.0,
                mine.0,
                theirs.2.len(),
                mine.2.len(),
                first.map(|n| theirs.2[n]),
                first.map(|n| mine.2[n]),
                theirs.1 == mine.1
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
