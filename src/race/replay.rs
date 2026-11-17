//! The replay file (`var\tmprpl`): opened with its header at the race's start (`race::open_replay`,
//! the header sealed by `race::seal_replay_header`), frames gathered (`race::add_replay_frame`),
//! written block by block (`race::write_replay_block` through `race::write_replay_chunk`, which
//! packs by runs), and closed with its index ("RCIB", `race::close_replay`). The steps call each
//! other (a failed block closes the file, which writes the last frames as a block); the file
//! primitives are `files::handle`'s.

use crate::errors::{BadInput, Refused};
use crate::memory::session::{PlainBytes, PlainDword};
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::system::owner::System;
#[cfg(test)]
use crate::test_support::own::menus;

/// The race record's seal key (the replay header's checksum, `race::seal_replay_header`;
/// `race::seal_stage_time` turns the first car's time by it). It is what makes a stage time in the
/// saved results depend on the replay it came from, so a time cannot be edited alone. Kept apart
/// from the sound dwords around it.
// The index (`crate::test_support::own::race().shared.replay_file().index.dwords()`) is 0x3850 bytes.
const _: () = assert!(crate::race::replay_file_store::INDEX_DWORDS * 4 == 0x3850);
pub const MOST_BLOCKS: u32 = rc2k_core::race::replay::MOST_BLOCKS;
pub const BLOCK_LEN: u32 = 0x2810;
pub const HEADER_LEN: u32 = 0x430;

/// A replay's name: 0x20 bytes.
pub const NAME_BYTES: usize = 0x20;
/// The replays that can be saved.
pub const SLOTS: usize = 100;

/// The replays' buffers.
#[derive(rc2k_derive::Cells)]
pub struct Replays {
    /// The file's header (a dword array, `HEADER_LEN` bytes).
    pub header: PlainBytes<{ HEADER_LEN as usize }>,
    /// The saved replays' names, 100 of 0x20 bytes (twenty to a page).
    pub names: PlainBytes<{ SLOTS * NAME_BYTES }>,
}

impl Replays {
    /// The replays' buffers; the first values are the image's (all zero).
    pub const fn new() -> Self {
        Replays {
            header: PlainBytes::new([0; HEADER_LEN as usize]),
            names: PlainBytes::new([0; SLOTS * NAME_BYTES]),
        }
    }
}

/// Seals the replay's header part (`sealed_header`, 0x40 dwords): version byte 6, the player's
/// name (`strings::copy_printable`, 0x20), the settings and options copied
/// (`copy_header_settings`), a checksum (from dword 1: add, rotate left 5) into dword 0; then its
/// 32 blocks of 8 chained through `race::seal_block` with the key "halfmoon" (the cipher's key is
/// put back after).
pub fn seal_replay_header(
    race: &Race,
    menus: &Menus,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::replay::{seal_checksum, chained, SEAL_KEY}`.
    use rc2k_core::race::replay::{SEAL_KEY, SEAL_VERSION, chained, seal_checksum};
    // The sealed header is the file's bytes: the version byte and the name sit inside its dwords.
    let replay = &race.replay;
    let sealed = &race.replay_work().sealed_header;
    put_header(sealed, VERSION_AT, &[SEAL_VERSION])?;
    let name: [u8; 0x20] = menus.play.slots.names.player_names.read()[..0x20]
        .try_into()
        .expect("0x20 bytes");
    let printable = crate::strings::copy_printable::printable(&name, 0x20)?;
    put_header(sealed, PLAYER_NAME_AT, &printable)?;
    copy_header_settings_of(race, menus, strings, sealed)?;
    let sealed_dwords = &race.replay_work().sealed_header;
    let dwords: Vec<u32> = (1..0x40).map(|n| sealed_dwords[n].get()).collect();
    let sum = seal_checksum(&dwords);
    sealed_dwords[0].set(sum);
    replay.seal_key().set(sum);
    let w = replay.seal_work();
    let key = [w.key[0].get(), w.key[1].get()];
    w.key[0].set(SEAL_KEY[0]);
    w.key[1].set(SEAL_KEY[1]);
    let (mut a, mut b) = (0u32, 0u32);
    for n in 0..0x20u32 {
        let at = 2 * n as usize;
        let [x, y] = chained(
            [sealed_dwords[at].get(), sealed_dwords[at + 1].get()],
            [a, b],
        );
        w.block[0].set(x);
        w.block[1].set(y);
        // `seal_block` works on the cipher's dwords.
        crate::race::save_seal::seal_block(w);
        a = w.sealed[0].get();
        sealed_dwords[at].set(a);
        b = w.sealed[1].get();
        sealed_dwords[at + 1].set(b);
    }
    w.key[1].set(key[1]);
    w.key[0].set(key[0]);
    Ok(())
}

/// Where the version byte and the player's name stand in the sealed header.
const VERSION_AT: usize = 4;
const PLAYER_NAME_AT: usize = 0xD;

/// `bytes` written into the sealed header from byte `at` (the file's bytes: the dwords hold them
/// little-endian, whatever the alignment).
fn put_header(sealed: &[PlainDword; 64], at: usize, bytes: &[u8]) -> Result<(), BadInput> {
    if at + bytes.len() > sealed.len() * 4 {
        return Err(BadInput("past the replay's sealed header"));
    }
    for (i, &b) in bytes.iter().enumerate() {
        let (dword, shift) = ((at + i) / 4, 8 * ((at + i) % 4));
        let kept = sealed[dword].get() & !(0xFF << shift);
        sealed[dword].set(kept | u32::from(b) << shift);
    }
    Ok(())
}

/// The settings the sealed header copies: the low bytes of each go to its place in the header;
/// the first car's race number also goes to the first dword of `unknown_7d0`, outside the header
/// (a slip kept as is).
#[cfg(test)]
fn copy_header_settings(sealed: &[PlainDword; 64]) -> Result<(), BadInput> {
    copy_header_settings_of(
        crate::test_support::own::race(),
        menus(),
        crate::test_support::own::strings(),
        sealed,
    )
}

/// `copy_header_settings` from the race's and the front end's variables.
fn copy_header_settings_of(
    race: &Race,
    menus: &Menus,
    strings: &crate::strings::owner::Strings,
    sealed: &[PlainDword; 64],
) -> Result<(), BadInput> {
    let shared = &race.shared;
    let settings = &shared.state.settings;
    let car = &settings.cars[0];
    let patches = shared.work.get().patch_settings();
    // (bytes, place in the sealed header, value)
    let cells: [(usize, usize, u32); 30] = [
        (
            1,
            0x2D,
            u32::from(crate::strings::load_language::notice_kind(strings)),
        ),
        (
            4,
            0x5,
            u32::from_le_bytes(
                shared
                    .replays()
                    .header
                    .read_at(0xC, 4, "the replay header's source")?
                    .try_into()
                    .expect("four bytes"),
            ),
        ),
        (1, 0x32, settings.car_kind_current.get()),
        (1, 0x33, settings.stage.get()),
        (1, 0x34, settings.direction.get()),
        (1, 0x35, settings.run_kind.get()),
        (1, 0x36, car.race_number.get()),
        (1, 0x37, car.view.get()),
        (4, 0x9, shared.replay_file().season_code.get()),
        (4, 0x2E, menus.play.cheats.cheats.get()),
        (1, 0x40, car.controls.gearing.get()),
        (1, 0x41, car.gear_ratio.get()),
        (1, 0x42, car.tyre.get()),
        (1, 0x43, car.brake_balance.get()),
        (1, 0x44, car.steering_sensitivity.get()),
        (1, 0x45, car.ride_height.get()),
        (1, 0x46, car.suspension_stiffness.get()),
        (1, 0x47, shared.work.get().patch_switch().get()),
        (1, 0x48, patches[0].get()),
        (1, 0x49, patches[1].get()),
        (2, 0x4A, patches[2].get()),
        (4, 0x4C, patches[3].get()),
        (2, 0x50, patches[4].get()),
        (2, 0x52, patches[5].get()),
        (2, 0x54, patches[6].get()),
        (2, 0x56, patches[7].get()),
        (2, 0x58, patches[8].get()),
        (2, 0x5A, patches[9].get()),
        (2, 0x5C, patches[10].get()),
        (2, 0x5E, patches[11].get()),
    ];
    for (size, at, value) in cells {
        put_header(sealed, at, &value.to_le_bytes()[..size])?;
    }
    // The viewer's first car number, kept outside the header.
    let door = &race.replay_work().unknown_7d0[0];
    door.set(door.get() & !0xFF | (car.race_number.get() & 0xFF));
    Ok(())
}

fn write(handle: u32, bytes: &[u8]) -> (u32, bool) {
    let (n, ok) = crate::files::handle::write_slice(handle, bytes);
    (n, !ok)
}

/// Writes `count` bytes at `from` to file `handle`; unless `chunks_plain` is 1 they are packed
/// (`race::pack_runs`, rounded up to a dword) and the packing is written when smaller. Both
/// callers write the whole replay block (`block`, 0x2810 bytes): here it is that block.
pub fn write_replay_chunk(
    race: &Race,
    system: &System,
    handle: u32,
) -> Result<(u32, bool), Refused> {
    let shared = &race.shared;
    let block = race.viewer_buffers().block_bytes();
    // A replay with the extra tape has its block's own size (the index's plain size).
    let plain_len = block.len() as u32;
    if shared.replay_file().chunks_plain.get() == 1 {
        return Ok(write(handle, &block));
    }
    let (packing, packed) = crate::race::save_seal::pack_runs(&block, system)?;
    // Logic in `rc2k_core::race::replay::packed_size`.
    Ok(
        match rc2k_core::race::replay::packed_size(packed, plain_len) {
            Some(size) => {
                // The size rounded up to a dword: within the end mark's three zeros.
                let (n, failed) = write(handle, &packing[..size as usize]);
                if failed {
                    (n, true)
                } else {
                    (n.wrapping_neg(), false)
                }
            }
            None => write(handle, &block),
        },
    )
}

/// The replay index's dword `k` (offset and size a block). A block count past
/// the index's 1800 (a broken file) reads on past it: the index ends the file block's record, so
/// there is nothing to read and it is refused.
pub fn block_place(race: &Race, k: usize) -> Result<u32, BadInput> {
    race.shared
        .replay_file()
        .index
        .block_places
        .get(k)
        .map(crate::memory::session::PlainDword::get)
        .ok_or(BadInput("the replay index: a block past the 1800"))
}

/// Writes the block (`block`, 0x2810 bytes) as a chunk; its offset and size (negative when packed)
/// go into the index, and the file's length and the block count move on. A failed write or the
/// index full (0x708 blocks) closes the replay.
pub fn write_replay_block(race: &Race, system: &System, replay: &Replay) -> Result<(), Refused> {
    let shared = &race.shared;
    let n = shared.replay_file().index.block_count.get();
    if n >= MOST_BLOCKS {
        return Err(BadInput("the replay's index is full already").into());
    }
    shared.replay_file().index.block_places[2 * n as usize]
        .set(replay.replay_view().file_offset.get());
    let (written, failed) =
        write_replay_chunk(race, system, shared.replay_file().replay_handle.get())?;
    let failed = failed || {
        shared.replay_file().index.block_places[2 * n as usize + 1].set(written);
        // Logic in `rc2k_core::race::replay::block_written`.
        let (size, full) = rc2k_core::race::replay::block_written(written, n);
        replay
            .replay_view()
            .file_offset
            .set(replay.replay_view().file_offset.get().wrapping_add(size));
        shared.replay_file().index.block_count.set(n + 1);
        full
    };
    if failed {
        close_replay(race, system)?;
    }
    Ok(())
}

/// With the replay open, puts `bytes` bytes at `from` (a frame when 0x10: the frame count moves
/// on) at the cursor (`bytes` / 4 dwords); at 100 frames the block is written
/// (`race::write_replay_block`) and the cursor and count go back. Refuses frames past the block's
/// room (before anything). Its one caller adds the padding frame (the replay view's
/// `pad_frame`): here the frame's cells, `bytes` their size.
pub fn add_replay_frame(race: &Race, system: &System, frame: &[PlainDword]) -> Result<(), Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    if !shared.replay_file().replay_open.is_set() {
        return Ok(());
    }
    // Logic in `rc2k_core::race::replay::{fits, frame_added}`.
    use rc2k_core::race::replay::{FRAME_BYTES, fits, frame_added};
    let bytes = 4 * frame.len() as u32;
    let used = race.viewer_buffers().block_used.get();
    let dwords = bytes >> 2;
    if !fits(used, dwords, BLOCK_LEN) {
        return Err(BadInput("a frame past the replay block's room").into());
    }
    let (in_block, full) = frame_added(bytes, replay.replay_view().frames_in_block.get());
    if bytes == FRAME_BYTES {
        shared
            .replay_file()
            .index
            .frame_count
            .set(shared.replay_file().index.frame_count.get().wrapping_add(1));
        replay.replay_view().frames_in_block.set(in_block);
    }
    let first = used as usize / 4;
    for (k, cell) in frame.iter().enumerate() {
        race.viewer_buffers().block[first + k].set(cell.get());
    }
    race.viewer_buffers().block_used.set(used + bytes);
    if full {
        write_replay_block(race, system, replay)?;
        race.viewer_buffers().block_used.set(0);
        replay.replay_view().frames_in_block.set(0);
    }
    Ok(())
}

/// With the replay open, fills the block up with the padding frame (`race::add_replay_frame`,
/// which writes it at 100), sets the last block's frame count (`frames_in_block`: 100, or for the
/// last block the frames modulo 100 plus one), seals the time, writes the index ("RCIB", 0x3850
/// bytes), closes the file and sets `replay_open` 0 and `replay_closed` 1.
pub fn close_replay(race: &Race, system: &System) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    if !shared.replay_file().replay_open.is_set() {
        return Ok(());
    }
    while replay.replay_view().frames_in_block.get() != 0 {
        add_replay_frame(race, system, &replay.replay_view().pad_frame)?;
    }
    // Logic in `rc2k_core::race::replay::closing_frames`.
    let (frames, _, _) = rc2k_core::race::replay::closing_frames(
        replay.replay_view().replay_block.get(),
        shared.replay_file().index.block_count.get(),
        shared.replay_file().index.frame_count.get(),
    );
    replay.replay_view().frames_in_block.set(frames);
    crate::race::race_shell::seal_stage_time(race)?;
    crate::files::handle::write_cells(
        shared.replay_file().replay_handle.get(),
        shared.replay_file().index.dwords(),
    );
    crate::files::handle::close(shared.replay_file().replay_handle.get());
    shared.replay_file().replay_open.set(false);
    shared.replay_file().replay_closed.set(1);
    Ok(())
}

/// Opens the replay: `replay_closed` 0; with replays on (`replays_on` 1, `no_replay_switch` 0,
/// `RaceState.settings.no_replay` not 1) and 30 MB free it makes the header
/// ("REPL", version 0x50400, or 0x50401 with the extra tape, the date, "tmpreplay", the race's settings), copies the car's header
/// (`seal_replay_header`) into it, clears the index "RCIB", sets the cursor, counts and length to
/// 0, creates `var\tmprpl` and writes the header. A failed open or write closes it
/// (`race::close_replay`).
pub fn open_replay(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    // Logic in `rc2k_core::race::replay::{replays_on, room_for_replay}` and the header's and the
    // index's words.
    use rc2k_core::drive::replay_frames::records_extra_tape;
    use rc2k_core::race::replay::{
        HEADER_MAGIC, HEADER_NAME, INDEX_MAGIC, header_version, replays_on, room_for_replay,
    };
    shared.replay_file().replay_closed.set(0);
    // A local race of more than two players keeps the inputs of cars 2 and 3 on an extra tape
    // (a replay of the version that has it); every other race is written as ever.
    let extra_tape = records_extra_tape(shared.view.get().players.get(), o.net.race_slot.is_on());
    race.viewer_buffers().extra_tape.set(extra_tape);
    replay.staging().clear_extra_tape();
    if !replays_on(
        shared.replay_file().replays_on.get(),
        shared.replay_file().no_replay_switch.get(),
        shared.state.settings.no_replay.get(),
    ) {
        return Ok(());
    }
    if !room_for_replay(crate::logfile::start_log::free_space_kb()) {
        return Ok(());
    }
    let header = &shared.replays().header;
    for k in 0..0x10C {
        header.set_u32(4 * k, 0);
    }
    header.set_u32(0, HEADER_MAGIC);
    header.set_u32(8, header_version(extra_tape));
    header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
    for (k, &w) in HEADER_NAME.iter().enumerate() {
        header.set_u32(0x10 + 4 * k, w);
    }
    let w = race.replay_work();
    w.header_one_player
        .set(o.sound.threads.shared.one_player.get());
    w.header_codriver_sex_0
        .set(shared.state.settings.cars[0].codriver_sex.get());
    w.header_codriver_sex_1
        .set(shared.state.settings.cars[1].codriver_sex.get());
    w.patches_in_header.set(6);
    for (to, from) in w
        .header_patches
        .iter()
        .zip(crate::race::replay_patches::switches(shared))
    {
        to.set(from.get());
    }
    seal_replay_header(race, o.menus, o.strings)?;
    // The car header (0x400 bytes from the stage setting on through the sealed part, the patch
    // switches and the door: all the dwords of the stretch) copied into the file's header at +0x30.
    let w = race.replay_work();
    let stage: Vec<u8> = [
        &w.header_stage,
        &w.header_mode,
        &w.header_players,
        &w.header_cars,
        &w.header_kind_0,
        &w.header_kind_1,
        &w.header_kind_2,
        &w.header_kind_3,
        &w.header_one_player,
        &w.header_codriver_sex_0,
        &w.header_codriver_sex_1,
        &w.header_start_place,
        &w.header_arcade_cars,
        &w.header_seed,
        &w.header_game_over_0,
        &w.header_game_over_1,
    ]
    .into_iter()
    .chain(&w.sealed_header)
    .chain(std::iter::once(&w.patches_in_header))
    .chain(&w.header_patches)
    .chain(&w.unknown_7d0)
    .flat_map(|d| d.get().to_le_bytes())
    .collect();
    debug_assert_eq!(stage.len(), 0x400);
    shared
        .replays()
        .header
        .write_at(0x30, &stage, "the replay header's stage")?;
    for cell in shared.replay_file().index.dwords() {
        cell.set(0);
    }
    shared.replay_file().index.magic.set(INDEX_MAGIC);
    race.viewer_buffers().block_used.set(0);
    replay.replay_view().frames_in_block.set(0);
    replay.replay_view().file_offset.set(0);
    let opened =
        crate::files::handle::open_by_mode(&crate::race::replay_path::last_race(replay), 1)?;
    let failed = match opened {
        None => true,
        Some(handle) => {
            shared.replay_file().replay_handle.set(handle);
            shared.replay_file().replay_open.set(true);
            replay.replay_view().file_offset.set(
                replay
                    .replay_view()
                    .file_offset
                    .get()
                    .wrapping_add(HEADER_LEN),
            );
            !crate::files::handle::write_slice(
                shared.replay_file().replay_handle.get(),
                &shared.replays().header.read(),
            )
            .1
        }
    };
    if failed {
        close_replay(race, o.system)?;
    }
    Ok(())
}

#[cfg(test)]
mod seal_key_tests {
    use crate::test_support::own::race;

    #[test]
    fn seal_key_starts_at_zero() {
        assert_eq!(race().replay.seal_key().get(), 0);
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;
    use crate::memory::session::DwordLike;
    use crate::test_support::own::race;

    const LOCALE: &str = "owners.strings.language.locale";
    const REPLAYS: &str = "owners.race.shared.replays";
    const REPLAY_FILE: &str = "owners.race.shared.replay_file";
    const WORK: &str = "owners.race.shared.work";
    const STATE: &str = "owners.race.shared.state";
    const CHEATS: &str = "owners.menus.play.cheats";
    #[derive(Clone, Copy)]
    enum Dest {
        /// At a byte of the sealed header part.
        Sealed(usize),
        /// The first byte of the co-driver's door.
        Door,
    }
    use Dest::{Door, Sealed};

    /// Where a copy reads: a cell (its low bytes), or bytes of a byte array from an offset.
    #[derive(Clone, Copy)]
    enum Source {
        /// The byte array at the field `path` (empty: the store itself), from byte `offset`.
        Bytes(&'static str, usize),
        /// The cell at the field `path`.
        Cell(&'static str),
    }
    use Source::{Bytes, Cell};

    /// The header's copies: (bytes, where they are written, the store read and the field in it).
    /// The header's own layout is the file's, so the destinations stay byte positions.
    const HEADER_MOVES: [(usize, Dest, &str, Source); 31] = [
        (1, Sealed(0x2D), LOCALE, Bytes("", 8)),
        (4, Sealed(0x5), REPLAYS, Bytes(".header", 0xC)),
        (1, Sealed(0x32), STATE, Cell(".settings.car_kind_current")),
        (1, Sealed(0x33), STATE, Cell(".settings.stage")),
        (1, Sealed(0x34), STATE, Cell(".settings.direction")),
        (1, Sealed(0x35), STATE, Cell(".settings.run_kind")),
        (
            1,
            Sealed(0x36),
            STATE,
            Cell(".settings.cars[0].own.race_number"),
        ),
        (1, Door, STATE, Cell(".settings.cars[0].own.race_number")),
        (1, Sealed(0x37), STATE, Cell(".settings.cars[0].own.view")),
        (4, Sealed(0x9), REPLAY_FILE, Cell(".season_code")),
        (4, Sealed(0x2E), CHEATS, Cell(".cheats")),
        (
            1,
            Sealed(0x40),
            STATE,
            Cell(".settings.cars[0].controls.gearing"),
        ),
        (
            1,
            Sealed(0x41),
            STATE,
            Cell(".settings.cars[0].own.gear_ratio"),
        ),
        (1, Sealed(0x42), STATE, Cell(".settings.cars[0].own.tyre")),
        (
            1,
            Sealed(0x43),
            STATE,
            Cell(".settings.cars[0].own.brake_balance"),
        ),
        (
            1,
            Sealed(0x44),
            STATE,
            Cell(".settings.cars[0].own.steering_sensitivity"),
        ),
        (
            1,
            Sealed(0x45),
            STATE,
            Cell(".settings.cars[0].own.ride_height"),
        ),
        (
            1,
            Sealed(0x46),
            STATE,
            Cell(".settings.cars[0].own.suspension_stiffness"),
        ),
        (1, Sealed(0x47), WORK, Cell(".patches[0]")),
        (1, Sealed(0x48), WORK, Cell(".patches[1]")),
        (1, Sealed(0x49), WORK, Cell(".patches[2]")),
        (2, Sealed(0x4A), WORK, Cell(".patches[3]")),
        (4, Sealed(0x4C), WORK, Cell(".patches[4]")),
        (2, Sealed(0x50), WORK, Cell(".patches[5]")),
        (2, Sealed(0x52), WORK, Cell(".patches[6]")),
        (2, Sealed(0x54), WORK, Cell(".patches[7]")),
        (2, Sealed(0x56), WORK, Cell(".patches[8]")),
        (2, Sealed(0x58), WORK, Cell(".patches[9]")),
        (2, Sealed(0x5A), WORK, Cell(".patches[10]")),
        (2, Sealed(0x5C), WORK, Cell(".patches[11]")),
        (2, Sealed(0x5E), WORK, Cell(".patches[12]")),
    ];

    fn old_copy() -> Result<(), BadInput> {
        for &(size, to, from, source) in &HEADER_MOVES {
            let walk = crate::test_support::state::walk(from);
            let (path, skip) = match source {
                Bytes(path, skip) => (path, skip),
                Cell(path) => (path, 0),
            };
            let cells = walk.path_bytes(path).expect("a field of the store");
            let bytes = cells[skip..skip + size].to_vec();
            let work = race().replay_work();
            match to {
                Sealed(at) => crate::memory::cells::Store::store(&work.sealed_header, at, &bytes)?,
                Door => crate::memory::cells::Store::store(&work.unknown_7d0[0], 0, &bytes)?,
            }
        }
        Ok(())
    }

    /// Splitmix64.
    fn next(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    #[test]
    fn the_typed_copy_makes_the_header_the_old_loop_made() {
        let _g = crate::test_support::test_lock();
        let shared = &race().shared;
        let settings = &shared.state.settings;
        let car = &settings.cars[0];
        let mut cells: Vec<&dyn DwordLike> = vec![
            &settings.stage,
            &settings.run_kind,
            &settings.car_kind_current,
            &settings.direction,
            &car.race_number,
            &car.view,
            &car.controls.gearing,
            &car.gear_ratio,
            &car.tyre,
            &car.brake_balance,
            &car.steering_sensitivity,
            &car.ride_height,
            &car.suspension_stiffness,
            &shared.replay_file().season_code,
            &menus().play.cheats.cheats,
        ];
        cells.extend(
            shared
                .work
                .get()
                .patches
                .iter()
                .map(|c| c as &dyn DwordLike),
        );
        let locale = &crate::test_support::own::strings().language.locale;
        let header = &shared.replays().header;
        let sealed = &crate::test_support::own::race().replay_work().sealed_header;
        let door = &crate::test_support::own::race().replay_work().unknown_7d0[0];
        let kept_cells: Vec<u32> = cells.iter().map(|c| c.get()).collect();
        let kept_locale = locale.get(8);
        let kept_header: Vec<u8> = (0..16).map(|i| header.get(i)).collect();
        let kept_sealed: Vec<u32> = sealed.iter().map(PlainDword::get).collect();
        let kept_door = door.get();
        for seed in 1..=8u64 {
            let mut state = seed;
            for c in &cells {
                c.set(next(&mut state) as u32);
            }
            locale.set(8, next(&mut state) as u8);
            for i in 0..16 {
                header.set(i, next(&mut state) as u8);
            }
            let run = |typed: bool| {
                for d in sealed.iter() {
                    d.set(0xAAAA_AAAA);
                }
                door.set(0x5555_5555);
                if typed {
                    copy_header_settings(sealed).unwrap();
                } else {
                    old_copy().unwrap();
                }
                let mut out: Vec<u32> = sealed.iter().map(PlainDword::get).collect();
                out.push(door.get());
                out
            };
            let old = run(false);
            let new = run(true);
            assert_eq!(old, new, "seed {seed}");
            assert_ne!(old[..64], [0xAAAA_AAAA; 64], "seed {seed}: nothing copied");
        }
        for (c, v) in cells.iter().zip(kept_cells) {
            c.set(v);
        }
        locale.set(8, kept_locale);
        for (i, b) in kept_header.into_iter().enumerate() {
            header.set(i, b);
        }
        for (d, v) in sealed.iter().zip(kept_sealed) {
            d.set(v);
        }
        door.set(kept_door);
    }
}
