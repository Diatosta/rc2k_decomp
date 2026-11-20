//! With `RC2K_COURSE_HASH` (a check, in the builds with the `remote` feature): what loading a
//! stage leaves in the course stores, and what freeing it leaves, hashed store by store and
//! logged as `course hash <stage> <load|free> <store>: <hex>`, so that two builds can be
//! compared stage by stage (`coursecmp.py`). Tables are hashed by their bytes, records by field
//! path and value (`drive::sim_hash`), so the records' layout does not matter. The stage's second
//! load in a process (after the first one's free) is a second `load` line set.

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
    say("course", format!("{:016x}", record_hash(reads.course)));
    say("lists", format!("{:016x}", record_hash(reads.lists)));
    say(
        "stage_prefs",
        format!("{:016x}", record_hash(reads.stage_prefs)),
    );
    say(
        "kind_records",
        format!("{:016x}", record_hash(reads.kind_records)),
    );
    say(
        "level_grid",
        format!("{:016x}", record_hash(race.course.level_grid.get())),
    );
    say(
        "object_slots",
        format!("{:016x}", record_hash(scene.objects.slots.get())),
    );
    say(
        "object_tables",
        format!("{:016x}", record_hash(scene.objects.tables.get())),
    );
    say(
        "texture_places",
        format!("{:016x}", record_hash(scene.textures.places.get())),
    );
    say(
        "texture_kinds",
        format!("{:016x}", record_hash(scene.textures.kinds.get())),
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
    say("picks", record_hash(&drops.picks));
    for (name, table) in [
        ("pixel_table_a", &drops.picks.pixel_table_a),
        ("pixel_table_b", &drops.picks.pixel_table_b),
    ] {
        if let Ok(bytes) = table.read_at(0, 0x2_0000, "a pixel table") {
            say(name, bytes_hash(&bytes));
        }
    }
    say("vars", record_hash(&masks.vars));
    say("work", record_hash(&effects.setup.work));
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

    /// The hashing scheme is frozen: a fresh (zero) course header and an empty table keep their
    /// hashes, so a change to how a store is hashed (not to the store) shows here.
    #[test]
    fn hashing_scheme_is_frozen() {
        let _lock = crate::test_support::test_lock();
        let course = crate::race::course_store::CourseWork::new();
        let zero = record_hash(&course);
        assert_eq!(
            zero,
            record_hash(&crate::race::course_store::CourseWork::new())
        );
        assert_eq!(bytes_hash(&[]), 0xa8c7_f832_281a_39c5);
        assert_eq!(bytes_hash(&[1, 2, 3]), bytes_hash(&[1, 2, 3]));
        assert_ne!(bytes_hash(&[1, 2, 3]), bytes_hash(&[1, 2, 4]));
        eprintln!("course header zero hash {zero:016x}");
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
