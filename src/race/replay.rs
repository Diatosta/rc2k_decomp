//! The replay file (`var\tmprpl`): opened with its header at the race's
//! start (`race::open_replay`, the header sealed by `race::seal_replay_header`), frames
//! gathered (`race::add_replay_frame`), written block by block (`race::write_replay_block`
//! through `race::write_replay_chunk`, which packs by runs), and closed with its index
//! ("RCIB", `race::close_replay`). The steps call each other (a failed block closes the
//! file, which writes the last frames as a block); the file primitives are
//! `files::handle`'s.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::net::owner::net;
use crate::pointers::GamePtr;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::owner_shared::RaceShared;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;
use crate::session::{Bytes, Dword};

/// The race record's seal key (the replay header's checksum, `race::seal_replay_header`;
/// `race::seal_stage_time` turns the first car's time by it). It is what makes a stage time
/// in the saved results depend on the replay it came from, so a time cannot be edited alone.
/// Kept apart from the sound dwords around it; `moved::MOVED` names IDA 0x4F66F9 here
/// (`race::owner_replay::Replay::seal_key`).
// The index (`crate::race::owner::race().shared.replay_file.index()`) is 0x3850 bytes.
const _: () = assert!(crate::race::replay_file_store::INDEX_DWORDS * 4 == 0x3850);
pub const MOST_BLOCKS: u32 = rc2k_core::race::replay::MOST_BLOCKS;
pub const BLOCK_LEN: u32 = 0x2810;
pub const HEADER_LEN: u32 = 0x430;

/// IDA's address of the replays' buffers.
pub const START: usize = 0x528504;
/// Their length.
pub const LEN: usize = 0x5295B4 - START;
/// A replay's name: 0x20 bytes.
pub const NAME_BYTES: usize = 0x20;
/// The replays that can be saved.
pub const SLOTS: usize = 100;

/// The replays' buffers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Replays {
    /// `dword_528504`: the file's header (a dword array, `HEADER_LEN` bytes).
    pub header: Bytes<{ HEADER_LEN as usize }>,
    /// `byte_528934`: the saved replays' names, 100 of 0x20 bytes (twenty to a page).
    pub names: Bytes<{ SLOTS * NAME_BYTES }>,
}

impl Replays {
    /// The replays' buffers; the first values are the image's (all zero).
    pub const fn new() -> Self {
        Replays {
            header: Bytes::new([0; HEADER_LEN as usize]),
            names: Bytes::new([0; SLOTS * NAME_BYTES]),
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use crate::race::owner::race;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&race().shared.replays, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        let shared = &race().shared;
        assert_eq!(offset_of!(Replays, names), 0x528934 - START);
        assert_eq!(
            crate::moved::place(START),
            shared.replays.header.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x528934),
            shared.replays.names.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x528510),
            (shared.replays.header.addr() + 0xC) as usize
        );
    }
}

/// `race::seal_replay_header` (0x41F0A3): the replay's sealed header part (`sealed_header`, 0x40
/// dwords): version byte 6, the player's name (`strings::copy_printable`, 0x20), the settings and
/// options copied (`copy_header_settings`), a checksum (from dword 1: add, `rol` 5) into dword 0 and
/// `dword_4F66F9`; then its 32 blocks of 8 chained through `race::seal_block` with the key
/// "halfmoon" (the game's key put back after).
pub fn seal_replay_header(replay: &Replay) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::replay::{seal_checksum, chained, SEAL_KEY}`.
    use rc2k_core::race::replay::{SEAL_KEY, SEAL_VERSION, chained, seal_checksum};
    // The sealed header is the file's bytes: the version byte and the name sit inside its
    // dwords.
    let sealed = GamePtr::to(&REPLAY_WORK.sealed_header);
    sealed
        .offset(0x719B58 - 0x719B54)
        .write::<u8>(SEAL_VERSION, "the replay header's version")?;
    crate::strings::copy_printable::copy_printable(
        crate::pointers::GamePtr::to(&menus().play.names.player_names), // g_playerNames
        sealed.offset(0x719B61 - 0x719B54),
        0x20,
    )?;
    copy_header_settings(sealed)?;
    let sealed_dwords = &REPLAY_WORK.sealed_header;
    let dwords: Vec<u32> = (1..0x40).map(|n| sealed_dwords[n].get()).collect();
    let sum = seal_checksum(&dwords);
    sealed_dwords[0].set(sum);
    replay.seal_key.set(sum);
    let w = &replay.seal_work;
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
        // `seal_block` works on the dwords at 0x4E052C .. (no register read).
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

/// The settings the sealed header copies, in the original's order: the low `size` bytes of
/// each setting go to `at` (an offset from the header's first byte, 0x138 and more past the
/// end of its 0x100 bytes are the first dword of `unknown_7d0`: the original's slip).
fn copy_header_settings(sealed: GamePtr) -> Result<(), BadInput> {
    use crate::race::state_store::{RaceCar, RaceSettings, RaceState};
    use core::mem::offset_of;
    // The cells of the race's settings as the front end keeps them, by IDA's address.
    const SETTINGS: usize = offset_of!(RaceState, settings);
    const CARS: usize = SETTINGS + offset_of!(RaceSettings, cars);
    const fn setting(ida: usize) -> usize {
        ida - 0x75ACCC
    }
    const _: () = {
        assert!(SETTINGS + offset_of!(RaceSettings, stage) == setting(0x75ACD4));
        assert!(SETTINGS + offset_of!(RaceSettings, run_kind) == setting(0x75ACD8));
        assert!(SETTINGS + offset_of!(RaceSettings, car_kind_current) == setting(0x75ACE8));
        assert!(SETTINGS + offset_of!(RaceSettings, direction) == setting(0x75ACF0));
        assert!(CARS + offset_of!(RaceCar, tyre) == setting(0x75AD1C));
        assert!(CARS + offset_of!(RaceCar, brake_balance) == setting(0x75AD20));
        assert!(CARS + offset_of!(RaceCar, ride_height) == setting(0x75AD24));
        assert!(CARS + offset_of!(RaceCar, suspension_stiffness) == setting(0x75AD28));
        assert!(CARS + offset_of!(RaceCar, steering_sensitivity) == setting(0x75AD2C));
        assert!(CARS + offset_of!(RaceCar, gear_ratio) == setting(0x75AD30));
        assert!(CARS + offset_of!(RaceCar, race_number) == setting(0x75AE1C));
        assert!(CARS + offset_of!(RaceCar, view) == setting(0x75AE24));
        assert!(CARS + offset_of!(RaceCar, gearing) == setting(0x75AE2C));
    };
    let shared = &crate::race::owner::race().shared;
    let settings = &shared.state.settings;
    let car = &settings.cars[0];
    let patches = &shared.work.patch_settings;
    let cells: [(usize, usize, u32); 31] = [
        (
            1,
            0x719B81,
            u32::from(crate::strings::load_language::notice_kind()),
        ),
        (
            4,
            0x719B59,
            u32::from_le_bytes(
                shared
                    .replays
                    .header
                    .read_at(0xC, 4, "the replay header's source")?
                    .try_into()
                    .expect("four bytes"),
            ),
        ),
        (1, 0x719B86, settings.car_kind_current.get()),
        (1, 0x719B87, settings.stage.get()),
        (1, 0x719B88, settings.direction.get()),
        (1, 0x719B89, settings.run_kind.get()),
        (1, 0x719B8A, car.race_number.get()),
        (1, 0x719C8C, car.race_number.get()),
        (1, 0x719B8B, car.view.get()),
        (4, 0x719B5D, shared.replay_file.season_code.get()),
        (4, 0x719B82, menus().play.cheats.cheats.get()),
        (1, 0x719B94, car.gearing.get()),
        (1, 0x719B95, car.gear_ratio.get()),
        (1, 0x719B96, car.tyre.get()),
        (1, 0x719B97, car.brake_balance.get()),
        (1, 0x719B98, car.steering_sensitivity.get()),
        (1, 0x719B99, car.ride_height.get()),
        (1, 0x719B9A, car.suspension_stiffness.get()),
        (1, 0x719B9B, shared.work.patch_switch.get()),
        (1, 0x719B9C, patches[0].get()),
        (1, 0x719B9D, patches[1].get()),
        (2, 0x719B9E, patches[2].get()),
        (4, 0x719BA0, patches[3].get()),
        (2, 0x719BA4, patches[4].get()),
        (2, 0x719BA6, patches[5].get()),
        (2, 0x719BA8, patches[6].get()),
        (2, 0x719BAA, patches[7].get()),
        (2, 0x719BAC, patches[8].get()),
        (2, 0x719BAE, patches[9].get()),
        (2, 0x719BB0, patches[10].get()),
        (2, 0x719BB2, patches[11].get()),
    ];
    for (size, to, value) in cells {
        let bytes = &value.to_le_bytes()[..size];
        if to < 0x719B54 + 0x100 {
            sealed
                .offset((to - 0x719B54) as u32)
                .write_bytes(bytes, "the replay header")?;
        } else {
            // 0x719C8C: the viewer's first car number, past the header.
            let door = &REPLAY_WORK.unknown_7d0[0];
            door.set(door.get() & !0xFF | u32::from(bytes[0]));
        }
    }
    Ok(())
}

fn write(handle: u32, bytes: &[u8]) -> (u32, bool) {
    let (n, ok) = crate::files::handle::write_slice(handle, bytes);
    (n, !ok)
}

/// `race::write_replay_chunk` (0x41EEE4): `count` bytes at `from` written to file `handle`; unless
/// `chunks_plain` is 1 they are packed (`race::pack_runs` into `g_pngInfo`, rounded up to a dword)
/// and the packing written when smaller. Both callers write the whole replay block (`block`,
/// 0x2810 bytes): here it is that block.
pub fn write_replay_chunk(shared: &RaceShared, handle: u32) -> Result<(u32, bool), Refused> {
    let system = crate::system::owner::system();
    let block = VIEWER_BUFFERS.block_bytes();
    if shared.replay_file.chunks_plain.get() == 1 {
        return Ok(write(handle, &block));
    }
    let (packing, packed) = crate::race::save_seal::pack_runs(&block, system)?;
    // Converted: `rc2k_core::race::replay::packed_size`.
    Ok(
        match rc2k_core::race::replay::packed_size(packed, BLOCK_LEN) {
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

/// The replay index's dword `k` (`lDistanceToMove` ..: offset and size a block). A block
/// count past the index's 1800 (a broken file) reads on past it, as the original does.
pub fn block_place(race: &Race, k: usize) -> Result<u32, BadInput> {
    let shared = &race.shared;
    let places = &shared.replay_file.block_places;
    crate::session::read_on_in(&shared.replay_file, places, k, "the replay index")
}

/// `race::write_replay_block` (0x41E9A3): the block (`block`, 0x2810 bytes) written as a
/// chunk; its offset and size (negative when packed) into the index; the file's length on; the
/// block count on. A failed write or the index full (0x708 blocks) closes the replay.
pub fn write_replay_block(race: &Race, replay: &Replay) -> Result<(), Refused> {
    let shared = &race.shared;
    let n = shared.replay_file.index_block_count.get();
    if n >= MOST_BLOCKS {
        return Err(BadInput("the replay's index is full already").into());
    }
    shared.replay_file.block_places[2 * n as usize].set(replay.replay_view.file_offset.get());
    let (written, failed) = write_replay_chunk(shared, shared.replay_file.replay_handle.get())?;
    let failed = failed || {
        shared.replay_file.block_places[2 * n as usize + 1].set(written);
        // Converted: `rc2k_core::race::replay::block_written`.
        let (size, full) = rc2k_core::race::replay::block_written(written, n);
        replay
            .replay_view
            .file_offset
            .set(replay.replay_view.file_offset.get().wrapping_add(size));
        shared.replay_file.index_block_count.set(n + 1);
        full
    };
    if failed {
        close_replay(race)?;
    }
    Ok(())
}

/// `race::add_replay_frame` (0x41E8DE): with the replay open, `bytes` bytes at `from` (a frame when
/// 0x10: the frame counts on) put at the cursor (`rep movsd`, `bytes` / 4 dwords); at 100 frames
/// the block written (`race::write_replay_block`) and the cursor and count back. Refuses frames
/// past the block's room (before anything). Its one caller adds the padding frame
/// (`REPLAY_VIEW.pad_frame`): here the frame's cells, `bytes` their size.
pub fn add_replay_frame(race: &Race, frame: &[Dword]) -> Result<(), Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    if !shared.replay_file.replay_open.is_set() {
        return Ok(());
    }
    // Converted: `rc2k_core::race::replay::{fits, frame_added}`.
    use rc2k_core::race::replay::{FRAME_BYTES, fits, frame_added};
    let bytes = 4 * frame.len() as u32;
    let used = VIEWER_BUFFERS.block_used.get();
    let dwords = bytes >> 2;
    if !fits(used, dwords, BLOCK_LEN) {
        return Err(BadInput("a frame past the replay block's room").into());
    }
    let (in_block, full) = frame_added(bytes, replay.replay_view.frames_in_block.get());
    if bytes == FRAME_BYTES {
        shared
            .replay_file
            .index_frame_count
            .set(shared.replay_file.index_frame_count.get().wrapping_add(1));
        replay.replay_view.frames_in_block.set(in_block);
    }
    let first = used as usize / 4;
    for (k, cell) in frame.iter().enumerate() {
        VIEWER_BUFFERS.block[first + k].set(cell.get());
    }
    VIEWER_BUFFERS.block_used.set(used + bytes);
    if full {
        write_replay_block(race, replay)?;
        VIEWER_BUFFERS.block_used.set(0);
        replay.replay_view.frames_in_block.set(0);
    }
    Ok(())
}

/// `race::close_replay` (0x41E85A): with the replay open, the block filled up with the padding
/// frame (`race::add_replay_frame`, which writes it at 100); the last block's frame count
/// (`frames_in_block`: 100, or for the last block the frames modulo 100 plus one); the time sealed;
/// the index ("RCIB", 0x3850 bytes) written; the file closed; `replay_open` 0, `replay_closed` 1.
pub fn close_replay(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    if !shared.replay_file.replay_open.is_set() {
        return Ok(());
    }
    while replay.replay_view.frames_in_block.get() != 0 {
        add_replay_frame(race, &replay.replay_view.pad_frame)?;
    }
    // Converted: `rc2k_core::race::replay::closing_frames`.
    let (frames, _, _) = rc2k_core::race::replay::closing_frames(
        replay.replay_view.replay_block.get(),
        shared.replay_file.index_block_count.get(),
        shared.replay_file.index_frame_count.get(),
    );
    replay.replay_view.frames_in_block.set(frames);
    crate::race::race_shell::seal_stage_time(race)?;
    crate::files::handle::write_cells(
        shared.replay_file.replay_handle.get(),
        shared.replay_file.index(),
    );
    crate::files::handle::close(shared.replay_file.replay_handle.get());
    shared.replay_file.replay_open.set(false);
    shared.replay_file.replay_closed.set(1);
    Ok(())
}

/// `race::open_replay` (0x41E6EC): `replay_closed` 0; with replays on (`replays_on` 1,
/// `no_replay_switch` 0, `session` not 1, `dword_75ACDC` not 1) and 30 MB free: the header
/// ("REPL", version 0x50400, the date, "tmpreplay", the race's settings), the car's header
/// (`seal_replay_header`) copied into it, the index "RCIB" cleared, the cursor, counts and length
/// 0; `var\tmprpl` created; the header written. A failed open or write closes it
/// (`race::close_replay`).
pub fn open_replay(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    // Converted: `rc2k_core::race::replay::{replays_on, room_for_replay}` and the
    // header's and the index's words.
    use rc2k_core::race::replay::{
        HEADER_MAGIC, HEADER_NAME, HEADER_VERSION, INDEX_MAGIC, replays_on, room_for_replay,
    };
    shared.replay_file.replay_closed.set(0);
    if !replays_on(
        shared.replay_file.replays_on.get(),
        shared.replay_file.no_replay_switch.get(),
        net().vars.session.get(),
        shared.state.settings.no_replay.get(),
    ) {
        return Ok(());
    }
    if !room_for_replay(crate::logfile::start_log::free_space_kb()) {
        return Ok(());
    }
    let header = &shared.replays.header;
    for k in 0..0x10C {
        header.set_u32(4 * k, 0);
    }
    header.set_u32(0, HEADER_MAGIC);
    header.set_u32(8, HEADER_VERSION);
    header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
    for (k, &w) in HEADER_NAME.iter().enumerate() {
        header.set_u32(0x10 + 4 * k, w);
    }
    let w = &REPLAY_WORK;
    w.header_one_player
        .set(crate::sound::owner::sound().race.sounds.one_player.get());
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
    seal_replay_header(replay)?;
    // The car header (0x400 bytes from `g_headerStage`, run on past the store's first
    // field) copied into the file's header at +0x30.
    let stage = GamePtr::to(&REPLAY_WORK.header_stage)
        .unbounded()
        .read_bytes(0x400, "the replay header's stage")?;
    shared
        .replays
        .header
        .write_at(0x30, &stage, "the replay header's stage")?;
    for cell in shared.replay_file.index() {
        cell.set(0);
    }
    shared.replay_file.index_magic.set(INDEX_MAGIC);
    VIEWER_BUFFERS.block_used.set(0);
    replay.replay_view.frames_in_block.set(0);
    replay.replay_view.file_offset.set(0);
    let opened =
        crate::files::handle::open_by_mode(&crate::race::replay_path::last_race(replay), 1)?;
    let failed = match opened {
        None => true,
        Some(handle) => {
            shared.replay_file.replay_handle.set(handle);
            shared.replay_file.replay_open.set(true);
            replay.replay_view.file_offset.set(
                replay
                    .replay_view
                    .file_offset
                    .get()
                    .wrapping_add(HEADER_LEN),
            );
            !crate::files::handle::write_slice(
                shared.replay_file.replay_handle.get(),
                &shared.replays.header.read(),
            )
            .1
        }
    };
    if failed {
        close_replay(race)?;
    }
    Ok(())
}

#[cfg(test)]
mod seal_key_tests {
    use crate::race::owner::race;

    #[test]
    fn seal_key_starts_as_the_games() {
        assert_eq!(crate::moved::image_bytes(0x4F66F9, 4), [0; 4]);
        assert_eq!(race().replay.seal_key.get(), 0);
    }

    #[test]
    fn sealed_header_sits_at_the_games_address() {
        use crate::race::replay_work_store::{ReplayWork, START};
        assert_eq!(
            core::mem::offset_of!(ReplayWork, sealed_header),
            0x719B54 - START
        );
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;
    use crate::race::owner::race;
    use crate::session::DwordLike;

    /// The header's copies as the first typed-by-address version did them (IDA names).
    const HEADER_MOVES: [(usize, usize, usize); 31] = [
        (1, 0x719B81, 0x4E07A0),
        (4, 0x719B59, 0x528510),
        (1, 0x719B86, 0x75ACE8),
        (1, 0x719B87, 0x75ACD4),
        (1, 0x719B88, 0x75ACF0),
        (1, 0x719B89, 0x75ACD8),
        (1, 0x719B8A, 0x75AE1C),
        (1, 0x719C8C, 0x75AE1C),
        (1, 0x719B8B, 0x75AE24),
        (4, 0x719B5D, 0x5BECAC),
        (4, 0x719B82, 0x75870C),
        (1, 0x719B94, 0x75AE2C),
        (1, 0x719B95, 0x75AD30),
        (1, 0x719B96, 0x75AD1C),
        (1, 0x719B97, 0x75AD20),
        (1, 0x719B98, 0x75AD2C),
        (1, 0x719B99, 0x75AD24),
        (1, 0x719B9A, 0x75AD28),
        (1, 0x719B9B, 0x71C6D0),
        (1, 0x719B9C, 0x71C6D4),
        (1, 0x719B9D, 0x71C6D8),
        (2, 0x719B9E, 0x71C6DC),
        (4, 0x719BA0, 0x71C6E0),
        (2, 0x719BA4, 0x71C6E4),
        (2, 0x719BA6, 0x71C6E8),
        (2, 0x719BA8, 0x71C6EC),
        (2, 0x719BAA, 0x71C6F0),
        (2, 0x719BAC, 0x71C6F4),
        (2, 0x719BAE, 0x71C6F8),
        (2, 0x719BB0, 0x71C6FC),
        (2, 0x719BB2, 0x71C700),
    ];

    fn old_copy() -> Result<(), BadInput> {
        for &(size, to, from) in &HEADER_MOVES {
            let (to, from) = (crate::moved::named(to)?, crate::moved::named(from)?);
            from.copy_to(to, size, "the replay header")?;
        }
        Ok(())
    }

    /// splitmix64.
    fn next(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    #[test]
    fn the_typed_copy_makes_the_header_the_old_loop_made() {
        let _g = crate::store::test_lock();
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
            &car.gearing,
            &car.gear_ratio,
            &car.tyre,
            &car.brake_balance,
            &car.steering_sensitivity,
            &car.ride_height,
            &car.suspension_stiffness,
            &shared.replay_file.season_code,
            &menus().play.cheats.cheats,
            &shared.work.patch_switch,
        ];
        cells.extend(
            shared
                .work
                .patch_settings
                .iter()
                .map(|c| c as &dyn DwordLike),
        );
        let locale = &crate::strings::owner::strings().language.locale;
        let header = &shared.replays.header;
        let sealed = &REPLAY_WORK.sealed_header;
        let door = &REPLAY_WORK.unknown_7d0[0];
        let kept_cells: Vec<u32> = cells.iter().map(|c| c.get()).collect();
        let kept_locale = locale.get(8);
        let kept_header: Vec<u8> = (0..16).map(|i| header.get(i)).collect();
        let kept_sealed: Vec<u32> = sealed.iter().map(Dword::get).collect();
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
                    copy_header_settings(GamePtr::to(sealed)).unwrap();
                } else {
                    old_copy().unwrap();
                }
                let mut out: Vec<u32> = sealed.iter().map(Dword::get).collect();
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
