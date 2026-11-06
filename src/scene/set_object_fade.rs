//! `scene::set_object_fade` (0x451609): how see-through an object is drawn by how far
//! off it is, and whether another, simpler object is drawn in its place.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx, edx and esi changed; its caller goes on
//!      without them, and the Rust version keeps them
//!
//! From the record of the object's kind (0x80 bytes at `unk_97DF30`) two words: +0x1D to
//! `qword_6214E0` (another kind to draw instead) and +0x22 to its second half (a distance).
//! `dword_635004` and `dword_63500C` = 0, `dword_63996C` = 0 (no fading), `dword_639974` =
//! 0x100 (not see-through). Nothing more if both words are 0 or the object's place
//! `dword_634FFC` (a point of the scene) is -1. Else that point's record (made if it is not
//! there: `sub_44CF33`, the same as `scene::project_point` makes one) gives `dword_635004` and
//! `dword_63500C`: its x and depth in the view as whole numbers. With the depth beyond the
//! distance times 65536: `dword_639974` and `dword_639978` = (the distance times 65536 less the
//! depth, plus 0x100000, not under 0) over 0x1000, `dword_63996C` = 1, and if the first word is
//! not 0 and an object of that kind exists (`sub_422298`: the first of the objects 1 .. 0xFF
//! with that kind), `g_objectNumber` becomes that object. One caller, `sub_45045D`. Refuses an
//! object's number past the tables and, where a record has to be made, no records or no points.

use crate::errors::BadInput;
use crate::math::x87;
use crate::scene::owner::scene;
use crate::scene::point_records::field;
use crate::scene::project_point::make_scene_record;
use crate::scene::set_object_material::MATERIAL_LEN;

// ---- game data ----
const OBJECTS: u32 = 0x100;

// ---- Rust version ----

fn record_at() -> Option<u32> {
    {
        let place: u32 = scene().drawing.object_work.object_place.bits();
        crate::scene::point_records::held()
            .then(|| crate::scene::point_records::of_scene_point(place))
    }
}

/// The two words of the object's kind, and whether the object's place is looked at;
/// None for an object's number past the tables.
fn words(shared: &crate::race::owner_shared::Shared) -> Result<Option<(u32, u32, bool)>, BadInput> {
    let scene = scene();
    {
        let object: u32 = shared.switches.object_number.bits();
        if object >= OBJECTS {
            return Ok(None);
        }
        let kind: u32 = scene.objects.tables.object_kind(object);
        let material = crate::scene::polygon::materials_ptr()
            .offset(((kind as usize).wrapping_mul(MATERIAL_LEN)) as u32);
        let word = |at: usize| {
            material
                .offset(at as u32)
                .read::<u16>("an object's material")
                .map(u32::from)
        };
        let (other_kind, distance) = (word(0x1D)?, word(0x22)?);
        let looked_at = (other_kind | distance) != 0
            && scene.drawing.object_work.object_place.bits() != u32::MAX;
        Ok(Some((other_kind, distance, looked_at)))
    }
}

pub fn set_object_fade(race: &crate::race::owner::Race) -> Result<(), BadInput> {
    let scene = scene();
    let Some((other_kind, distance, needs_record)) = words(&race.shared)? else {
        return Err(BadInput("an object's number past the tables"));
    };
    let place: u32 = scene.drawing.object_work.object_place.bits();
    // What can refuse is asked before anything is written.
    let record = match (needs_record, record_at()) {
        (false, _) => 0,
        (true, None) => return Err(BadInput("no records for the points")),
        (true, Some(record)) => {
            if field(record, 0)? != place && race.course.course.scene_points.bits() == 0 {
                return Err(BadInput("no points"));
            }
            record
        }
    };
    crate::session::write_bits(&race.shared.switches.fade_words, [other_kind, distance]);
    scene.drawing.object_work.object_position[0].set_bits(0u32);
    scene.drawing.object_work.object_position[2].set_bits(0u32);
    race.physics.ground.object_fading.set_bits(0u32);
    race.physics.ground.object_fade[0].set_bits(0x100u32);
    if !needs_record {
        return Ok(());
    }
    if field(record, 0)? != place {
        make_scene_record(place, record)?;
    }
    scene.drawing.object_work.object_position[0].set_bits(x87::to_int(field(record, 0x10)?) as u32);
    let depth = x87::to_int(field(record, 0x18)?);
    scene.drawing.object_work.object_position[2].set_bits(depth as u32);
    let near = ((distance << 16) as i32).wrapping_sub(depth);
    if near >= 0 {
        return Ok(());
    }
    let fade = (near.wrapping_add(0x10_0000).max(0) as u32) >> 12;
    crate::session::write_bits(&race.physics.ground.object_fade, [fade, fade]);
    race.physics.ground.object_fading.set_bits(1u32);
    if other_kind != 0 {
        let kinds: [u32; 0x100] = crate::session::read_bits(&scene.objects.tables.object_kinds);
        if let Some(other) = (1..0x100).find(|&n| kinds[n] == other_kind) {
            race.shared.switches.object_number.set_bits(other as u32);
        }
    }
    Ok(())
}
