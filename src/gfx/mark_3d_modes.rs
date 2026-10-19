//! `gfx::mark_3d_modes` (0x40442C): the screen modes Direct3D can draw in, marked in the
//! modes' table.
//!
//! In:  eax = the kind of card the modes were listed for (0x1000 the primary, 0x2000 the
//!      secondary: `gfx::list_primary_modes`, `gfx::list_secondary_modes`)
//! Out: every register preserved (`pusha` .. `popa`)
//! Nothing unless `g_hasFpu` (0x5183EC, set by `system::test_processor`) is 1.
//! `sub_419853` (through `part`: Direct3D's devices looked at, "D3D: detect card", the
//! depth formats into `dword_4F492C` and `dword_4F4930`; the carry: none fit) the
//! platform's, through the game; in the standalone program it says none fit. Then over the
//! table `dword_4F4F3C` (`dword_4F4F38` entries of 8 bytes: flags, a word of extra flags):
//! the next entry with all of eax's bits (`system::find_entry_with_bits`), from there the
//! next with any of `dword_4F492C`'s (`system::find_entry_with_any_bit`); that one gets
//! 0x100000 and, with `dword_4F4930` not 0, 0x200000 and `dword_4F4930` in its second
//! dword; one entry on (`system::next_entry`, its carry not looked at), again, until a
//! search finds nothing. Refuses (before anything) more than 0x20 entries, the table's
//! room (`sub_404A56` stops there): past it the original would mark what lies after.
//! Shadow mode runs the original alone (the platform's call is made once). Self-test: a
//! copy whose five variables are the test's and whose Direct3D call goes to a stand-in
//! (which sets the depth formats, or fails), on tables of the test's, against the Rust
//! version.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow, standalone,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::system::find_entry_with_any_bit::find_entry_with_any_bit;
use crate::system::find_entry_with_bits::find_entry_with_bits;
use crate::system::next_entry::next_entry;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::mark_3d_modes", rva(0x40442C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// The call: 0 `sub_419853` (the carry: no Direct3D device fits).
const DETECT_RVA: usize = rva(0x419853);
/// The table's room, in entries.
pub const MODES_ROOM: u32 = 0x20;
const DRAWS_3D: u32 = 0x10_0000;
const HAS_EXTRA: u32 = 0x20_0000;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ModeVars {
    /// g_hasFpu
    pub has_fpu: u32,
    /// dword_4F4F3C: the table
    pub table: u32,
    /// dword_4F4F38: its entries
    pub count: u32,
    /// dword_4F492C: the depth formats' bits
    pub depth: u32,
    /// dword_4F4930: the extra flags
    pub extra: u32,
}

impl ModeVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            has_fpu: a(0x5183EC),
            table: a(0x4F4F3C),
            count: a(0x4F4F38),
            depth: a(0x4F492C),
            extra: a(0x4F4930),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn or(at: u32, bits: u32) {
    unsafe { (at as *mut u32).write_unaligned(get(at) | bits) };
}

/// The marking over the table (after the platform's call).
pub unsafe fn mark_modes(kind: u32, v: &ModeVars) {
    unsafe {
        let (mut at, mut left) = (v.table, get(v.count));
        loop {
            let (found, e, l) = find_entry_with_bits(kind, at, left);
            if !found {
                return;
            }
            let (found, e, l) = find_entry_with_any_bit(get(v.depth), e, l);
            if !found {
                return;
            }
            or(e, DRAWS_3D);
            let extra = get(v.extra);
            if extra != 0 {
                or(e, HAS_EXTRA);
                or(e + 4, extra);
            }
            (_, at, left) = next_entry(e, l);
        }
    }
}

pub unsafe fn mark_3d_modes(entry: &Regs, v: &ModeVars, part: Part) -> Result<(), Refused> {
    unsafe {
        if get(v.has_fpu) != 1 {
            return Ok(());
        }
        if get(v.count) > MODES_ROOM {
            return Err(BadInput("more screen modes than the table has room for").into());
        }
        let mut r = *entry;
        part(0, &mut r)?;
        if r.carry() {
            return Ok(());
        }
        mark_modes(entry.eax, v);
        Ok(())
    }
}

/// In the game: Direct3D through the game; in the standalone program none fits.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    if standalone() {
        r.set_carry(true);
        return Ok(());
    }
    unsafe { call_game(r, game_addr(DETECT_RVA)) };
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { mark_3d_modes(&entry, &ModeVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

static CALLS: AtomicU32 = AtomicU32::new(0);
static FAILS: AtomicU32 = AtomicU32::new(0);
/// The test's depth and extra variables, and what the stand-in puts in them.
static DEPTH_AT: AtomicU32 = AtomicU32::new(0);
static DEPTH: AtomicU32 = AtomicU32::new(0);
static EXTRA: AtomicU32 = AtomicU32::new(0);

fn note(_place: u32, r: &mut Regs) {
    CALLS.fetch_add(1, Ordering::SeqCst);
    let at = DEPTH_AT.load(Ordering::SeqCst);
    unsafe {
        (at as *mut u32).write_volatile(DEPTH.load(Ordering::SeqCst));
        ((at + 4) as *mut u32).write_volatile(EXTRA.load(Ordering::SeqCst));
    }
    r.set_carry(FAILS.load(Ordering::SeqCst) != 0);
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    act(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // has_fpu, count, depth, extra, then the table.
    let mut m = vec![0u32; 4 + 2 * MODES_ROOM as usize];
    let at = m.as_mut_ptr() as u32;
    let v = ModeVars {
        has_fpu: at,
        count: at + 4,
        depth: at + 8,
        extra: at + 12,
        table: at + 16,
    };
    DEPTH_AT.store(v.depth, Ordering::SeqCst);
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40442C,
            len: 0x55,
            entry: &[0x60, 0x83, 0x3D, 0xEC, 0x83, 0x51, 0x00, 0x01],
            calls: &[0x0C, 0x21, 0x2D, 0x4C],
            jumps_out: &[],
        },
        &|callee| (callee == DETECT_RVA).then(|| stand_in(0)),
        &[
            (0x5183EC, v.has_fpu),
            (0x4F4F3C, v.table),
            (0x4F4F38, v.count),
            (0x4F492C, v.depth),
            (0x4F4930, v.extra),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(note);
    // The table's entries: kinds 0x1000 / 0x2000 and depth bits 1 .. 8, by lot.
    let table: Vec<u32> = (0..MODES_ROOM)
        .flat_map(|n| {
            let kind = [0x1000u32, 0x2000, 0x3000, 0][(n % 4) as usize];
            [kind | (n.wrapping_mul(0x9E37) >> 3) & 0xF | (n << 24), n]
        })
        .collect();
    // (has fpu, count, kind, fails, depth, extra)
    let cases = [
        (0u32, 8u32, 0x1000u32, 0u32, 1u32, 0u32),
        (1, 8, 0x1000, 1, 1, 0),
        (1, 8, 0x1000, 0, 1, 0),
        (1, 0x20, 0x1000, 0, 3, 0x40),
        (1, 0x20, 0x2000, 0, 8, 0),
        (1, 0x20, 0x2000, 0, 0, 0x7),
        (1, 0, 0x1000, 0, 0xF, 0x1),
        (1, 13, 0x1000, 0, 0x4, 0x100),
        (1, 1, 0x3000, 0, 0xF, 0),
    ];
    for (fpu, count, kind, fails, depth, extra) in cases {
        FAILS.store(fails, Ordering::SeqCst);
        DEPTH.store(depth, Ordering::SeqCst);
        EXTRA.store(extra, Ordering::SeqCst);
        let run = |rust: bool| unsafe {
            let p = at as *mut u32;
            p.write_volatile(fpu);
            p.add(1).write_volatile(count);
            p.add(2).write_volatile(0xAAAA_AAAA);
            p.add(3).write_volatile(0xAAAA_AAAA);
            for (n, x) in table.iter().enumerate() {
                p.add(4 + n).write_volatile(*x);
            }
            CALLS.store(0, Ordering::SeqCst);
            let before = Regs {
                eax: kind,
                ..Regs::SENTINEL
            };
            let (done, preserved) = if rust {
                (mark_3d_modes(&before, &v, &mut test_part).is_ok(), true)
            } else {
                let mut ro = before;
                call_game(&mut ro, c.addr());
                (true, ro.preserved_except(&before, 0))
            };
            (
                done && preserved,
                CALLS.load(Ordering::SeqCst),
                std::slice::from_raw_parts(at as *const u32, m_len()).to_vec(),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs == mine, || {
            format!(
                "fpu {fpu}, {count} modes, kind {kind:#x}, fails {fails}, depth {depth:#x}, extra {extra:#x}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    drop(m);
    t.finish()
}

fn m_len() -> usize {
    4 + 2 * MODES_ROOM as usize
}
