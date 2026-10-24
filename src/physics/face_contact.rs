//! `physics::face_contact` (0x45E3BF): one course face (ebx, `g_courseFaces` + 0x40 n)
//! tried against the work car. `dword_6D3218` = n. Faces hidden by kind
//! (`scene::kind_hidden_at`) and faces whose first byte is not 0xFF (not an object's)
//! are left. An object byte (+1) of 0x3B: the kind 0xFF, `physics::touch_face`. Else the
//! object (`dword_6D321C`), its contact kind `dword_6D3624` from its material record
//! (`g_objectMaterials` + 0x80 `g_objectKinds[n]`, byte +0x1C); left when byte +0x25
//! bit 0 is set, when its bits 1 .. 2 (a direction + 1) are not 0 and not
//! `g_stageDirection` + 1, or when the object is 0x38 or more but for 0x3A. Its turn
//! (+8, +0xC, +0x10 bytes << 8 into `dword_63503C`, `635040`, `635044`) and scale (+0x18
//! into `dword_635038`); `physics::scene_point_hits_car` (ebp the face). Not inside: for
//! kinds 9, 8, 6, 5, when the point's distance from the car (x, z) is within the object's
//! reach (`dword_634BA0[n]` x the scale >> 4, + 0x28000, by the sign of the `cmp`),
//! `physics::object_hits_car`. Inside either way: the contact's angle (`fpatan`, into
//! `dword_6D3B70`, + 0x80 & 0xFF), the car's speed x 65536 (`dword_6D3B74`) and the
//! point / 65536 (`flt_6D3B58` ..); then by kind: 2, 7, 3, 8, 4 `touch_face`; 6, 9, 5, 1,
//! 10 `physics::restore_saved_car`, `physics::feel_ground`, `touch_face` and the push
//! back: minus the car's speed, level (y 0), made one long (`race::normalise_work_vector`)
//! as the normal, the speeds and the point (y 0) taken, no slide limit,
//! `physics::contact_force` into contact record 0.
//!
//! In:  ebx the face
//! Out: the registers as the steps and calls leave them (eax's low byte from the face's
//!      bytes where the original loads `al`); no flags read (the caller goes on)
//!
//! Caller: `physics::record_face_contacts`.
//! Refuses a kind past the 0x4B0 material records, and what its calls refuse.
//! Self-test (in a race; skipped otherwise): the original and the Rust version on the
//! stage's own faces (everything written saved and put back), the car where it is and
//! moved onto each object face's point (so the paths inside are taken), both directions.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::int_hypot;
use crate::math::x87_run::compare_80;
use crate::physics::contact_force::{RECORD, contact_force};
use crate::physics::flow::{a, base, copy3, fill3, g, negate, p, x87_run};
use crate::physics::point_hits_car::KIND;
use crate::physics::touch_face::{FACES, touch_face};
use crate::physics::vector_length::vector_length;
use crate::race::norms::normalise_work_vector;
use rc2k_core::physics::face_contacts::{self as contacts, FaceObject, Inside};

const WORK: usize = 0x5C5F60; // g_workVector
const OBJECT: usize = 0x6D321C; // dword_6D321C

fn byte(x: u32) -> u32 {
    unsafe { (x as *const u8).read() as u32 }
}

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

fn touch() -> Result<(), Refused> {
    unsafe { touch_face() }?;
    Ok(())
}

/// The steps for face `number` (ebx). The registers the original leaves are read by none
/// of its callers.
// Converted: `rc2k_core::physics::face_contacts` (the x87 steps stay).
pub fn steps(number: u32) -> Result<(), Refused> {
    p(0x6D3218, number);
    let face = (number << 6).wrapping_add(g(FACES));
    if unsafe { crate::scene::kind_hidden_at::kind_hidden_at(face) } {
        return Ok(());
    }
    let object = match contacts::face_object(byte(face), || byte(face + 1)) {
        FaceObject::NotObject => return Ok(()),
        FaceObject::TouchOnly => {
            p(KIND, 0xFF);
            return touch();
        }
        FaceObject::Object(object) => object,
    };
    p(OBJECT, object);
    let kind = g(0x5CD27C + 4 * object as usize);
    contacts::check_material(kind)?;
    p(KIND, byte(a(0x97DF30) + 0x80 * kind + 0x1C));
    if contacts::object_tried(byte(face + 0x25), g(0x71B400), object).is_err() {
        return Ok(());
    }
    for (k, o) in [8u32, 0xC, 0x10].iter().enumerate() {
        p(0x63503C + 4 * k, contacts::turn_word(byte(face + o)));
    }
    p(0x635038, byte(face + 0x18));
    // esi and ebp both the face here.
    let (inside, _, point) =
        unsafe { crate::physics::scene_point_hits_car::scene_point_hits_car(face, face) };
    if !inside {
        if !contacts::tries_points(g(KIND)) {
            return Ok(());
        }
        let dx = rd(point)
            .wrapping_add(rd(face + 0x30))
            .wrapping_sub(g(0x71A10C));
        let dz = rd(point + 8)
            .wrapping_add(rd(face + 0x38))
            .wrapping_sub(g(0x71A114));
        let distance = int_hypot(dx as i32, dz as i32) as u32;
        p(0x62400C, distance);
        let (reach, _) = contacts::reach(g(0x634BA0 + 4 * g(OBJECT) as usize), g(0x635038));
        if contacts::out_of_reach(reach, distance) {
            return Ok(());
        }
        // esi the scene point, ebp the face.
        if !unsafe { crate::physics::object_hits_car::object_hits_car(point, face) }? {
            return Ok(());
        }
    }
    // 0x45E523: inside.
    x87_run!(
        "fild dword ptr [{b} + 0x2D3B4C]",
        "fild dword ptr [{b} + 0x2D3B54]",
        ".byte 0xD9, 0xF3", // fpatan
        "fmul dword ptr [{b} + 0x1C5E2C]",
        "fmul dword ptr [{b} + 0x1C5DD4]",
        "fistp dword ptr [{b} + 0x2D3B70]";
        b = base()
    );
    p(0x6D3B70, g(0x6D3B70).wrapping_add(0x80) & 0xFF);
    x87_run!(
        "fld dword ptr [{b} + 0x31A170]",
        ".byte 0xD8, 0xC8", // fmul st, st
        "fld dword ptr [{b} + 0x31A178]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{b} + 0x31A174]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA", // fsqrt
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fistp dword ptr [{b} + 0x2D3B74]",
        "fild dword ptr [{b} + 0x2D3090]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2D3B58]",
        "fild dword ptr [{b} + 0x2D3094]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2D3B5C]",
        "fild dword ptr [{b} + 0x2D3098]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2D3B60]";
        b = base()
    );
    match contacts::inside(g(KIND)) {
        Inside::PushBack => {}
        Inside::Touch => return touch(),
        Inside::Nothing => return Ok(()),
    }
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?;
    unsafe { crate::physics::feel_ground::feel_ground() }?;
    touch()?;
    // 0x45E62A: pushed back against the motion, level.
    copy3(WORK, 0x71A170);
    negate(WORK);
    negate(WORK + 4);
    negate(WORK + 8);
    if compare_80(unsafe { vector_length(a(WORK)) }, g(0x5C5D90)) & 0x4000 != 0 {
        fill3(WORK, 0);
    }
    p(WORK + 4, 0);
    unsafe { normalise_work_vector() };
    copy3(0x6D3078, WORK);
    copy3(0x6D30A8, 0x71A170);
    copy3(0x6D30B4, 0x71A15C);
    copy3(0x6D3090, 0x6D3B78);
    p(0x6D3094, 0);
    p(0x6D3100, 0);
    p(RECORD, a(0x614A50));
    unsafe { contact_force() };
    Ok(())
}
