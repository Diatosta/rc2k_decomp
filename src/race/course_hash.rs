//! With `RC2K_COURSE_HASH` (a check, in the builds with the `remote` feature): what loading a
//! stage leaves in the course stores, and what freeing it leaves, hashed store by store and
//! logged as `course hash <stage> <load|free> <store>: <hex>`, so that two builds can be
//! compared stage by stage (`coursecmp.py`). Tables are hashed by their bytes, records by field
//! path and value (`drive::sim_hash`), so the records' layout does not matter. The stage's second
//! load in a process (after the first one's free) is a second `load` line set.

#[cfg(feature = "remote")]
use crate::drive::sim_hash::{byte_run_hash, dword_run_hash};
use crate::drive::sim_hash::{bytes_hash, record_hash};

/// The switch, read once.
#[cfg(feature = "remote")]
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_COURSE_HASH").is_some())
}

/// A shared table's hash, or "none" when it is not taken.
fn table(table: &crate::memory::shared_table::SharedTable) -> String {
    let bytes = table
        .len()
        .and_then(|len| table.bytes(0, len))
        .ok()
        .flatten();
    bytes.map_or_else(
        || "none".to_string(),
        |b| format!("{:016x}", bytes_hash(&b)),
    )
}

/// The hash of the texture places as the cell store they were hashed as: every table one run of
/// dwords under its field's name (`record_hash`'s way), the three dwords past the normals' marks
/// included.
#[cfg(feature = "remote")]
fn texture_places_hash(scene: &crate::scene::owner::Scene) -> u64 {
    let places = scene.texture_places();
    let slots = scene.textures.place_slots.borrow();
    let place_slot: &[u32] = &slots.place_slot;
    let totals = [
        places.video_kilobytes,
        places.system_kilobytes,
        slots.video_textures,
        places.system_textures,
    ];
    let runs: [(&str, &[u32]); 18] = [
        (".texture_of[0]", &places.texture_of),
        (".place_slot[0]", place_slot),
        (".kind_pictures[0]", &places.kind_pictures),
        (".wanted_pictures[0]", &places.wanted_pictures),
        (".loaded_pictures[0]", &places.loaded_pictures),
        (".slot_wanted[0]", &places.slot_wanted),
        (".slot_loaded[0]", &places.slot_loaded),
        (".totals[0]", &totals),
        (
            ".kind_slot_bytes",
            std::slice::from_ref(&places.kind_slot_bytes),
        ),
        (".counters[0]", &places.counters),
        (".step_flags[0]", &places.step_flags),
        (".car_icon_sheet_x[0]", &places.car_icon_sheet_x),
        (".car_icon_sheet_y[0]", &places.car_icon_sheet_y),
        (".object_texture_records[0]", &places.object_texture_records),
        (".face_marks[0]", &places.face_marks),
        (".point_marks[0]", &places.point_marks),
        (".normal_marks[0]", &places.normal_marks),
        (".reserved_173d4[0]", &[0; 3]),
    ];
    runs.iter().fold(0, |sum, (path, run)| {
        sum.wrapping_add(dword_run_hash(path, run))
    })
}

/// The hash of the slot shapes as the cell store they were: each table one run of dwords under
/// its field's name.
#[cfg(feature = "remote")]
fn slot_shapes_hash(scene: &crate::scene::owner::Scene) -> u64 {
    let shapes = scene.objects.slots.borrow();
    [
        (".first_point[0]", &shapes.first_point),
        (".point_counts[0]", &shapes.point_counts),
        (".normal_counts[0]", &shapes.normal_counts),
        (".first_face[0]", &shapes.first_face),
        (".face_counts[0]", &shapes.face_counts),
    ]
    .iter()
    .fold(0u64, |sum, (path, run)| {
        sum.wrapping_add(dword_run_hash(path, *run))
    })
}

/// The hash of the object kinds as the cell store they were: the kinds one run, the two lone
/// fields.
#[cfg(feature = "remote")]
fn object_kinds_hash(scene: &crate::scene::owner::Scene) -> u64 {
    let tables = scene.objects.tables.borrow();
    dword_run_hash(".object_kinds[0]", &tables.object_kinds)
        .wrapping_add(dword_run_hash(".point_count", &[tables.point_count]))
        .wrapping_add(dword_run_hash(".d3d_drawing_on", &[tables.d3d_drawing_on]))
}

/// The hash of the object slots' tables: the lent store's runs and the plain fields' as the one cell
/// store they were (the two unused tables of zeros included).
#[cfg(feature = "remote")]
fn object_slots_hash(scene: &crate::scene::owner::Scene) -> u64 {
    let slots = scene.loaded_slots();
    let mut runs: Vec<(String, &[u32])> = vec![
        (".listed_faces[0]".into(), &[0; 4096]),
        (".phase_work[0]".into(), &[0; 3]),
        (
            ".texture_count".into(),
            std::slice::from_ref(&slots.texture_count),
        ),
        (".texture_list[0]".into(), &slots.texture_list),
        (
            ".no_normals".into(),
            std::slice::from_ref(&slots.no_normals),
        ),
        (".measures[0]".into(), &slots.measures),
        (".radii[0]".into(), &slots.radii),
    ];
    for (n, table) in slots.part_extents.iter().enumerate() {
        runs.push((format!(".part_extents[{n}][0]"), table));
    }
    runs.iter()
        .fold(slot_shapes_hash(scene), |sum, (path, run)| {
            sum.wrapping_add(dword_run_hash(path, run))
        })
}

/// The hash of the object tables: the lent store's runs and the plain fields' as the one cell store they
/// were (the unused fields included, as they stood).
#[cfg(feature = "remote")]
fn object_tables_hash(scene: &crate::scene::owner::Scene) -> u64 {
    let levels = scene.texture_levels();
    let runs: [(&str, &[u32]); 12] = [
        (".face_count", std::slice::from_ref(&levels.face_count)),
        (".normals_at", &[0]),
        (".car_mark_textures[0]", &[0; 256]),
        (".car_letter_b", &[0]),
        (".reserved_c14[0]", &[0, 0, 0, 0x100, 0x80, 0x40, 0x20]),
        (".level_sizes[0]", &levels.level_sizes),
        (".level_mips[0]", &levels.level_mips),
        (".other_levels", std::slice::from_ref(&levels.other_levels)),
        (".texture_tags[0]", &levels.texture_tags),
        (".system_levels[0]", &levels.system_levels),
        (".video_levels[0]", &levels.video_levels),
        (".slot_kinds[0]", &levels.slot_kinds),
    ];
    runs.iter()
        .fold(object_kinds_hash(scene), |sum, (path, run)| {
            sum.wrapping_add(dword_run_hash(path, run))
        })
}

/// The hash of the kinds' texture tables: the lent store's slots and the first textures, as the one
/// cell store they were.
#[cfg(feature = "remote")]
fn texture_kinds_hash(scene: &crate::scene::owner::Scene) -> u64 {
    dword_run_hash(
        ".kind_first_slots[0]",
        &scene.textures.kinds.borrow().kind_first_slots,
    )
    .wrapping_add(dword_run_hash(
        ".kind_first_textures[0]",
        &*scene.textures.first_textures(),
    ))
}

/// The hash of the course lists as the one cell store they were: each field one run of dwords under
/// its name (`record_hash`'s way), the memory lists' four words each, the physics' turn work, and the
/// fields no longer kept (they only ever held 0) as zero runs.
#[cfg(feature = "remote")]
fn lists_hash(course: &crate::race::owner_course::Course) -> u64 {
    let lists = course.lists.borrow();
    let turn = course.turn_work();
    let memory = course.memory();
    let bits = |floats: &[f32]| floats.iter().map(|f| f.to_bits()).collect::<Vec<u32>>();
    let cells = |cells: &[crate::memory::session::PlainFloat]| {
        cells
            .iter()
            .map(crate::memory::session::PlainFloat::bits)
            .collect::<Vec<u32>>()
    };
    let mut runs: Vec<(String, Vec<u32>)> = vec![
        (".input_keys_mask".into(), vec![lists.input_keys_mask]),
        (".face_key_factors[0]".into(), bits(&lists.face_key_factors)),
        (".list_6ed0[0]".into(), vec![0; 4]),
        (".course_bytes_total".into(), vec![lists.course_bytes_total]),
        (
            ".second_course_list_a".into(),
            vec![lists.second_course_list_a],
        ),
        (
            ".second_course_list_b".into(),
            vec![lists.second_course_list_b],
        ),
        (".object_points".into(), vec![lists.object_points]),
        (".face_memory".into(), vec![lists.face_memory]),
        (".object_points_room".into(), vec![lists.object_points_room]),
        (".face_memory_room".into(), vec![lists.face_memory_room]),
        (".dword_6fb4_unused".into(), vec![0]),
        (".point_records".into(), vec![lists.point_records]),
        (".squared_row_0[0]".into(), vec![0; 4]),
        (".squared_row_1[0]".into(), vec![0; 4]),
        (".squared_row_2_and_after[0]".into(), vec![0; 8]),
        (".turn_matrix[0]".into(), cells(&turn.turn_matrix)),
        (".invert_copy[0]".into(), vec![0; 9]),
        (".invert_made[0]".into(), vec![0; 9]),
        (".vector_length".into(), vec![0]),
        (".vector_unit_x".into(), vec![0]),
        (".vector_unit_y".into(), vec![0]),
        (".vector_unit_z".into(), vec![0]),
        (".vector_scratch".into(), vec![0]),
        (".root_table[0]".into(), bits(&lists.root_table)),
        (
            ".exponent_root_table[0]".into(),
            bits(&lists.exponent_root_table),
        ),
        (".spin_axis[0]".into(), cells(&turn.spin_axis)),
    ];
    for (name, cell) in [
        ("turn_sine", &turn.turn_sine),
        ("turn_cosine", &turn.turn_cosine),
        ("turn_one_less_cosine", &turn.turn_one_less_cosine),
        ("turn_product_xz", &turn.turn_product_xz),
        ("turn_product_xy", &turn.turn_product_xy),
        ("turn_product_yz", &turn.turn_product_yz),
        ("turn_sine_x", &turn.turn_sine_x),
        ("turn_sine_y", &turn.turn_sine_y),
        ("turn_sine_z", &turn.turn_sine_z),
        ("car_pitch", &turn.car_pitch),
        ("car_heading", &turn.car_heading),
        ("car_roll", &turn.car_roll),
        ("turned_x", &turn.turned_x),
        ("turned_y", &turn.turned_y),
        ("turned_z", &turn.turned_z),
        ("spin_angle", &turn.spin_angle),
    ] {
        runs.push((format!(".{name}"), vec![cell.bits()]));
    }
    for (name, list) in [
        ("course_memory_list", &memory.course_memory_list),
        ("second_course_list", &memory.second_course_list),
        ("object_points_list", &memory.object_points_list),
        ("point_records_list", &memory.point_records_list),
        ("mud_list", &memory.mud_list),
    ] {
        let list = list.get();
        for (field, value) in [
            ("head", list.head),
            ("total", list.total),
            ("cursor", list.cursor),
            ("remaining", list.remaining),
        ] {
            runs.push((format!(".{name}.{field}"), vec![value]));
        }
    }
    runs.iter().fold(0, |sum, (path, run)| {
        sum.wrapping_add(dword_run_hash(path, run))
    })
}

/// The hash of the stage's stores after loading (`after` "load") or freeing it ("free"), logged.
#[cfg(feature = "remote")]
pub fn log_hashes(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
    after: &str,
) {
    if !on() {
        return;
    }
    let data = &race.course.stage.course;
    let stage = race.shared.vars.get().stage_number.get();
    let say = |store: &str, hash: String| {
        crate::program_log::log(&format!("course hash {stage} {after} {store}: {hash}"));
    };
    let reads = race.course.reads();
    say("faces", table(&data.faces));
    say("records", table(&data.records));
    say("info", table(&data.info));
    say("scene_points", table(&data.points));
    say(
        "object_points",
        table(race.course.stage.object_points.table()),
    );
    say(
        "object_faces",
        table(race.course.stage.object_faces.table()),
    );
    // The course store's fields as they stood before the unused ones were deleted (their sum
    // was the old "course" line, which the deleted fields' values made unrepeatable): one line
    // for each field kept, hashed as the one cell it was.
    let work = reads.course.get();
    for (path, value) in [
        (".course_record_count", work.course_record_count),
        (".course_face_count", work.course_face_count),
        (".scene_point_count", work.scene_point_count),
        (".camera_first", work.camera_first),
        (".place_heading", work.place_heading),
        (".flat_depth", work.flat_depth),
        (".place_x", work.place_x),
        (".place_y", work.place_y),
        (".place_z", work.place_z),
    ] {
        say(
            &format!("course_field{path}"),
            format!("{:016x}", dword_run_hash(path, &[value])),
        );
    }
    say("lists", format!("{:016x}", lists_hash(&race.course)));
    say(
        "stage_prefs",
        format!("{:016x}", record_hash(reads.stage_prefs)),
    );
    let kinds = race.course.kind_records().borrow();
    say(
        "kind_records",
        format!(
            "{:016x}",
            dword_run_hash(".first_records[0]", &kinds.first_records)
                .wrapping_add(dword_run_hash(".last_records[0]", &kinds.last_records))
        ),
    );
    drop(kinds);
    // The level grid store (one constant, the depth shift) is gone: its last hash, which never changed.
    say("level_grid", "731b982efc7e13e3".to_string());
    say("object_slots", format!("{:016x}", object_slots_hash(scene)));
    say(
        "object_tables",
        format!("{:016x}", object_tables_hash(scene)),
    );
    say(
        "texture_places",
        format!("{:016x}", texture_places_hash(scene)),
    );
    say(
        "texture_kinds",
        format!("{:016x}", texture_kinds_hash(scene)),
    );
    say(
        "mud",
        race.course
            .stage
            .mud
            .with(|m| format!("{:016x}", bytes_hash(m)))
            .unwrap_or("none".into()),
    );
    say(
        "map",
        race.course
            .stage
            .game_map
            .with(|m| {
                let tiles: Vec<u8> = format!("{:?}", m.tiles).into_bytes();
                format!("{:016x}:{:016x}", bytes_hash(&tiles), bytes_hash(&m.pixels))
            })
            .unwrap_or("none".into()),
    );
}

/// The running hash of the texture rows uploaded since the stage load began (`RC2K_COURSE_HASH`).
#[cfg(feature = "remote")]
static TEXTURE_HASH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Starts the texture rows' hash anew.
#[cfg(feature = "remote")]
pub fn reset_texture_hash() {
    TEXTURE_HASH.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// Adds a texture row (its texture, level, row number and bytes) to the texture rows' hash.
#[cfg(feature = "remote")]
pub fn note_texture_row(texture: u32, level: u32, y: u32, row: &[u8]) {
    if !on() {
        return;
    }
    let key = u64::from(texture) << 40 ^ u64::from(level) << 32 ^ u64::from(y);
    let row_hash = bytes_hash(row) ^ key.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let old = TEXTURE_HASH.load(std::sync::atomic::Ordering::Relaxed);
    TEXTURE_HASH.store(
        old.rotate_left(5).wrapping_mul(0x0100_0000_01B3) ^ row_hash,
        std::sync::atomic::Ordering::Relaxed,
    );
}

/// The hash of the one-time marks' mask variables, as the cell store they replaced hashed:
/// one run per field (an array of cells, or a table of bytes, whole), a shape's fields by their
/// own path, and the two zero runs of the store's unused words.
#[cfg(feature = "remote")]
fn mask_vars_hash(masks: &crate::effects::mask_store::Masks) -> u64 {
    let v = &*masks.vars.borrow();
    let mut sum = 0u64;
    let mut dwords = |path: &str, run: &[u32]| sum = sum.wrapping_add(dword_run_hash(path, run));
    let bits = |run: &[f32]| run.iter().map(|f| f.to_bits()).collect::<Vec<u32>>();
    dwords(".spans_total", &[v.spans_total]);
    let list = v.list.get();
    dwords(".list.head", &[list.head]);
    dwords(".list.total", &[list.total]);
    dwords(".list.cursor", &[list.cursor]);
    dwords(".list.remaining", &[list.remaining]);
    dwords(".work", &[0]);
    dwords(".count", &[v.count]);
    dwords(".corner_scale[0]", &bits(&v.corner_scale));
    dwords(".turn[0]", &bits(&v.turn));
    dwords(".slope[0]", &bits(&v.slope));
    dwords(".work_pixel[0]", &v.work_pixel);
    dwords(".lines[0]", &v.lines);
    dwords(".ends[0]", &bits(&v.ends));
    dwords(".ring_corners", &[v.ring_corners]);
    for (n, shape) in v.shapes.iter().enumerate() {
        dwords(&format!(".shapes[{n}].ease_step"), &[shape.ease_step]);
        dwords(&format!(".shapes[{n}].count"), &[shape.count]);
        dwords(&format!(".shapes[{n}].rows"), &[shape.rows]);
        dwords(&format!(".shapes[{n}].middle[0]"), &bits(&shape.middle));
        dwords(
            &format!(".shapes[{n}].angle_ends[0]"),
            &bits(&shape.angle_ends),
        );
        dwords(&format!(".shapes[{n}].radii[0]"), &bits(&shape.radii));
    }
    dwords(".height", &[v.height.to_bits()]);
    dwords(".row_int", &[v.row_int]);
    dwords(".row", &[v.row.to_bits()]);
    dwords(".slopes[0]", &bits(&v.slopes));
    dwords(".xs[0]", &bits(&v.xs));
    dwords(".split", &[v.split.to_bits()]);
    dwords(".cursor", &[v.cursor]);
    dwords(".runs", &[v.runs]);
    dwords(".edge[0]", &v.edge);
    dwords(".triangle[0]", &bits(&v.triangle));
    dwords(".quad_triangle[0]", &bits(&v.quad_triangle));
    dwords(".ring_quad[0]", &bits(&v.ring_quad));
    dwords(".run_count", &[v.run_count]);
    dwords(".ring_ends[0]", &bits(&v.ring_ends));
    dwords(".ring_middle", &[v.ring_middle.to_bits()]);
    dwords(".ring_half", &[v.ring_half.to_bits()]);
    dwords(".ring_angles[0]", &bits(&v.ring_angles));
    dwords(".ring_turns[0]", &bits(&v.ring_turns));
    dwords(".ring_angle", &[v.ring_angle]);
    dwords(".ring_step", &[v.ring_step.to_bits()]);
    dwords(".pixel[0]", &v.pixel);
    dwords(".mask_value", &[v.mask_value]);
    dwords(".pixel_table", &[v.pixel_table]);
    dwords(".middle[0]", &bits(&v.middle));
    dwords(".row_distance", &[v.row_distance.to_bits()]);
    dwords(".pixel_corners", &[v.pixel_corners]);
    let mut bytes = |path: &str, run: &[u8]| sum = sum.wrapping_add(byte_run_hash(path, run));
    bytes(".list_rest", &[0; 48]);
    bytes(".corners_a", &v.corners[0]);
    bytes(".corners_b", &v.corners[1]);
    bytes(".control_word", &v.control_word.to_le_bytes());
    bytes(".status_word", &v.status_word.to_le_bytes());
    bytes(".run_table", &v.run_table);
    bytes(".mask_a", &v.masks[0]);
    bytes(".mask_b", &v.masks[1]);
    bytes(".ring_angles_a", &v.ring_angle_tables[0]);
    bytes(".ring_angles_b", &v.ring_angle_tables[1]);
    sum
}

/// The hash of what the marks' masks leave: the picks (with both pixel tables), the mask
/// variables and the marks' work cells, logged as `course hash masks <store>: <hex>`.
#[cfg(feature = "remote")]
pub fn log_mask_hashes(
    masks: &crate::effects::mask_store::Masks,
    drops: &crate::effects::windscreen_drop_vars_store::WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) {
    if !on() {
        return;
    }
    let say = |store: &str, hash: u64| {
        crate::program_log::log(&format!("course hash masks {store}: {hash:016x}"));
    };
    // The plain fields hash as the cell store they replaced: a run of dwords each (a lone one for
    // a single field), the pixel tables' bytes as one run apiece.
    let tables = [
        ("pixel_table_a", &drops.pixel_tables.a),
        ("pixel_table_b", &drops.pixel_tables.b),
    ];
    let picks = drops.picks.borrow();
    let runs: [(&str, &[u32]); 11] = [
        (".flags[0]", &picks.flags),
        (".index", &[picks.index]),
        (".table[0]", &picks.table),
        (".values[0]", &picks.values),
        (".shapes", &[picks.shapes]),
        (".mask", &[picks.mask]),
        (".table_in_use", &[picks.table_in_use]),
        (".corners", &[picks.corners]),
        (".moving", &[picks.moving]),
        (".next", &[picks.next]),
        (".masks[0]", &picks.masks),
    ];
    let mut picks_hash = runs.iter().fold(0u64, |sum, (path, run)| {
        sum.wrapping_add(dword_run_hash(path, run))
    });
    for (name, table) in tables {
        let bytes = table.borrow();
        picks_hash = picks_hash.wrapping_add(byte_run_hash(&format!(".{name}"), &*bytes));
        say(name, bytes_hash(&*bytes));
    }
    say("picks", picks_hash);
    drop(picks);
    say("vars", mask_vars_hash(masks));
    let work = effects.setup.work.borrow();
    let fields: [(&str, u32); 9] = [
        (".fpu_status", work.fpu_status),
        (".shake", work.shake.to_bits()),
        (".scratch", work.scratch),
        (".seed", work.seed),
        (".sound_source", work.sound_source),
        (".sound_zero", work.sound_zero),
        (".cosine_start", work.cosine_start),
        (".cosine_step", work.cosine_step),
        (".cosine_half", work.cosine_half),
    ];
    // The plain fields hash as the cell store they replaced: a lone dword each.
    say(
        "work",
        fields.iter().fold(0u64, |sum, (path, value)| {
            sum.wrapping_add(dword_run_hash(path, &[*value]))
        }),
    );
}

/// The hash of what the stage's start (the race init after the load) leaves in the checkpoints
/// and the race variables, logged as `course hash <stage> started <store>: <hex>`.
#[cfg(feature = "remote")]
pub fn log_started_hashes(race: &crate::race::owner::Race) {
    if !on() {
        return;
    }
    let stage = race.shared.vars.get().stage_number.get();
    let say = |store: &str, hash: u64| {
        crate::program_log::log(&format!("course hash {stage} started {store}: {hash:016x}"));
    };
    say("checkpoints", record_hash(race.checkpoints()));
    say("race_vars", record_hash(race.shared.vars.get()));
    say(
        "texture_rows",
        TEXTURE_HASH.load(std::sync::atomic::Ordering::Relaxed),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hashing scheme is frozen: a fresh numbers record and an empty table keep their
    /// hashes, so a change to how a store is hashed (not to the store) shows here.
    #[test]
    fn hashing_scheme_is_frozen() {
        let _lock = crate::test_support::test_lock();
        let numbers = crate::race::numbers_store::NumberVars::new();
        let zero = record_hash(&numbers);
        assert_eq!(
            zero,
            record_hash(&crate::race::numbers_store::NumberVars::new())
        );
        assert_eq!(bytes_hash(&[]), 0xa8c7_f832_281a_39c5);
        assert_eq!(bytes_hash(&[1, 2, 3]), bytes_hash(&[1, 2, 3]));
        assert_ne!(bytes_hash(&[1, 2, 3]), bytes_hash(&[1, 2, 4]));
        eprintln!("numbers record fresh hash {zero:016x}");
    }

    /// A taken table shows as its bytes' hash, a given-back one as "none".
    #[test]
    fn a_table_is_none_until_taken_and_after_it_is_given_back() {
        let _lock = crate::test_support::test_lock();
        let t = crate::memory::shared_table::SharedTable::new(0x100, "test");
        assert_eq!(table(&t), "none");
        t.set(&[7; 16]).unwrap();
        assert_eq!(table(&t), format!("{:016x}", bytes_hash(&[7; 16])));
        t.give_back();
        assert_eq!(table(&t), "none");
    }
}
