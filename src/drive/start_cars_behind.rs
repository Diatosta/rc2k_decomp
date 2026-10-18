//! `drive::start_cars_behind` (0x43703E): with `dword_62389C` 2 (by `sub_436758`), two
//! computer cars kept near the lead car (car block 0), put back on the line behind or
//! ahead of it as it passes them (`dword_70FA44` counts the moves; `dword_70FA48` and
//! `dword_70FA4C` are their cars, `dword_70FA50` / `54` their start numbers):
//! - once (`dword_70FA58`): the two set at starts `dword_6051BC` - 2 and - 1 (cars from
//!   `dword_604ECC`), 80 and 40 checkpoints on (`drive::start_car_on_line`);
//! - the gaps by `dword_70FA40` (held to 0 .. `dword_6051B4` - 1 by the signs) and the
//!   start's `dword_605034` (low four bits): 0x2C / 0x28 + it (`dword_6051D4`, `D8`,
//!   `DC` their sum);
//! - a car the lead has gone `dword_6051D4` past moved behind it (- `dword_6051D8`), one
//!   too far ahead of it moved ahead (+), when `drive::checkpoint_taken` allows; each
//!   moved car's start values cleared (`drive::clear_start_state`);
//! - unless `dword_70F494` is set: the cars ahead and behind counted into `dword_70FA40`,
//!   their least gaps (`drive::note_gap_ahead` / `behind`, from -1), the wanted gaps
//!   (0x70FA14, 0x70FA18; held to `dword_6051D8` .. `dword_6051DC`), eased into 0x70FA1C,
//!   0x70FA20 (times 0x14A; jumps past 1000, else 1/64 of the way);
//! - the four cars' +0x674 0, and for the two when within 8 checkpoints of each other (not
//!   0) a push apart of 8 less the gap (times `flt_604EC4`, as floats).
//!
//! By the look of it a time trial's pace cars (a reading).
//!
//! In:  nothing
//! Out: the registers as the path leaves them (through its calls); no flags read
//!
//! Caller: `drive::race_mode_cars` (0x436758).
//! Refuses a start count (`dword_6051B4`) above 0x58 (the start tables' room) or a start
//! table car past the eight blocks, before anything; the calls' refusals mid-way (the
//! cars, the starts' variables and the work car put back first; the calls' scratch is
//! not).
//! Shadow mode runs the original alone (`drive::start_car_on_line` does, for
//! `race::reset_car_motion`'s calls into the game).
//! Self-test: a copy whose 24 calls go to stand-ins (`install::stand_ins`), against the
//! Rust version with the same stand-ins: every call with its registers, the registers
//! left and the cars' and starts' memory compared; all put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::race::car_starts::Part;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::start_cars_behind", rva(0x43703E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place (IDA addresses).
pub const PLACES: [usize; 7] = [
    0x460C40, // 0: drive::load_car
    0x460C5D, // 1: drive::store_car
    0x437613, // 2: drive::start_car_on_line
    0x4375A5, // 3: drive::checkpoint_taken
    0x4376E3, // 4: drive::clear_start_state
    0x4373C0, // 5: drive::note_gap_behind
    0x4373A4, // 6: drive::note_gap_ahead
];
const LOAD: u32 = 0;
const STORE: u32 = 1;
const START: u32 = 2;
const TAKEN: u32 = 3;
const CLEAR: u32 = 4;
const BEHIND: u32 = 5;
const AHEAD: u32 = 6;

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

const COUNT: u32 = 0x6051B4; // the starts
const CARS_BY_START: u32 = 0x604ECC; // dword_604ECC
const LEAD: u32 = 0x70F484; // car block 0's checkpoint
const CURRENT: u32 = 0x71B3F8; // g_currentCar

fn block(car: u32) -> u32 {
    g(0x71B3B8 + 4 * car)
}

/// The refusals worked out before anything.
fn refusal() -> Option<&'static str> {
    let count = g(COUNT);
    if count > 0x58 {
        return Some("a start count above 0x58 (the start tables' room)");
    }
    for k in 0..=count + 1 {
        if g(CARS_BY_START + 4 * k) > 7 {
            return Some("a start table car past the eight blocks");
        }
    }
    if g(0x70FA48) > 7 || g(0x70FA4C) > 7 {
        return Some("a kept car past the eight blocks");
    }
    None
}

/// The memory put back on a refusal mid-way (and compared by the test).
pub fn regions() -> Vec<Region> {
    let a = |x: u32| game_addr(rva(x as usize)) as u32;
    let mut r = vec![
        (a(0x70F3EC), 8 * 0xFE0),
        (a(0x719FD8), 0xFE0),
        (a(0x6051A0), 0x40),
        (a(CURRENT), 4),
        (a(0x6365E4), 0xC),
    ];
    r.extend(crate::race::start_place::place_regions());
    r
}

/// One car moved: its slot `from_slot` emptied into the other's (`shift`), a new car at
/// start `start` with `dist` checkpoints on.
fn put_car(
    r: &mut Regs,
    part: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
    start: u32,
    slot: u32,
    number: u32,
    dist: u32,
    speed: u32,
) -> Result<(), Refused> {
    p(number, start);
    p(0x6051A0, start);
    r.ebx = start;
    r.eax = g(CARS_BY_START + 4 * start);
    p(CURRENT, r.eax);
    p(slot, r.eax);
    part(LOAD, r)?;
    p(0x6051B0, speed);
    r.ebx = dist;
    part(START, r)?;
    part(STORE, r)
}

/// `cmp; jns` held below `hi`, `cmp 0; jns` above 0, as the original holds the gap.
fn hold_signs(v: u32, lo_zero: bool, hi: u32) -> u32 {
    let mut v = v;
    if lo_zero && (v as i32) < 0 {
        v = 0;
    }
    if (v.wrapping_sub(hi) as i32) >= 0 {
        v = hi;
    }
    v
}

/// The Rust core, the calls through `part` (`PLACES`).
pub unsafe fn start_cars_behind(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if let Some(why) = refusal() {
        return Err(BadInput(why).into());
    }
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    let result = unsafe { run(&mut r, part) };
    match result {
        Ok(()) => Ok(r),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

unsafe fn run(r: &mut Regs, part: Part) -> Result<(), Refused> {
    unsafe {
        if g(0x70FA58) == 0 {
            r.ebx = g(0x6051BC).wrapping_sub(2);
            p(0x6051C4, r.ebx);
            r.ebx = r.ebx.wrapping_add(1);
            p(0x6051C8, r.ebx);
            p(0x6051CC, 0x50);
            p(0x6051D0, 0x28);
            for (start, slot, dist) in [
                (0x6051C4, 0x70FA48, 0x6051CC),
                (0x6051C8, 0x70FA4C, 0x6051D0),
            ] {
                put_car(r, part, g(start), slot, 0x70FA50, g(dist), 0)?;
            }
            p(0x70FA58, 1);
        }
        let count = g(COUNT);
        r.eax = hold_signs(g(0x70FA40), true, count);
        r.eax = hold_signs(r.eax.wrapping_sub(1), true, count);
        r.ebx = g(0x605034 + 4 * r.eax) & 0xF;
        p(0x6051D4, 0x2C + r.ebx);
        p(0x6051D8, 0x28 + r.ebx);
        r.eax = g(0x6051D8).wrapping_add(g(0x6051D4));
        p(0x6051DC, r.eax);
        'moves: {
            if g(0x70FA44) != 0 {
                r.ebx = g(0x70FA48);
                r.esi = block(r.ebx);
                r.eax = at(r.esi + 0x98).wrapping_sub(g(LEAD));
                if (r.eax.wrapping_sub(g(0x6051D4)) as i32) >= 0 {
                    r.eax = g(LEAD).wrapping_sub(g(0x6051D8));
                    part(TAKEN, r)?;
                    if r.carry() {
                        break 'moves;
                    }
                    p(CURRENT, r.ebx);
                    part(LOAD, r)?;
                    part(CLEAR, r)?;
                    part(STORE, r)?;
                    r.eax = g(0x70FA4C);
                    p(0x70FA48, r.eax);
                    r.eax = g(0x70FA54);
                    p(0x70FA50, r.eax);
                    let start = g(COUNT).wrapping_sub(g(0x70FA44));
                    put_car(
                        r,
                        part,
                        start,
                        0x70FA4C,
                        0x70FA54,
                        g(0x6051D8).wrapping_neg(),
                        g(0x5C5D0C),
                    )?;
                    p(0x70FA44, g(0x70FA44).wrapping_sub(1));
                }
            }
            r.eax = g(COUNT).wrapping_sub(3);
            if g(0x70FA44) == r.eax {
                break 'moves;
            }
            r.eax = g(COUNT).wrapping_sub(2).wrapping_sub(g(0x6051C0));
            if g(0x70FA44) >= r.eax {
                break 'moves;
            }
            r.ebx = g(0x70FA4C);
            r.esi = block(r.ebx);
            r.eax = g(LEAD).wrapping_sub(at(r.esi + 0x98));
            if (r.eax.wrapping_sub(g(0x6051D4)) as i32) < 0 {
                break 'moves;
            }
            r.eax = g(LEAD).wrapping_add(g(0x6051D8));
            part(TAKEN, r)?;
            if r.carry() {
                break 'moves;
            }
            p(CURRENT, r.ebx);
            part(LOAD, r)?;
            part(CLEAR, r)?;
            part(STORE, r)?;
            r.eax = g(0x70FA50);
            p(0x70FA54, r.eax);
            r.eax = g(0x70FA48);
            p(0x70FA4C, r.eax);
            let start = g(COUNT).wrapping_sub(3).wrapping_sub(g(0x70FA44));
            put_car(r, part, start, 0x70FA48, 0x70FA50, g(0x6051D8), g(0x5C5D0C))?;
            p(0x70FA44, g(0x70FA44).wrapping_add(1));
        }
        if g(0x70F494) == 0 {
            p(0x70FA14, u32::MAX);
            p(0x70FA18, u32::MAX);
            r.ebp = g(COUNT).wrapping_sub(g(0x70FA44));
            r.eax = g(LEAD);
            r.edx = g(0x70FA10);
            for slot in [0x70FA4C, 0x70FA48] {
                r.ebx = g(slot);
                r.esi = block(r.ebx);
                let d = r.eax.wrapping_sub(at(r.esi + 0x98));
                let ahead = (d as i32) < 0
                    || (d == 0 && (r.edx.wrapping_sub(at(r.esi + 0x624)) as i32) < 0);
                if ahead {
                    part(AHEAD, r)?;
                } else {
                    r.ebp = r.ebp.wrapping_sub(1);
                    part(BEHIND, r)?;
                }
            }
            p(0x70FA40, r.ebp);
            let gap = |r: &mut Regs, other: u32| {
                r.eax = g(0x6051DC).wrapping_sub(g(other));
                if (r.eax.wrapping_sub(g(0x6051D8)) as i32) < 0 {
                    r.eax = g(0x6051D8);
                }
                if (r.eax.wrapping_sub(g(0x6051DC)) as i32) >= 0 {
                    r.eax = g(0x6051DC);
                }
                r.eax
            };
            let mut to_ease = true;
            if g(0x70FA18) == u32::MAX {
                if g(0x70FA44) == 0 {
                    to_ease = false;
                } else {
                    let v = gap(r, 0x70FA14);
                    p(0x70FA18, v);
                }
            }
            if to_ease && g(0x70FA14) == u32::MAX && g(0x70FA40) != 1 {
                let v = gap(r, 0x70FA18);
                p(0x70FA14, v);
            }
            for (wanted, eased) in [(0x70FA18, 0x70FA20), (0x70FA14, 0x70FA1C)] {
                r.eax = g(wanted).wrapping_mul(0x14A).wrapping_sub(g(eased));
                if (r.eax as i32) < 0 {
                    r.eax = r.eax.wrapping_neg();
                }
                if r.eax > 1000 {
                    r.eax = g(wanted).wrapping_mul(0x14A);
                    p(eased, r.eax);
                }
            }
            for (wanted, eased) in [(0x70FA18, 0x70FA20), (0x70FA14, 0x70FA1C)] {
                r.eax = ((g(wanted).wrapping_mul(0x14A).wrapping_sub(g(eased)) as i32) >> 6) as u32;
                p(eased, g(eased).wrapping_add(r.eax));
            }
        }
        for v in [0x70FA60, 0x710A40, 0x711A20, 0x712A00] {
            p(v, 0);
        }
        r.ebx = g(0x70FA48);
        r.esi = block(r.ebx);
        r.ebx = g(0x70FA4C);
        r.edi = block(r.ebx);
        r.eax = at(r.esi + 0x98).wrapping_sub(at(r.edi + 0x98));
        p(0x70FA5C, r.eax);
        if (r.eax.wrapping_sub(8) as i32) >= 0 || (r.eax.wrapping_add(8) as i32) < 0 || r.eax == 0 {
            return Ok(());
        }
        let base: u32 = if (r.eax as i32) > 0 { 8 } else { 0xFFFF_FFF8 };
        r.eax = base.wrapping_sub(g(0x70FA5C));
        let put = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
        put(r.esi + 0x674, r.eax);
        r.eax = r.eax.wrapping_neg();
        put(r.edi + 0x674, r.eax);
        let k = g(0x604EC4);
        for b in [r.esi, r.edi] {
            put(b + 0x674, x87::int_times(at(b + 0x674) as i32, k));
        }
        Ok(())
    }
}

/// The calls in the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            LOAD => crate::drive::load_car::load_car()?.apply(r),
            STORE => crate::drive::store_car::store_car()?.apply(r),
            START => {
                *r = crate::drive::start_car_on_line::start_car_on_line(
                    r,
                    &mut crate::race::car_starts::game_part,
                )?
            }
            TAKEN => crate::drive::checkpoint_taken::checkpoint_taken(r).apply(r),
            CLEAR => crate::drive::clear_start_state::clear_start_state(),
            BEHIND => crate::drive::note_gap_behind::note_gap_behind(r.esi).apply(r),
            _ => crate::drive::note_gap_ahead::note_gap_ahead(r.esi).apply(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { start_cars_behind(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- the self-test's stand-ins ----

static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());
static CALLS: AtomicU32 = AtomicU32::new(0);

/// A stand-in: notes the call, then changes the registers (eax, edx, the carry for
/// `checkpoint_taken`) by the call's number.
fn note(place: u32, r: &mut Regs) {
    let k = CALLS.fetch_add(1, Ordering::SeqCst);
    CALLED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]));
    r.eax = r.eax.rotate_left(5) ^ k.wrapping_mul(0x101);
    r.edx = r.edx.wrapping_add(k);
    if place == TAKEN {
        r.set_carry(k.is_multiple_of(3));
    }
}

fn take_called() -> Vec<(u32, [u32; 7])> {
    CALLS.store(0, Ordering::SeqCst);
    std::mem::take(&mut *CALLED.lock().unwrap())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const CALL_OFFSETS: [usize; 24] = [
        0x5A, 0x6F, 0x74, 0x9C, 0xB1, 0xB6, 0x168, 0x179, 0x17E, 0x183, 0x1C5, 0x1DE, 0x1E3, 0x24B,
        0x258, 0x25D, 0x262, 0x2A7, 0x2BE, 0x2C3, 0x326, 0x32D, 0x352, 0x359,
    ];
    crate::install::stand_ins::set_handler(note);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x43703E)),
            // Through its second chunk (0x4373DC .. 0x4375A5, past `note_gap_ahead` /
            // `behind`): 0x366, IDA's first chunk, left its jumps there going off the copy
            // (the selftest crash of 2026-10-18).
            0x567,
            &[0x83, 0x3D, 0x58, 0xFA, 0x70, 0x00, 0x00],
            &CALL_OFFSETS,
            &|c| {
                PLACES
                    .iter()
                    .position(|&k| rva(k) == c)
                    .map(|n| crate::install::stand_ins::stand_in(n as u32))
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let extra: Vec<Region> = [0x71B3B8u32, 0x604ECC, 0x605034]
        .iter()
        .map(|&x| (game_addr(rva(x as usize)) as u32, 0x170))
        .collect();
    let mut all = regions();
    all.extend(extra);
    let saved = capture(&all);
    // Eight blocks of the test's in place of the game's pointers (the table is saved).
    let mut blocks = vec![0u8; 8 * 0xFE0];
    let base = blocks.as_mut_ptr() as u32;
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for seed in 1..=12u32 {
        let fill = || {
            let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
            let mut next = move || {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
                x >> 8
            };
            for k in 0..8u32 {
                p(0x71B3B8 + 4 * k, base + 0xFE0 * k);
                let b = base + 0xFE0 * k;
                unsafe {
                    ((b + 0x98) as *mut u32).write(100 + next() % 200);
                    ((b + 0x624) as *mut u32).write(next() % 0x1000);
                    ((b + 0x674) as *mut u32).write(next());
                }
            }
            let count = 10 + next() % 20;
            p(COUNT, count);
            for k in 0..0x5A {
                p(CARS_BY_START + 4 * k, next() % 8);
                p(0x605034 + 4 * k, next());
            }
            p(0x70FA58, u32::from(seed % 3 != 0));
            p(0x6051BC, 2 + next() % (count - 2));
            p(0x70FA40, next() % (count + 4));
            p(0x70FA44, next() % 6);
            p(0x6051C0, next() % 4);
            p(0x70FA48, next() % 8);
            p(0x70FA4C, next() % 8);
            p(0x70FA54, next() % count);
            p(0x70FA10, next() % 0x1000);
            p(0x70F494, u32::from(seed % 4 == 0));
            p(LEAD, 100 + next() % 200);
            for v in [0x70FA14, 0x70FA18, 0x70FA1C, 0x70FA20] {
                p(v, next() % 0x200);
            }
        };
        fill();
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            Some(regs(&ro)),
            take_called(),
            capture(&all),
            blocks.clone(),
        );
        fill();
        let mine = unsafe { start_cars_behind(&Regs::SENTINEL, &mut note_part) };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&all),
            blocks.clone(),
        );
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "seed {seed}: original {:x?} {} calls; rust {:x?} {} calls; memory first differs at {diff:x?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len()
            )
        });
    }
    restore(&all, &saved);
    drop((copy, blocks));
    t.finish()
}

fn note_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}
