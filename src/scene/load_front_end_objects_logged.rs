//! `scene::load_front_end_objects_logged`: `scene::load_front_end_objects`, then its log: a
//! rule, "Front end objects loaded", "total points" the object point count, "total pointmem"
//! `ObjectWork.point_memory_total`, "total faces" `ObjectTables.face_count`, "total facemem"
//! `ObjectWork.face_memory_total` (each label without a line's end, its value in hex8 with
//! one), a rule. Caller: `scene::set_up_front_end_scene`.

use crate::errors::Refused;
use crate::logfile::log_hex8::log_hex8;
use crate::scene::owner::Scene;

const RULE: &std::ffi::CStr = c"===========================";
const LOADED: &std::ffi::CStr = c"Front end objects loaded";
const RULE_2: &std::ffi::CStr = c"===========================";
/// (label, value)
type Total = (&'static std::ffi::CStr, fn(&Scene) -> u32);
const TOTALS: [Total; 4] = [
    (c"total points", |scene| {
        scene.objects.tables.get().point_count.get()
    }),
    (c"total pointmem", |scene| {
        scene.object_work().point_memory_total.get()
    }),
    (c"total faces", |scene| {
        scene.objects.tables.get().face_count.get()
    }),
    (c"total facemem", |scene| {
        scene.object_work().face_memory_total.get()
    }),
];

pub fn load_front_end_objects_logged(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::scene::load_front_end_objects::load_front_end_objects(race, scene)?;
    let text = |s: &'static std::ffi::CStr| s;
    crate::logfile::log_write::log_nul(text(RULE), true, 1);
    crate::logfile::log_write::log_nul(text(LOADED), true, 1);
    for (label, value) in TOTALS {
        crate::logfile::log_write::log_nul(text(label), false, 1);
        log_hex8(value(scene), true, 1);
    }
    crate::logfile::log_write::log_nul(text(RULE_2), true, 1);
    Ok(())
}
