//! `physics::wall_contacts` (0x45D7B6): the probe points (`unk_71A238` of them) against
//! walls. For each point k, its face now (`unk_71ADC8[k]`, into `dword_6D3174`) and before
//! (`dword_6D2814[k]`, into `dword_6D3170` when they differ): their words +2 or-ed
//! together; when they differ and the pair's has both 0x400 and 0x800, the wall met:
//! `dword_6D3214` = k, the point's three floats from `unk_71AAC8[k]` .. into
//! `unk_71A1FC` .., `physics::wall_edge_contact`, `unk_71A208` raised to |`flt_6D3108`|,
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32, `physics::restore_saved_car`,
//! `physics::feel_ground`. None met: `dword_6D3174` = `dword_6D3170`, the faces' words
//! or-ed & 0xC00 into `dword_6D3038`; at 0xC00, the car-frame speed (`unk_71A17C` ..)
//! into `unk_71A1FC` .., `physics::wall_push_back`, the same two words,
//! `restore_saved_car`, `feel_ground`.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them (ebp the point, or -1 after the
//!      loop); no flags read (callers go on)
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. Refuses (before anything) a
//! count of 0 or past 0x40 (the original's loop runs on, or past the tables), and what its
//! calls refuse (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::flow::{g, p};
use crate::physics::touch_face::FACES;
use rc2k_core::physics::walls;

/// `g_workVector`.
pub const WORK: usize = 0x5C5F60;
/// Contact record 4 (`g_contactRecords` + 0x80).
pub const WALL_RECORD: usize = 0x614AD0;

fn word(face: u32) -> u32 {
    let at = (face << 6).wrapping_add(g(FACES)) + 2;
    unsafe { (at as *const u16).read_unaligned() as u32 }
}

fn after() -> Result<(), Refused> {
    CAR.contact_flag.set(1);
    CAR.wall_hit_timer.set(0x32);
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?;
    unsafe { crate::physics::feel_ground::feel_ground() }
}

/// The steps, not guarded (`check` before). The registers the original leaves are read by
/// none of its callers.
// Converted: `rc2k_core::physics::walls::walk`.
pub fn steps() -> Result<(), Refused> {
    let walked = walls::walk(CAR.probe_count.get(), |k| {
        let k = k as usize;
        let now = CAR.probe_point_face[k].get();
        p(0x6D3174, now);
        let now_word = word(now);
        let before = g(0x6D2814 + 4 * k);
        let before_word = (now != before).then(|| {
            p(0x6D3170, before);
            word(before)
        });
        Ok::<_, Refused>(walls::ProbeFaces {
            now_word,
            before_word,
        })
    })?;
    match walked {
        walls::Walk::Met { probe, .. } => {
            // 0x45D83E: a wall met.
            let k = probe as usize;
            p(0x6D3214, probe);
            CAR.contact_point[0].set_bits(CAR.probe_point_x[k].bits());
            CAR.contact_point[1].set_bits(CAR.probe_point_y[k].bits());
            CAR.contact_point[2].set_bits(CAR.probe_point_z[k].bits());
            crate::physics::wall_edge_contact::steps()?;
            if let Some(hit) = rc2k_core::physics::walls::raised_hit(
                f32::from_bits(g(0x6D3108)),
                f32::from_bits(CAR.contact_depth.bits()),
            ) {
                CAR.contact_depth.set_bits(hit.to_bits());
            }
            after()
        }
        walls::Walk::Clear { bits } => {
            p(0x6D3174, g(0x6D3170));
            p(0x6D3038, bits);
            if bits != walls::WALL_BITS {
                return Ok(());
            }
            CAR.contact_point[0].set_bits(crate::physics::car_store::CAR.local_velocity[0].bits());
            CAR.contact_point[1].set_bits(crate::physics::car_store::CAR.local_velocity[1].bits());
            CAR.contact_point[2].set_bits(crate::physics::car_store::CAR.local_velocity[2].bits());
            crate::physics::wall_push_back::steps()?;
            after()
        }
    }
}

/// The probe count checked: 0 or past 0x40 refused.
pub fn check() -> Result<(), Refused> {
    Ok(walls::check_probe_count(CAR.probe_count.get())?)
}

/// The Rust version.
pub fn wall_contacts() -> Result<(), Refused> {
    check()?;
    steps()
}
