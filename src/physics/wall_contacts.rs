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
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use rc2k_core::physics::walls;

fn word(face: u32) -> Result<u32, Refused> {
    Ok(u32::from(crate::race::course_places::face_field::<u16>(
        face,
        2,
        "a face's flags",
    )?))
}

fn after(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    race.shared.car.contact_flag.set(1);
    race.shared.car.wall_hit_timer.set(0x32);
    { crate::physics::restore_saved_car::restore_saved_car(race) }?;
    crate::physics::feel_ground::feel_ground(race, scene)
}

/// The steps, not guarded (`check` before). The registers the original leaves are read by
/// none of its callers.
// Converted: `rc2k_core::physics::walls::walk`.
pub fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics_store = &race.physics;
    let walked = walls::walk(race.shared.car.probe_count.get(), |k| {
        let k = k as usize;
        let now = race.shared.car.probe_point_face[k].get();
        physics_store.contact.wall_face_now.set(now);
        let now_word = word(now)?;
        let before = physics_store.contact.probe_saved_faces[k].get();
        let before_word = if now != before {
            physics_store.contact.wall_face_before.set(before);
            Some(word(before)?)
        } else {
            None
        };
        Ok::<_, Refused>(walls::ProbeFaces {
            now_word,
            before_word,
        })
    })?;
    match walked {
        walls::Walk::Met { probe, .. } => {
            // 0x45D83E: a wall met.
            let k = probe as usize;
            physics_store.contact.probe_index.set(probe);
            race.shared.car.contact_point[0].set_bits(race.shared.car.probe_point_x[k].bits());
            race.shared.car.contact_point[1].set_bits(race.shared.car.probe_point_y[k].bits());
            race.shared.car.contact_point[2].set_bits(race.shared.car.probe_point_z[k].bits());
            crate::physics::wall_edge_contact::steps(race, scene)?;
            if let Some(hit) = rc2k_core::physics::walls::raised_hit(
                f32::from_bits(physics_store.contact.contact_speed_length.bits()),
                f32::from_bits(race.shared.car.contact_depth.bits()),
            ) {
                race.shared.car.contact_depth.set_bits(hit.to_bits());
            }
            after(race, scene)
        }
        walls::Walk::Clear { bits } => {
            physics_store
                .contact
                .wall_face_now
                .set(physics_store.contact.wall_face_before.get());
            physics_store.contact.wall_bits.set(bits);
            if bits != walls::WALL_BITS {
                return Ok(());
            }
            race.shared.car.contact_point[0].set_bits(race.shared.car.local_velocity[0].bits());
            race.shared.car.contact_point[1].set_bits(race.shared.car.local_velocity[1].bits());
            race.shared.car.contact_point[2].set_bits(race.shared.car.local_velocity[2].bits());
            crate::physics::wall_push_back::steps(race, scene)?;
            after(race, scene)
        }
    }
}

/// The probe count checked: 0 or past 0x40 refused.
pub fn check(shared: &Shared) -> Result<(), Refused> {
    Ok(walls::check_probe_count(shared.car.probe_count.get())?)
}

/// The Rust version.
pub fn wall_contacts(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    check(&race.shared)?;
    steps(race, scene)
}
