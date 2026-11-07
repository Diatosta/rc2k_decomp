//! `race::loadlevel_init`'s "init ed textures" (`scene::init_texture_levels`, 0x424001,
//! IDA 2026-10-16): which texture slots are wanted at which level (`g_textureSystemLevels`,
//! `g_textureVideoLevels`) and tagged how (`g_textureTags`), 0x400 slots each, for the
//! stage: each car's runs, each kind's slots by the stage prefs, then fixed runs. With
//! `dword_5CDADC` set every level is another (it is set to 0 here first, and nothing sets
//! it otherwise).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::session::Dword;

/// A kind's block: its first-texture variable (an index in `kind_first_textures`; only its place in `g_textureWantedPictures`
/// is worked out, and not used: section 16), its prefs entries, the first slot, how many,
/// the tag, the level, the other level, and what `dword_5DC978` gets a slot (4, or 1 for
/// the kinds of slots 0x40 .. 0x6F).
type Block = (usize, Prefs, usize, u32, u32, u32, u32, u32, u32);
#[derive(Clone, Copy)]
enum Prefs {
    A,
    B,
    C,
}

impl Prefs {
    fn list(self, course: &Course) -> &[Dword] {
        match self {
            Prefs::A => &course.stage_prefs.picture_lists_a,
            Prefs::B => &course.stage_prefs.picture_lists_b,
            Prefs::C => &course.stage_prefs.picture_lists_c,
        }
    }
}

const BLOCKS: [Block; 26] = [
    (0, Prefs::A, 0, 0x00, 4, 0x0D, 0, 2, 4),
    (4, Prefs::A, 4, 0x04, 4, 0x0C, 0, 2, 4),
    (8, Prefs::A, 8, 0x08, 4, 0x0B, 0, 2, 4),
    (12, Prefs::A, 12, 0x0C, 4, 0x0A, 0, 2, 4),
    (16, Prefs::A, 16, 0x10, 4, 0x09, 0, 2, 4),
    (20, Prefs::A, 20, 0x14, 4, 0x08, 0, 2, 4),
    (24, Prefs::A, 24, 0x18, 4, 0x07, 0, 2, 4),
    (28, Prefs::A, 28, 0x1C, 4, 0x06, 0, 2, 4),
    (32, Prefs::A, 32, 0x20, 4, 0x05, 0, 2, 4),
    (36, Prefs::A, 36, 0x24, 4, 0x04, 0, 2, 4),
    (40, Prefs::A, 40, 0x28, 4, 0x03, 0, 2, 4),
    (44, Prefs::A, 44, 0x2C, 4, 0x02, 0, 2, 4),
    (48, Prefs::A, 48, 0x30, 4, 0x01, 0, 2, 4),
    (64, Prefs::B, 0, 0x40, 4, 0x20, 1, 2, 1),
    (68, Prefs::B, 4, 0x44, 4, 0x1F, 1, 2, 1),
    (72, Prefs::B, 8, 0x48, 4, 0x1E, 1, 2, 1),
    (76, Prefs::B, 12, 0x4C, 4, 0x1D, 1, 2, 1),
    (80, Prefs::B, 16, 0x50, 4, 0x1C, 1, 2, 1),
    (84, Prefs::B, 20, 0x54, 4, 0x1B, 1, 2, 1),
    (88, Prefs::B, 24, 0x58, 4, 0x1A, 1, 2, 1),
    (92, Prefs::B, 28, 0x5C, 4, 0x19, 1, 2, 1),
    (96, Prefs::B, 32, 0x60, 4, 0x18, 1, 2, 1),
    (100, Prefs::B, 36, 0x64, 4, 0x17, 1, 2, 1),
    (104, Prefs::B, 40, 0x68, 4, 0x16, 1, 2, 1),
    (108, Prefs::B, 44, 0x6C, 4, 0x15, 1, 2, 1),
    (128, Prefs::C, 0, 0x80, 7, 0x3D, 0, 1, 4),
];

fn other(scene: &crate::scene::owner::Scene) -> bool {
    scene.objects.tables.other_levels.get() != 0
}

const COUNT_TEXT_RVA: &std::ffi::CStr = c"NUM DIFFERENT CARS/OBJECTS/TEXTURE SETS "; // "NUM DIFFERENT CARS/OBJECTS/TEXTURE SETS "

/// `scene::init_texture_levels` (0x424001), see the header: `g_raceCars` held to 1 .. 8
/// (by the signs of the differences, as the original); the tables reset, the count in the
/// log; `dword_5CDADC` 0 and `input::update`; the first car's two runs (slots 0x100,
/// 0x104; the second's level by `dword_63DB94`), each further car's two (0x108 + 8 a car,
/// tag 0x28 + the car); each kind's slots (its prefs entry not 0 or -1: tag, level, and
/// `dword_5DC978` + 4, or + 1 for slots 0x40 .. 0x6F); the fixed runs, two loops at slot 0xDC (the system level of six,
/// the video level of two), and with `dword_621514` three more. What the original
/// leaves in the registers its one caller reads none of (register liveness, 2026-10-23:
/// the site in `race::loadlevel_init`, every register followed, no read).
pub fn init_texture_levels(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let shared = &race.shared;
    // Converted: `rc2k_core::scene::texture_levels::{held_cars, set_levels}`.
    use crate::logfile::log_int::log_int;
    use crate::scene::textures::with_cells;
    use rc2k_core::scene::texture_levels::{KindSlots, held_cars, set_levels};
    let cars = held_cars(shared.view.race_cars.bits());
    shared.view.race_cars.set_bits(cars);
    crate::scene::reset_texture_tables::reset_texture_tables(scene);
    crate::logfile::log_write::log_nul(&COUNT_TEXT_RVA, false, 1);
    log_int(cars as i32, true, 1);
    scene.objects.tables.other_levels.set(0);
    crate::input::keys::update_input(&mut crate::input::keys::game_screenshot);
    let kinds: Vec<(KindSlots, Vec<u32>)> = BLOCKS
        .iter()
        .map(
            |&(_, list, from, first, count, tag, level, other_level, step)| {
                let k = KindSlots {
                    first,
                    tag,
                    level,
                    other_level,
                    step,
                };
                let prefs = list.list(&race.course);
                (
                    k,
                    (0..count as usize).map(|n| prefs[from + n].get()).collect(),
                )
            },
        )
        .collect();
    let set = with_cells(
        [
            &scene.objects.tables.texture_tags,
            &scene.objects.tables.system_levels,
            &scene.objects.tables.video_levels,
        ],
        |[tags, system, video]| {
            set_levels(
                &mut rc2k_core::scene::textures::LevelTables {
                    tags,
                    system,
                    video,
                },
                cars,
                shared.view.players.bits() == 1,
                &kinds,
                shared.switches.editor_mode.bits() != 0,
                other(scene),
                |car| {
                    let kinds = &shared.vars.race_car_kinds;
                    let own = crate::race::blocks_store::own_car_number();
                    car as usize == own || kinds[car as usize].get() == kinds[own].get()
                },
            )
        },
    )?;
    // `g_kindSlotBytes`: + 4 (or 1) a kind's slot.
    scene.textures.places.kind_slot_bytes.set(
        scene
            .textures
            .places
            .kind_slot_bytes
            .get()
            .wrapping_add(set.kind_slots_added),
    );
    Ok(())
}
