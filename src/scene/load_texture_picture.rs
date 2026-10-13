//! `scene::load_texture_picture` (0x42578E): loads a numbered picture into a system
//! texture.
//!
//! In:  eax = the picture's number, ebx = the texture
//! Out: the original leaves eax, ebx, ecx, edx and ebp changed; its one caller keeps
//!      them all. The Rust version keeps them.
//!
//! A picture is `tx<number>.pcx`, four decimal digits, and with it goes the picture
//! numbered one higher (by the look of it its transparency). The folder is by the
//! number: `files\a` under 900, `files\m` for 900 and 901, `files\a` up to 999,
//! `files\b` up to 1999, `files\g\frnt` up to 2999, else `files\s\extr`. The digits
//! are written into all six names and into five of the six second names (the
//! `files\s\extr` one keeps its `xxxx`; and the `files\m` name gets its third digit
//! twice: the original's slips, kept).
//!
//! For a picture 1000 .. 1999 of a car (the texture less kind 0x100's first texture,
//! over 8, is a car's number under `dword_63DB90`): "carnum <n> grav <g>" is logged,
//! g being `dword_6235FC[car]`; with g at 3 or more and the file there, the picture
//! is taken from `files\b\gravel` instead, and its second name logged.
//!
//! Then `gfx::load_picture_to_texture` (0x405313, not ported) with eax = the name,
//! ecx = the second name, ebx = the texture.
//! One caller, `scene::load_changed_texture_picture`.
//! Shadow mode runs the original alone and compares the names it left with the Rust
//! version's.
//! Self-test: a redirected copy whose loading notes the names, 336 cases.

use crate::errors::Refused;
use crate::files::file_exists::file_exists;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::redirect::RedirectedCopy;
use crate::scene::textures::KIND_TEXTURE_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::load_texture_picture",
    rva(0x42578E),
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

/// `dword_60AF04` the picture's number, `dword_60AF08` the texture, then the twelve
/// names up to the end of `files\s\extr\txxxxx.pcx`.
const WORK_RVA: usize = rva(0x60AF04);
const WORK_LEN: usize = 0x10E;
/// The six folders' names and second names, and where the number is in the name.
const FOLDERS: [(usize, usize, usize); 6] = [
    (rva(0x60AF0C), rva(0x60AF8F), 0x0A), // files\a
    (rva(0x60AF1F), rva(0x60AFA2), 0x0A), // files\m
    (rva(0x60AF32), rva(0x60AFB5), 0x0A), // files\b
    (rva(0x60AF45), rva(0x60AFC8), 0x11), // files\b\gravel
    (rva(0x60AF5F), rva(0x60AFE2), 0x0F), // files\g\frnt
    (rva(0x60AF77), rva(0x60AFFA), 0x0F), // files\s\extr
];
const GRAVEL: usize = 3;
const FIRST_CAR_TEXTURE_RVA: usize = KIND_TEXTURE_RVA + 0x100 * 4; // dword_5D0EE8
const CARS_RVA: usize = rva(0x63DB90); // dword_63DB90
const CAR_GRAVEL_RVA: usize = rva(0x6235FC); // dword_6235FC
const LOAD_RVA: usize = rva(0x405313); // gfx::load_picture_to_texture

/// What the function calls.
pub enum Call {
    Text(*const u8),
    Number(u32),
    /// Answers whether the file is there.
    Exists(usize),
    /// The name, the second name, the texture.
    Load(usize, usize, u32),
}

// ---- Rust version ----

fn digits(number: u32) -> [u8; 4] {
    [
        (number / 1000) as u8,
        (number % 1000 / 100) as u8,
        (number % 100 / 10) as u8,
        (number % 10) as u8,
    ]
    .map(|digit| digit.wrapping_add(b'0'))
}

/// The folder a picture's number picks.
fn folder(number: u32) -> usize {
    match number {
        0..0x384 => 0,
        0x384..0x386 => 1,
        0x386..0x3E8 => 0,
        0x3E8..0x7D0 => 2,
        0x7D0..0xBB8 => 4,
        _ => 5,
    }
}

/// The numbers and the names: everything the function writes.
unsafe fn write_names(number: u32, texture: u32) {
    unsafe {
        game_write(WORK_RVA, [number, texture]);
        let (first, second) = (digits(number), digits(number.wrapping_add(1)));
        for (n, &(name, second_name, at)) in FOLDERS.iter().enumerate() {
            let mut first = first;
            if n == 1 {
                // The original stores the third digit where the fourth belongs.
                first[3] = first[2];
            }
            (game_addr(name + at) as *mut [u8; 4]).write(first);
            if n != 5 {
                (game_addr(second_name + at) as *mut [u8; 4]).write(second);
            }
        }
    }
}

pub unsafe fn load_texture_picture(number: u32, texture: u32, call: &mut dyn FnMut(Call) -> bool) {
    unsafe {
        write_names(number, texture);
        let mut chosen = folder(number);
        if (1000..2000).contains(&number) {
            let car = texture.wrapping_sub(game_read(FIRST_CAR_TEXTURE_RVA)) >> 3;
            if car < game_read::<u32>(CARS_RVA) {
                call(Call::Text(c"carnum".as_ptr().cast()));
                call(Call::Number(car));
                call(Call::Text(c"grav".as_ptr().cast()));
                let gravel: u32 = game_read(CAR_GRAVEL_RVA + car as usize * 4);
                call(Call::Number(gravel));
                // By the difference's sign, as the original has it.
                if (gravel.wrapping_sub(3) as i32) >= 0 && call(Call::Exists(FOLDERS[GRAVEL].0)) {
                    chosen = GRAVEL;
                    call(Call::Text(game_ptr(FOLDERS[GRAVEL].1)));
                }
            }
        }
        let (name, second, _) = FOLDERS[chosen];
        call(Call::Load(name, second, texture));
    }
}

/// With the game's log, files and loading.
pub unsafe fn load_texture_picture_in_game(number: u32, texture: u32) -> Result<(), Refused> {
    let mut refused = None;
    unsafe {
        load_texture_picture(number, texture, &mut |call| {
            match call {
                Call::Text(text) => log_write(text, false, 1),
                Call::Number(number) => log_int(number as i32, false, 1),
                Call::Exists(path) => match file_exists(game_ptr(path)) {
                    Ok(exists) => return exists,
                    Err(e) => refused = Some(e),
                },
                Call::Load(name, second, texture) => {
                    let mut r = Regs {
                        eax: game_addr(name) as u32,
                        ecx: game_addr(second) as u32,
                        ebx: texture,
                        ..Regs::default()
                    };
                    call_game(&mut r, game_addr(LOAD_RVA));
                }
            }
            false
        });
    }
    refused.map_or(Ok(()), |e| Err(e.into()))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        // The names as the Rust version writes them, then the original's.
        let region = [(game_addr(WORK_RVA) as u32, WORK_LEN)];
        let before = capture(&region);
        unsafe { write_names(r.eax, r.ebx) };
        let mine = capture(&region);
        restore(&region, &before);
        let (number, texture) = (r.eax, r.ebx);
        unsafe { call_original(r) };
        if capture(&region) != mine {
            HOOK.mismatch(|| {
                format!("picture {number:#x}, texture {texture:#x}: the names differ")
            });
        }
        return;
    }
    if let Err(e) = unsafe { load_texture_picture_in_game(r.eax, r.ebx) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const START: usize = 0x42578E;
const LEN: usize = 0x210;
const ENTRY_BYTES: [u8; 5] = [0xA3, 0x04, 0xAF, 0x60, 0x00];
const CALL_SITES: [usize; 9] = [
    0x425925, 0x42592C, 0x425939, 0x425947, 0x425958, 0x425977, 0x42597E, 0x425985, 0x425998,
];

/// Each call: its place (0 a text, 1 a number, 2 "is the file there", 3 the loading)
/// and what it was given: a text's bytes as a mark, the names by their place.
static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());
/// Whether the gravel picture is there.
static EXISTS: Mutex<bool> = Mutex::new(false);

fn mark(text: u32) -> u32 {
    unsafe { std::ffi::CStr::from_ptr(text as *const _) }
        .to_bytes()
        .iter()
        .fold(0x811C_9DC5u32, |mark, &byte| {
            (mark ^ byte as u32).wrapping_mul(0x0100_0193)
        })
}

fn handler(place: u32, r: &mut Regs) {
    let given = match place {
        0 => [0, mark(r.eax), 0, 0],
        1 => [1, r.eax, 0, 0],
        2 => [2, mark(r.eax), 0, 0],
        _ => [3, mark(r.eax), mark(r.ecx), r.ebx],
    };
    CALLED.lock().unwrap().push(given);
    if place == 2 {
        r.set_carry(!*EXISTS.lock().unwrap());
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
                // logfile::log_write_l1, log_int_l1, files::file_exists, the loading
                [rva(0x401427), rva(0x401472), rva(0x402EBE), LOAD_RVA]
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
    let regions: [Region; 1] = [(game_addr(WORK_RVA) as u32, WORK_LEN)];
    let inputs: [Region; 3] = [
        at(FIRST_CAR_TEXTURE_RVA),
        at(CARS_RVA),
        (game_addr(CAR_GRAVEL_RVA) as u32, 8),
    ];
    let saved = (capture(&regions), capture(&inputs));
    const FIRST: u32 = 0x40;
    for number in [
        5u32, 0x383, 0x384, 0x385, 0x386, 0x3E7, 1000, 1500, 1999, 2000, 0xBB7, 0xBB8, 9999,
        123_456,
    ] {
        for car in [0u32, 1, 2] {
            for gravel in [2u32, 3, 0x8000_0000, 7] {
                for exists in [false, true] {
                    let texture = FIRST + car * 8 + 3;
                    let run = |rust: bool| unsafe {
                        restore(&regions, &saved.0);
                        game_write(FIRST_CAR_TEXTURE_RVA, FIRST);
                        game_write(CARS_RVA, 2u32);
                        game_write(CAR_GRAVEL_RVA, [gravel, gravel ^ 1]);
                        CALLED.lock().unwrap().clear();
                        *EXISTS.lock().unwrap() = exists;
                        set_handler(handler);
                        if rust {
                            load_texture_picture(number, texture, &mut |call| {
                                let mut r = Regs::default();
                                let place = match call {
                                    Call::Text(text) => {
                                        r.eax = text as u32;
                                        0
                                    }
                                    Call::Number(number) => {
                                        r.eax = number;
                                        1
                                    }
                                    Call::Exists(path) => {
                                        r.eax = game_addr(path) as u32;
                                        2
                                    }
                                    Call::Load(name, second, texture) => {
                                        r.eax = game_addr(name) as u32;
                                        r.ecx = game_addr(second) as u32;
                                        r.ebx = texture;
                                        3
                                    }
                                };
                                act(place, &mut r);
                                place == 2 && !r.carry()
                            });
                        } else {
                            let mut r = Regs {
                                eax: number,
                                ebx: texture,
                                ..Regs::SENTINEL
                            };
                            call_game(&mut r, copy.addr());
                        }
                        (capture(&regions), CALLED.lock().unwrap().clone())
                    };
                    let (theirs, mine) = (run(false), run(true));
                    t.check(theirs == mine, || {
                        format!(
                            "picture {number}, car {car}, gravel {gravel:#x}, there {exists}: calls {:x?} / {:x?}; names same {}",
                            theirs.1,
                            mine.1,
                            theirs.0 == mine.0
                        )
                    });
                }
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
