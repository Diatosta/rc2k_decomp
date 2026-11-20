//! The probe points (`probe_count` of them) against walls. For each point k, its face now
//! (`probe_point_face[k]`, into `wall_face_now`) and before (`probe_saved_faces[k]`, into
//! `wall_face_before` when they differ): their flag words are or-ed together; when they differ and
//! the pair's has both 0x400 and 0x800, the wall is met: `probe_index` = k, the point goes from
//! `probe_point_x` into `contact_point`, then `physics::wall_edge_contact`, `contact_depth` raised
//! to |`contact_speed_length`|, `contact_flag` = 1, `wall_hit_timer` = 0x32,
//! `physics::restore_saved_car`, `physics::feel_ground`. None met: `wall_face_now` =
//! `wall_face_before`, the faces' words or-ed & 0xC00 into `wall_bits`; at 0xC00, the car-frame
//! speed (`local_velocity`) goes into `contact_point`, then `physics::wall_push_back`, the same two
//! words, `restore_saved_car`, `feel_ground`.
//!
//! Refuses (before anything) a count of 0 or past 0x40, and what its calls refuse (what was written
//! is put back).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::step_families::ContactInputs;
use crate::race::owner_course::Course;
use rc2k_core::physics::walls;

fn word(course: &Course, face: u32) -> Result<u32, Refused> {
    Ok(u32::from(crate::race::course_places::face_field::<u16>(
        course.reads(),
        face,
        2,
    )?))
}

fn after(c: &ContactInputs<'_>) -> Result<(), Refused> {
    c.car.contact_flag.set(1);
    c.car.wall_hit_timer.set(0x32);
    { crate::physics::restore_saved_car::restore_saved_car(c.car, c.tables, c.contact) }?;
    crate::physics::feel_ground::feel_ground(&c.ground)
}

/// The steps, not guarded (`check` before).
// Core: `rc2k_core::physics::walls::walk`.
pub fn steps(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let contact = c.contact;
    let walked = walls::walk(c.car.probe_count.get(), |k| {
        let k = k as usize;
        let now = c.car.probe_point_face[k].get();
        contact.wall_face_now.set(now);
        let now_word = word(c.course, now)?;
        let before = contact.probe_saved_faces[k].get();
        let before_word = if now != before {
            contact.wall_face_before.set(before);
            Some(word(c.course, before)?)
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
            // A wall met.
            let k = probe as usize;
            contact.probe_index.set(probe);
            c.car.contact_point[0].set(c.car.probe_point_x[k].get());
            c.car.contact_point[1].set(c.car.probe_point_y[k].get());
            c.car.contact_point[2].set(c.car.probe_point_z[k].get());
            crate::physics::wall_edge_contact::steps(c, scene)?;
            if let Some(hit) = rc2k_core::physics::walls::raised_hit(
                contact.contact_speed_length.get(),
                c.car.contact_depth.get(),
            ) {
                c.car.contact_depth.set(hit);
            }
            after(c)
        }
        walls::Walk::Clear { bits } => {
            contact.wall_face_now.set(contact.wall_face_before.get());
            contact.wall_bits.set(bits);
            if bits != walls::WALL_BITS {
                return Ok(());
            }
            c.car.contact_point[0].set(c.car.local_velocity[0].get());
            c.car.contact_point[1].set(c.car.local_velocity[1].get());
            c.car.contact_point[2].set(c.car.local_velocity[2].get());
            crate::physics::wall_push_back::steps(c, scene)?;
            after(c)
        }
    }
}

/// The probe count checked: 0 or past 0x40 refused.
pub fn check(car: &Car) -> Result<(), Refused> {
    Ok(walls::check_probe_count(car.probe_count.get())?)
}

pub fn wall_contacts(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    check(c.car)?;
    steps(c, scene)
}
