//! `race::init_overlay` (0x45A9DE): the overlay's stage parts (`initoverlay`).
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
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
//! One caller, `sub_44426F` ("loadlevel_init : initoverlay"; no `pusha`). Refuses more than
//! 0x40 parts (the original writes past its three lists).

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::math::x87;
use crate::race::numbers_store::NUMBERS;
use crate::race::overlay_store::RACE_OVERLAY;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;

/// The overlay's variables: 0x6D1460 .. 0x6D180C (unaligned, packed).
const BLOCK: usize = 0x6D1460;
const MILES: usize = 0x6D1466; // g_overlayMiles
const PARTS: usize = 0x6D1472; // g_overlayParts
const CLEARED: [usize; 2] = [0x6D146A, 0x6D146E]; // flt_6D146A, flt_6D146E
/// The sizes (`rc2k_core::race::panel::OVERLAY_SIZES`): dword_6D1486 .. dword_6D149E.
const SIZES: [usize; 6] = [0x6D1486, 0x6D148A, 0x6D148E, 0x6D1496, 0x6D149A, 0x6D149E];
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
            kinds: (RACE_VARS.checkpoint_count.addr() as u32),
            found: a(0x601830),
            list: a(0x60102C),
            distances: a(0x9F78D8),
            scratch: (DRAW.scratch.addr() as u32),
            block: a(BLOCK),
            car_blocks: (RACE_WORK.car_blocks[0].addr() as u32),
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
pub unsafe fn init_overlay(v: &OverlayVars) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::panel::{overlay_parts, half_length, OVERLAY_SIZES,
        // OVERLAY_CLEARED}`; the miles the x87's.
        use rc2k_core::race::panel::{OVERLAY_CLEARED, OVERLAY_SIZES, half_length, overlay_parts};
        let (found, parts) = overlay_parts(get(v.kinds), || get(v.found))?;
        if let Some(count) = found {
            put(v.of(PARTS), count);
        }
        match parts {
            Some(count) => {
                put(v.of(MILES), 0);
                let (scale, mile): (u32, u32) =
                    (NUMBERS.k65536.bits(), NUMBERS.metres_in_mile.bits());
                let distance = |k: u32| get(v.distances.wrapping_add(k.wrapping_mul(4)));
                for n in 0..count {
                    let end = distance(get(v.list + 4 * n + 4));
                    let start = distance(get(v.list + 4 * n));
                    let half = half_length(end, start);
                    put(v.scratch, half);
                    let miles = x87::int_div_div_doubled(half as i32, scale, mile);
                    RACE_OVERLAY.overlay_part_miles[n as usize].set_bits(miles); // g_overlayPartMiles
                    put(v.of(MILES), x87::add(miles, get(v.of(MILES))));
                    RACE_OVERLAY.overlay_part_kinds_a[n as usize].set(0); // dword_6D160C
                    RACE_OVERLAY.overlay_part_kinds_b[n as usize].set(0); // dword_6D170C
                }
            }
            None => {
                put(v.of(PARTS), 1);
                RACE_OVERLAY.overlay_part_miles[0].set_bits(ONE);
                put(v.of(MILES), ONE);
                RACE_OVERLAY.overlay_part_kinds_a[0].set(0);
                RACE_OVERLAY.overlay_part_kinds_b[0].set(0);
            }
        }
        for (&variable, value) in SIZES.iter().zip(OVERLAY_SIZES) {
            put(v.of(variable), value);
        }
        for n in 0..CARS {
            let block = get(v.car_blocks + 4 * n);
            for off in OVERLAY_CLEARED {
                put(block + off, 0);
            }
        }
        for variable in CLEARED {
            put(v.of(variable), 0);
        }
    }
    Ok(())
}
