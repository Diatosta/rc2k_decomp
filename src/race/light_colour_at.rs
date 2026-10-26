//! `race::light_colour_at` (0x4537CE): the light at the lit place (`g_litPlace`,
//! `dword_63658C`, `dword_636590`) into `g_lightColour`: 0, then the cars' headlights
//! (`sub_44F601`, each car by `sub_44F691`: the place turned into the car's light frame
//! (+0x9CC, `fistp`'d into `dword_6365B8` ..), a spread by distance and side, times +0xABC,
//! a colour from `g_carLightTable`) and the lamps (`sub_44F863`: by `scene::approximate_distance`,
//! times the lamp's strength, a grey times its colour), each added byte by byte (held to
//! 0xFF; three bytes without MMX, four with, as the original's two paths).
//!
//! In:  nothing (the place)
//! Out: the registers as it leaves them (ecx the cars, esi, eax, ebx, edx as the last car
//!      and lamp left them)
//!
//! `sub_44F601`, `sub_44F691` and `sub_44F863` are covered here (their only callers are in
//! this tree). The car's second spread (from z' - 0x100000) is worked out and not used, as
//! the original (`notes/bugs.md`). See `race/frame_draw.rs`. Callers: `race::view_light`,
//! 0x4537B3, 0x453A35, 0x453C05, 0x453DD5. Refuses (before anything) no race cars or more
//! than eight, a null block, more than 16 lamps. Shadow mode: by state. Self-test: the
//! game's blocks given lights, places and frames by pattern, test lamps (points of the
//! test's), with and without MMX, the modes and player counts; all put back (skipped with a
//! null block).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::light_colour_at", rva(0x4537CE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn colour_regions() -> Vec<Region> {
    vec![
        at(LIGHT_COLOUR_RVA),
        (game_addr(rva(0x6365B8)) as u32, 0xC),
        at(rva(0x62400C)),
        at(rva(0x639958)),
        at(rva(0x62DA7C)),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { light_colour_at(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &colour_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// A lit test scene: the game's blocks with lights set by `seed`, test lamps on points of
/// the test's.
pub struct LightStage {
    pub points: Vec<u32>,
    pub kept: Vec<Region>,
    pub saved: Vec<u8>,
    pub blocks: Vec<u32>,
}

impl LightStage {
    pub fn new(extra: &[Region]) -> Option<Self> {
        let blocks: Vec<u32> = (0..8)
            .map(|n| unsafe { game_read::<u32>(CAR_BLOCKS_RVA + 4 * n) })
            .collect();
        if blocks.contains(&0) {
            return None;
        }
        let mut kept = colour_regions();
        kept.extend_from_slice(extra);
        for a in [
            0x5CCCC0usize,
            0x5183A8,
            0x63DC48,
            0x52A910,
            0x52A920,
            0x6238B8,
            0x63DB94,
            0x62389C,
            0x63DB9C,
            0x63DBA8,
            0x63DBAC,
            0x636588,
            0x63658C,
            0x636590,
        ] {
            kept.push(at(rva(a)));
        }
        kept.push(at(CARS_RVA));
        kept.push((game_addr(rva(0x63E05C)) as u32, 0x100));
        for &b in &blocks {
            kept.extend([
                (b + 0x84, 4),
                (b + 0x134, 0xC),
                (b + 0x9CC, 0x30),
                (b + 0xABC, 4),
            ]);
        }
        let saved = capture(&kept);
        let points: Vec<u32> = (0..16u32 * 3)
            .map(|k| k.wrapping_mul(0x2F1D3) % 0x30_0000)
            .collect();
        Some(Self {
            points,
            kept,
            saved,
            blocks,
        })
    }

    /// Everything put back, then the case in place.
    pub fn prepare(&self, c: &LightCase) {
        let LightCase {
            seed,
            mmx,
            cars,
            lamps,
            players,
            mode,
            one,
            all,
        } = *c;
        restore(&self.kept, &self.saved);
        unsafe {
            let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
            game_write(rva(0x5CCCC0), self.points.as_ptr() as u32);
            game_write(rva(0x5183A8), mmx);
            game_write(CARS_RVA, cars);
            game_write(rva(0x63DC48), lamps);
            game_write(rva(0x63DB94), players);
            game_write(rva(0x62389C), mode);
            game_write(rva(0x52A910), one);
            game_write(rva(0x52A920), 2u32);
            game_write(rva(0x6238B8), all);
            game_write(rva(0x63DB9C), 1u32);
            game_write(rva(0x63DBAC), 1u32);
            game_write(rva(0x636588), 0x10_0000u32);
            game_write(rva(0x63658C), 0x2_0000u32);
            game_write(rva(0x636590), 0x18_0000u32);
            let lamp = game_addr(rva(0x63E05C)) as u32;
            for k in 0..16u32 {
                w(lamp + 16 * k, k);
                w(
                    lamp + 16 * k + 4,
                    if k % 4 == 1 { 0 } else { 0x40 + k * 13 },
                );
                w(lamp + 16 * k + 8, 0x00C0_8040 ^ (k * 0x10101) ^ seed);
            }
            for (n, &b) in self.blocks.iter().enumerate() {
                let s = seed.wrapping_mul(n as u32 * 2 + 1);
                w(b + 0x84, u32::from(n % 3 != 2));
                w(b + 0x134, 0x0F_0000 + (s % 0x4_0000));
                w(b + 0x138, s % 0x1_0000);
                w(b + 0x13C, 0x17_0000 + (s.rotate_left(9) % 0x4_0000));
                let a = (s % 628) as f32 / 100.0;
                let m = [
                    a.cos(),
                    0.0,
                    -a.sin(),
                    0.0,
                    1.0,
                    0.0,
                    a.sin(),
                    0.0,
                    a.cos(),
                    0.0,
                    0.0,
                    0.0,
                ];
                for (k, v) in m.iter().enumerate() {
                    w(b + 0x9CC + 4 * k as u32, v.to_bits());
                }
                w(b + 0xABC, 0x80 + s % 0x100);
            }
        }
    }

    pub fn end(self) {
        restore(&self.kept, &self.saved);
    }
}

/// One case of the light tests: `seed` for the blocks, MMX, cars, lamps, players, mode,
/// the chosen-car switch and `dword_6238B8`.
#[derive(Clone, Copy, Debug)]
pub struct LightCase {
    pub seed: u32,
    pub mmx: u32,
    pub cars: u32,
    pub lamps: u32,
    pub players: u32,
    pub mode: u32,
    pub one: u32,
    pub all: u32,
}

pub const LIGHT_CASES: [LightCase; 6] = [
    LightCase {
        seed: 1,
        mmx: 1,
        cars: 8,
        lamps: 6,
        players: 2,
        mode: 1,
        one: 0,
        all: 0,
    },
    LightCase {
        seed: 7,
        mmx: 0,
        cars: 8,
        lamps: 6,
        players: 2,
        mode: 1,
        one: 0,
        all: 0,
    },
    LightCase {
        seed: 3,
        mmx: 1,
        cars: 5,
        lamps: 0,
        players: 1,
        mode: 0,
        one: 1,
        all: 0,
    },
    LightCase {
        seed: 5,
        mmx: 0,
        cars: 3,
        lamps: 16,
        players: 2,
        mode: 2,
        one: 0,
        all: 1,
    },
    LightCase {
        seed: 9,
        mmx: 0,
        cars: 8,
        lamps: 3,
        players: 8,
        mode: 1,
        one: 1,
        all: 1,
    },
    LightCase {
        seed: 11,
        mmx: 1,
        cars: 1,
        lamps: 1,
        players: 1,
        mode: 3,
        one: 0,
        all: 0,
    },
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Some(stage) = LightStage::new(&[]) else {
        return t.skip("a car block is null (no race loaded)");
    };
    let regions = colour_regions();
    for c in &LIGHT_CASES {
        check_state(
            &mut t,
            &format!("{c:?}"),
            call_original,
            Regs::SENTINEL,
            &|| stage.prepare(c),
            &regions,
            &rust,
        );
    }
    stage.end();
    t.finish()
}
