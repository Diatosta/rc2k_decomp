//! `race::init_overlay` (0x45A9DE): the overlay's stage parts (`initoverlay`).
//!
//! In:  nothing
//! Out: edx 8, ecx 8, esi the eighth car's block; with the parts made, ebp their count,
//!      eax the last part's difference, ebx its first checkpoint; with two kinds or more but
//!      no parts, ebp `dword_601830`; the rest preserved
//!
//! The parts are the stretches between the checkpoints `race::init_checkpoints` listed
//! (`dword_60102C`, `dword_601830` of them): each one's length, the difference of its two
//! checkpoints' `g_checkpointDistances` halved (`shr`, so a negative one comes out huge) /
//! 65536 / 1609.344 (metres in a mile) and doubled, into `g_overlayPartMiles` (floats, 0x40),
//! their sum into `g_overlayMiles`, `g_overlayParts` their count, the two dwords of each
//! part at `dword_6D160C` and `dword_6D170C` 0. With fewer than two checkpoint kinds or no
//! parts, one part of 1.0. Then six sizes (0x40, 0x80, 0x50, twice), each car block's
//! +0xF78 .. +0xF98 and +0xFA0 0, `flt_6D146A` and `flt_6D146E` 0. The x87 steps through
//! `math::x87`; `dword_62400C` left as the original's `fild` leaves it.
//!
//! One caller, `sub_44426F` ("loadlevel_init : initoverlay"; no `pusha`). Refuses more
//! than 0x40 parts (the original writes past its three lists). Shadow mode compares the
//! overlay's variables, `dword_62400C` and the car blocks' dwords. Self-test: a copy of the
//! original whose variables (32 references) are the test's, against the Rust version on
//! the same test variables.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The overlay's variables: 0x6D1460 .. 0x6D180C (unaligned, packed).
const BLOCK: usize = 0x6D1460;
const MILES: usize = 0x6D1466; // g_overlayMiles
const PARTS: usize = 0x6D1472; // g_overlayParts
const PART_MILES: usize = 0x6D150C; // g_overlayPartMiles
const PART_A: usize = 0x6D160C; // dword_6D160C
const PART_B: usize = 0x6D170C; // dword_6D170C
const CLEARED: [usize; 2] = [0x6D146A, 0x6D146E]; // flt_6D146A, flt_6D146E
/// The sizes (`rc2k_core::race::panel::OVERLAY_SIZES`): dword_6D1486 .. dword_6D149E.
const SIZES: [usize; 6] = [0x6D1486, 0x6D148A, 0x6D148E, 0x6D1496, 0x6D149A, 0x6D149E];
const SCALE_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0
const MILE_RVA: usize = rva(0x5C5CF0); // g_metresInMile, 1609.344
const CARS: u32 = 8;
const ONE: u32 = 0x3F80_0000;

/// Where the variables are: the game's, or a test's (the overlay's block as one piece).
pub struct OverlayVars {
    pub kinds: u32,
    pub found: u32,
    pub list: u32,
    pub distances: u32,
    pub scratch: u32,
    pub block: u32,
    pub car_blocks: u32,
}

impl OverlayVars {
    pub fn game() -> Self {
        let a = |v: usize| game_addr(rva(v)) as u32;
        Self {
            kinds: a(0x623640),
            found: a(0x601830),
            list: a(0x60102C),
            distances: a(0x9F78D8),
            scratch: a(0x62400C),
            block: a(BLOCK),
            car_blocks: a(0x71B3B8),
        }
    }

    fn of(&self, variable: usize) -> u32 {
        self.block + (variable - BLOCK) as u32
    }
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// `race::init_overlay` (see the header).
pub unsafe fn init_overlay(entry: &Regs, v: &OverlayVars) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        // Converted: `rc2k_core::race::panel::{overlay_parts, half_length, OVERLAY_SIZES,
        // OVERLAY_CLEARED}`; the miles the x87's.
        use rc2k_core::race::panel::{OVERLAY_CLEARED, OVERLAY_SIZES, half_length, overlay_parts};
        let (found, parts) = overlay_parts(get(v.kinds), || get(v.found))?;
        if let Some(count) = found {
            g.ebp = count;
            put(v.of(PARTS), count);
        }
        match parts {
            Some(count) => {
                put(v.of(MILES), 0);
                let (scale, mile): (u32, u32) = (game_read(SCALE_RVA), game_read(MILE_RVA));
                for n in 0..count {
                    g.ebx = get(v.list + 4 * n + 4);
                    g.eax = get(v.distances.wrapping_add(g.ebx.wrapping_mul(4)));
                    g.ebx = get(v.list + 4 * n);
                    let (end, start) =
                        (g.eax, get(v.distances.wrapping_add(g.ebx.wrapping_mul(4))));
                    g.eax = end.wrapping_sub(start);
                    let half = half_length(end, start);
                    put(v.scratch, half);
                    let miles = x87::int_div_div_doubled(half as i32, scale, mile);
                    put(v.of(PART_MILES) + 4 * n, miles);
                    put(v.of(MILES), x87::add(miles, get(v.of(MILES))));
                    put(v.of(PART_A) + 4 * n, 0);
                    put(v.of(PART_B) + 4 * n, 0);
                }
            }
            None => {
                put(v.of(PARTS), 1);
                put(v.of(PART_MILES), ONE);
                put(v.of(MILES), ONE);
                put(v.of(PART_A), 0);
                put(v.of(PART_B), 0);
            }
        }
        for (&variable, value) in SIZES.iter().zip(OVERLAY_SIZES) {
            put(v.of(variable), value);
        }
        for n in 0..CARS {
            g.esi = get(v.car_blocks + 4 * n);
            for off in OVERLAY_CLEARED {
                put(g.esi + off, 0);
            }
        }
        g.ecx = 8;
        g.edx = CARS;
        for variable in CLEARED {
            put(v.of(variable), 0);
        }
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        init_overlay(r, &OverlayVars::game())
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
