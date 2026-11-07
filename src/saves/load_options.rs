//! `saves::load_options` (0x4AAB32): reads the game's kept options, `var\ral.dat`.
//!
//! In:  nothing
//! Out: carry = not loaded; every register preserved
//! `g_optionsLoaded` (0x75557D) = 0. Up to 0x10000 bytes of the file into the scratch block at
//! 0x93E8B0; it is taken if it starts "R5CI" and has 0x50004 at +8. From +0x10 on it is copied out
//! piece by piece to the 69 places of `pieces` (0xB656 bytes: the screen modes, the sound's and
//! the network's settings, the four players, the record table, the players' names and much else).
//! After the last piece the file must have "####", else fatal error 0x80ACFF01 "ral.dat out of
//! date" (the pieces have been copied by then). Then `saves::rebuild_car_lists`, and
//! `g_optionsLoaded` = 1. A file shorter than that is not noticed as such: what the scratch block
//! held decides. One caller, `sub_411239`. `sub_4AAE57` writes the file, the same pieces the other
//! way.

use crate::audio::owner::audio;
use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_file::read_file_to_store;
use crate::gfx::owner::gfx;
use crate::menu::owner::menus;
use crate::net::owner::net;
use crate::pointers::{GamePtr, Store};
use crate::race::owner::race;
use crate::saves::owner::saves;
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use crate::service::owner::service;
use crate::settings::TEXT_ROOM;
use crate::system::owner::system;

// ---- game data ----

/// IDA's address of the options file's name.
pub const PATH_START: usize = 0x5295D8;

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::saves::owner::SAVES.files.options_path,
            PATH_START,
            PATH_START + 12,
        );
        assert_eq!(
            crate::moved::place(PATH_START),
            saves().files.options_path.addr() as usize
        );
    }
}

/// The 69 pieces' lengths together.
pub const PIECES_LEN: usize = 0xB656;

/// One piece of the options file: bytes of a field of the game's state, or (a slip kept) a run
/// that goes on past its field.
pub enum Piece<'a> {
    /// `len` bytes of `store` from `offset`.
    Field {
        store: &'a dyn Store,
        offset: usize,
        len: usize,
    },
    /// `len` bytes from a place that does not stop at its field's end.
    Past { place: GamePtr, len: usize },
}

impl<'a> Piece<'a> {
    /// All of a field: the piece's length is the field's size.
    pub fn whole(store: &'a dyn Store) -> Self {
        Self::Field {
            store,
            offset: 0,
            len: store.size(),
        }
    }

    /// `len` bytes of a field from `offset`.
    pub fn part(store: &'a dyn Store, offset: usize, len: usize) -> Self {
        Self::Field { store, offset, len }
    }

    /// The original's run goes on past the field (`option_work`'s switches and levels): a
    /// place, not a field.
    pub fn past(place: GamePtr, len: usize) -> Self {
        Self::Past { place, len }
    }

    pub fn len(&self) -> usize {
        match *self {
            Self::Field { len, .. } | Self::Past { len, .. } => len,
        }
    }

    pub fn read(&self) -> Result<Vec<u8>, BadInput> {
        match *self {
            Self::Field { store, offset, len } => {
                let mut bytes = vec![0; len];
                store.load(offset, &mut bytes)?;
                Ok(bytes)
            }
            Self::Past { place, len } => place.read_bytes(len, "a saved option"),
        }
    }

    pub fn write(&self, bytes: &[u8]) -> Result<(), BadInput> {
        match *self {
            Self::Field { store, offset, .. } => store.store(offset, bytes),
            Self::Past { place, .. } => place.write_bytes(bytes, "an option"),
        }
    }
}

/// The file's pieces after its head, in order: (where each is, its length).
#[rustfmt::skip]
pub fn pieces() -> [Piece<'static>; 69] {
    let race = race();
    let net = net();
    let audio = audio();
    let system = system();
let saves = saves();
let display = &gfx().display;
let menus = menus();

    let names = |n: usize| Piece::part(&menus.play.names.player_names, 0x20 * n, 0x20);
    [
        Piece::whole(&system.start.run_count.runs),                            // how many times the game was run
        Piece::whole(&display.mode.menu_mode),                     // the menus' screen mode
        Piece::whole(&display.mode.race_mode),                     // the race's screen mode
        Piece::whole(&display.choices.mipmapping),                 // mipmapping, off or on
        Piece::whole(&audio.state.menu_sound_mode),                     // the menus' sound mode
        Piece::whole(&audio.state.race_sound_mode),                     // the race's sound mode
        Piece::whole(&audio.state.master_volume),                       // the master volume
        Piece::whole(&crate::sound::owner::sound().menu.options.kept_f0),                     // a sound option, not worked out
        Piece::whole(&crate::sound::owner::sound().menu.options.name_switch),                 // the driver names switch
        Piece::whole(&crate::sound::owner::sound().menu.options.tune_turn),                   // which tune comes next
        Piece::whole(&crate::sound::owner::sound().menu.options.music_volume),                // the music volume
        Piece::whole(&crate::sound::owner::sound().menu.options.race_music_volume),           // the race music's volume
        Piece::whole(&net.vars.machine_name),                         // the machine's name
        Piece::whole(&net.vars.most_players),                          // the most players a hosted session takes
        Piece::whole(&net.vars.host_address),                         // the host's address (TCP/IP)
        Piece::whole(&net.vars.port),                                  // the TCP/IP port
        Piece::whole(&net.vars.saved_connection),                     // the saved connection's GUID
        Piece::whole(&saves.save_games.kept_a),                         // a saved-games word, not worked out
        Piece::whole(&saves.save_games.kept_b),                         // a saved-games word, not worked out
        Piece::whole(&service().set_ups.kept),                              // a car set-ups word, not worked out
        Piece::whole(&race.shared.tyre_change.record_name),                  // the tyre change record holder's name
        Piece::whole(&race.shared.tyre_change.record),                        // the tyre change record time
        Piece::whole(&saves.files.options_kept[0]),                                   // four dwords nothing else reads
        Piece::whole(&saves.files.options_kept[1]),
        Piece::whole(&saves.files.options_kept[2]),
        Piece::whole(&saves.files.options_kept[3]),
        Piece::whole(&menus.play.players.players[0]),                 // the four players' records
        Piece::whole(&menus.play.players.players[1]),
        Piece::whole(&menus.play.players.players[2]),
        Piece::whole(&menus.play.players.players[3]),
        Piece::whole(&menus.screens.option_work.master_volume_shown),           // the master volume shown
        Piece::whole(&menus.screens.option_work.music_level),                   // the music level
        Piece::whole(&menus.screens.option_work.other_volume),                  // read by the music "up" step
        Piece::whole(&menus.screens.option_work.menu_sounds_c),                 // the menu sounds' third volume
        Piece::whole(&menus.screens.option_work.menu_sounds_a),                 // the menu sounds' first volume
        Piece::whole(&menus.screens.option_work.speech_level),                  // the speech level
        Piece::whole(&menus.screens.option_work.sounds_a),                      // the sounds' first level
        Piece::whole(&menus.screens.option_work.sounds_b),                      // the sounds' second level
        Piece::past(GamePtr::to(&menus.screens.option_work.switch_0).unbounded(), 0x2C),      // the options pages' work copies, switch_0 .. setting_4
        Piece::past(GamePtr::to(&menus.screens.option_work.audio_quality).unbounded(), 0x18),    // the audio quality, the 3D switches and the channels
        Piece::whole(&menus.tables.pages.services),                       // the services counted
        Piece::whole(&menus.screens.flow.kept),                          // kept by the event races
        Piece::whole(&race.shared.state.settings.a8_game),               // the race settings' first word
        Piece::part(&menus.play.arcade.levels_won, 0, 0x6),                    // the arcade levels won, a byte each
        Piece::whole(&menus.cars.numbers.first_numbers),                // the first list of car numbers
        Piece::part(&menus.cars.numbers.second_numbers, 0, 0x50),         // the second list of car numbers, 20 entries
        Piece::whole(&system.records.stage_tables.championship),              // the championship's records
        Piece::whole(&system.records.stage_tables.events[5].records),          // the events' record tables, in the file's order
        Piece::whole(&system.records.stage_tables.events[4].records),
        Piece::whole(&system.records.stage_tables.events[3].records),
        Piece::whole(&system.records.stage_tables.events[2].records),
        Piece::whole(&system.records.stage_tables.events[1].records),
        Piece::whole(&system.records.stage_tables.events[0].records),
        Piece::whole(&system.records.stage_tables.events[6].records),
        Piece::whole(&system.records.stage_tables.events[7].records),
        Piece::whole(&system.records.stage_tables.events[8].records),
        Piece::whole(&system.records.stage_tables.events[9].records),
        Piece::whole(&system.records.stage_tables.events[10].records),
        Piece::whole(&system.records.stage_tables.events[11].records),
        Piece::whole(&saves.rally_times.rallies[0]),                  // the six rallies' best times
        Piece::whole(&saves.rally_times.rallies[1]),
        Piece::whole(&saves.rally_times.rallies[2]),
        Piece::whole(&saves.rally_times.rallies[3]),
        Piece::whole(&saves.rally_times.rallies[4]),
        Piece::whole(&saves.rally_times.rallies[5]),
        names(0),                                               // the four players' names
        names(1),
        names(2),
        names(3),
    ]
}

// ---- Rust version ----

/// Where the function works: the game's own places, or a test's.
pub struct Places<'a> {
    pub loaded: &'a crate::session::Dword,
    pub pieces: &'a [Piece<'a>],
    pub rebuild_car_lists: &'a mut dyn FnMut() -> Result<(), BadInput>,
}

/// `saves::load_options` (see the header).
pub fn load_options(path: &dyn Store, places: &mut Places) -> Result<bool, Refused> {
    places.loaded.set(0);
    let file = &system().work.files.buffer;
    let read = read_file_to_store(
        &crate::files::cstring(path, "the options file's path")?,
        file,
        TEXT_ROOM + 1,
        0,
    )?;
    use rc2k_core::saves::kept;
    let block = file.read_at(0, TEXT_ROOM as usize + 1, "the options file")?;
    if !read.ok || !kept::head_ok(&block, kept::OPTIONS) {
        return Ok(false);
    }
    let sizes: Vec<usize> = places.pieces.iter().map(Piece::len).collect();
    let (pieces, ended) = kept::pieces(&block, &sizes)?;
    for (place, piece) in places.pieces.iter().zip(pieces) {
        place.write(piece)?;
    }
    if !ended {
        return Err(FatalError {
            code: 0x80AC_FF01,
            message: "ral.dat out of date",
        }
        .into());
    }
    (places.rebuild_car_lists)()?;
    places.loaded.set(1);
    Ok(true)
}

pub fn load_game_options() -> Result<bool, Refused> {
    let pieces = pieces();
    load_options(
        &saves().files.options_path,
        &mut Places {
            loaded: &menus().screens.loop_flags.options_loaded,
            pieces: &pieces,
            rebuild_car_lists: &mut || rebuild_car_lists(crate::menu::owner::menus()),
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::pointers::GamePtr;
    use crate::saves::owner::saves;
    #[test]
    fn kept_starts_as_the_games() {
        let image = crate::moved::image_bytes(0x4F66DD, 20);
        let mine = unsafe {
            std::slice::from_raw_parts(saves().files.options_kept.as_ptr().cast::<u8>(), 20)
        };
        assert_eq!(mine, &image[..]);
    }

    /// The IDA addresses and lengths of the pieces, as the original has them.
    #[rustfmt::skip]
    const OPTIONS: [(u32, usize); 69] = [
        (0x518BB0, 0x4),
        (0x4F4944, 0x4),
        (0x4F4948, 0x4),
        (0x4F50D0, 0x4),
        (0x4F82C4, 0x4),
        (0x4F82C8, 0x4),
        (0x4F82D8, 0x4),
        (0x5180F0, 0x4),
        (0x5180F4, 0x4),
        (0x5180F8, 0x4),
        (0x5180FC, 0x4),
        (0x518100, 0x4),
        (0x52AB90, 0x40),
        (0x52ABD0, 0x4),
        (0x52ABE4, 0x40),
        (0x52AC24, 0x4),
        (0x52AC94, 0x10),
        (0x51EBB0, 0x4),
        (0x527834, 0x4),
        (0x528500, 0x4),
        (0x5BEC08, 0x20),
        (0x5BEC28, 0x4),
        (0x4F66DD, 0x4),
        (0x4F66E1, 0x4),
        (0x4F66E5, 0x4),
        (0x4F66E9, 0x4),
        (0x755A4C, 0x28C),
        (0x755CD8, 0x28C),
        (0x755F64, 0x28C),
        (0x7561F0, 0x28C),
        (0x755684, 0x4),
        (0x755688, 0x4),
        (0x75568C, 0x4),
        (0x755690, 0x4),
        (0x755694, 0x4),
        (0x755698, 0x4),
        (0x75569C, 0x4),
        (0x7556A0, 0x4),
        (0x755640, 0x2C),
        (0x75566C, 0x18),
        (0x755254, 0x4),
        (0x7554C8, 0x4),
        (0x75ACCC, 0x4),
        (0x758800, 0x6),
        (0x755258, 0xA0),
        (0x7552F8, 0x50),
        (0x735576, 0x2A48),
        (0x739642, 0x47C),
        (0x7391C2, 0x47C),
        (0x738D42, 0x47C),
        (0x7388C2, 0x47C),
        (0x738442, 0x47C),
        (0x737FC2, 0x47C),
        (0x739AC2, 0x47C),
        (0x739F42, 0x47C),
        (0x73A3C2, 0x47C),
        (0x73A842, 0x47C),
        (0x73ACC2, 0x47C),
        (0x73B142, 0x47C),
        (0x742CB0, 0xC30),
        (0x7438E0, 0xC30),
        (0x744510, 0xC30),
        (0x745140, 0xC30),
        (0x745D70, 0xC30),
        (0x7469A0, 0xC30),
        (0x755784, 0x20),
        (0x7557A4, 0x20),
        (0x7557C4, 0x20),
        (0x7557E4, 0x20),
    ];

    /// Each piece is where its IDA address has gone, with its length, whole in one store.
    #[test]
    fn pieces_are_the_options_places() {
        let pieces = super::pieces();
        assert_eq!(
            OPTIONS.iter().map(|&(_, len)| len).sum::<usize>(),
            super::PIECES_LEN
        );
        for (n, (&(at, len), piece)) in OPTIONS.iter().zip(pieces.iter()).enumerate() {
            let at = at as usize;
            assert_eq!(piece.len(), len, "piece {n} ({at:#x}) has another length");
            let (place, size) = match *piece {
                super::Piece::Field { store, offset, len } => {
                    assert!(
                        offset + len <= store.size(),
                        "piece {n} ({at:#x}) leaves its field"
                    );
                    (GamePtr::of_store(store).offset(offset as u32), len)
                }
                super::Piece::Past { place, len } => (place, len),
            };
            assert_eq!(size, len);
            assert_eq!(
                place.addr() as usize,
                crate::moved::place(at),
                "piece {n} ({at:#x}) is elsewhere"
            );
            let (start, end) = crate::pointers::region_of(place.addr()).expect("a store");
            assert!(
                place.addr() + len as u32 <= end,
                "piece {n} ({at:#x}) leaves its store"
            );
            assert!(start <= place.addr());
        }
    }
}
