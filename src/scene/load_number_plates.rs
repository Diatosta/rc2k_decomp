//! `scene::load_number_plates` (0x4255AA): the two number plates' textures.
//!
//! In:  nothing
//! Out: kind 0xC1 has no first texture: ebx = -1, the rest preserved; else eax, ecx =
//!      the second plate's two picture names, edx, esi its two numbers, edi = 0xAD,
//!      ebp = 0x94, ebx = the texture
//!
//! Unless kind 0xC1's first texture (`dword_5D0DEC`) is none, twice, for the player's
//! car (`dword_70F410`, or with `dword_52A910` set the block of the car
//! `dword_52A920`, its +0x24) and then for `dword_7103F0`:
//! - `scene::known_kind_plate` of it gives the plate's number;
//! - under 0x12 the pictures `files\b\tx1822.pcx` and `tx1823.pcx`, else `tx1824.pcx`
//!   and `tx1825.pcx` (eax, ecx);
//! - edx, esi = `dword_5DCD9C[number]`, `dword_5DCE2C[number]`, edi = 0xAD, ebp = 0x68
//!   for the first and 0x94 for the second, ebx = the texture;
//! - `scene::upload_plate` (`pusha` .. `popa`).
//!
//! One caller, `scene::reload_system_textures`.
//! Refuses a number of 0x24 or more (the tables' room) and a car without its block.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose upload notes its registers.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::known_kind_plate::known_kind_plate;
use crate::scene::upload_plate::upload_plate_in_game;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::load_number_plates", rva(0x4255AA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TEXTURE_RVA: usize = rva(0x5D0DEC); // dword_5D0DEC: kind 0xC1's first texture
const PLAYER_CAR_RVA: usize = rva(0x70F410); // dword_70F410
const OTHER_CAR_RVA: usize = rva(0x7103F0); // dword_7103F0
const ONE_CAR_RVA: usize = rva(0x52A910); // dword_52A910
const WHICH_CAR_RVA: usize = rva(0x52A920); // dword_52A920
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const FIRSTS_RVA: usize = rva(0x5DCD9C); // dword_5DCD9C
const SECONDS_RVA: usize = rva(0x5DCE2C); // dword_5DCE2C
/// The tables' room: 0x90 bytes each.
const NUMBERS: u32 = 0x24;
const UPLOAD_RVA: usize = rva(0x41B2F2); // scene::upload_plate
/// `files\b\tx1822.pcx` .. `tx1825.pcx`.
const NAMES: [usize; 4] = [rva(0x60AEB7), rva(0x60AECA), rva(0x60AEDD), rva(0x60AEF0)];

// ---- Rust version ----

/// `upload` is called with the registers the original hands `scene::upload_plate`.
pub unsafe fn load_number_plates(
    r: &mut Regs,
    upload: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let texture: u32 = game_read(TEXTURE_RVA);
        r.ebx = texture;
        if texture == u32::MAX {
            return Ok(());
        }
        let player = if game_read::<u32>(ONE_CAR_RVA) != 0 {
            let car: u32 = game_read(WHICH_CAR_RVA);
            if car >= 8 {
                return Err(BadInput("the one car's number is past the blocks").into());
            }
            let block: u32 = game_read(BLOCKS_RVA + car as usize * 4);
            if block == 0 {
                return Err(BadInput("a car without its block").into());
            }
            ((block + 0x24) as *const u32).read_unaligned()
        } else {
            game_read(PLAYER_CAR_RVA)
        };
        let numbers = [
            known_kind_plate(player),
            known_kind_plate(game_read(OTHER_CAR_RVA)),
        ];
        if numbers.iter().any(|&n| n >= NUMBERS) {
            return Err(BadInput("a plate's number past its tables").into());
        }
        for (number, ebp) in numbers.into_iter().zip([0x68, 0x94]) {
            let names = if number < 0x12 { 0 } else { 2 };
            r.eax = game_addr(NAMES[names]) as u32;
            r.ecx = game_addr(NAMES[names + 1]) as u32;
            r.edx = game_read(FIRSTS_RVA + number as usize * 4);
            r.esi = game_read(SECONDS_RVA + number as usize * 4);
            r.edi = 0xAD;
            r.ebp = ebp;
            r.ebx = texture;
            upload(r)?;
        }
        Ok(())
    }
}

/// With the game's upload.
pub unsafe fn load_number_plates_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        // pusha .. popa: what it is given is what it leaves.
        load_number_plates(r, &mut |r| upload_plate_in_game(r))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { load_number_plates_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const START: usize = 0x4255AA;
const LEN: usize = 0xBF;
const ENTRY_BYTES: [u8; 6] = [0x8B, 0x1D, 0xEC, 0x0D, 0x5D, 0x00];
const CALL_SITES: [usize; 4] = [0x4255E4, 0x425618, 0x42562F, 0x425663];

/// Each upload's registers: eax, ebx, ecx, edx, esi, edi, ebp.
static CALLED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());

fn handler(_place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push([r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]);
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
            &|callee| (callee == UPLOAD_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs: [Region; 8] = [
        at(TEXTURE_RVA),
        at(PLAYER_CAR_RVA),
        at(OTHER_CAR_RVA),
        at(ONE_CAR_RVA),
        at(WHICH_CAR_RVA),
        (game_addr(BLOCKS_RVA) as u32, 8 * 4),
        (game_addr(FIRSTS_RVA) as u32, NUMBERS as usize * 4),
        (game_addr(SECONDS_RVA) as u32, NUMBERS as usize * 4),
    ];
    let saved = capture(&inputs);
    // The known kinds' plates as the game has them: the test's numbers come from them.
    let plates: Vec<u32> = (0..0x22).map(|p| unsafe { known_kind_plate(p) }).collect();
    if plates.iter().any(|&n| n >= NUMBERS) {
        restore(&inputs, &saved);
        return t.skip("the known kinds' plates are not loaded");
    }
    let mut block = [0u32; 16];
    let places = [0u32, 1, 5, 0x11, 0x12, 0x21, 0x30, 0xFFFF_FFFF];
    for (case, &(texture, player, other, one)) in [
        (u32::MAX, 0u32, 0u32, 0u32),
        (0x123, 0, 0x21, 0),
        (0x40, 5, 0x11, 0),
        (0x40, 0x12, 1, 0),
        (0x77, 0x30, 0xFFFF_FFFF, 0),
        (0x77, 0, 0, 1),
        (0x10, 0x21, 0x12, 1),
        (0x10, 3, 0x12, 1),
    ]
    .iter()
    .enumerate()
    {
        block[9] = places[case % places.len()];
        let run = |rust: bool| unsafe {
            game_write(TEXTURE_RVA, texture);
            game_write(PLAYER_CAR_RVA, player);
            game_write(OTHER_CAR_RVA, other);
            game_write(ONE_CAR_RVA, one);
            game_write(WHICH_CAR_RVA, case as u32 % 8);
            let mut blocks = [0u32; 8];
            blocks[case % 8] = block.as_ptr() as u32;
            game_write(BLOCKS_RVA, blocks);
            let firsts: Vec<u32> = (0..NUMBERS).map(|n| 0x1000 + n * 3).collect();
            let seconds: Vec<u32> = (0..NUMBERS).map(|n| 0x2000 + n * 5).collect();
            for n in 0..NUMBERS as usize {
                game_write(FIRSTS_RVA + n * 4, firsts[n]);
                game_write(SECONDS_RVA + n * 4, seconds[n]);
            }
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                if load_number_plates(&mut r, &mut |r| {
                    act(0, &mut { *r });
                    Ok(())
                })
                .is_err()
                {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            format!(
                "case {case}: texture {texture:#x}, cars {player:#x} {other:#x}, one {one}: regs {:x?} / {:x?}; uploads {:x?} / {:x?}",
                theirs.0, mine.0, theirs.1, mine.1
            )
        });
    }
    restore(&inputs, &saved);
    t.finish()
}
