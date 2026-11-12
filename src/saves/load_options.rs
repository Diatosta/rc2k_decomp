//! `saves::load_options` (0x4AAB32): reads the game's kept options, `var\ral.dat`.
//!
//! `g_optionsLoaded` (0x75557D) = 0. Up to 0x10000 bytes of the file into the scratch block at
//! 0x93E8B0; it is taken if it starts "R5CI" and has 0x50004 at +8. From +0x10 on it is copied out
//! piece by piece to the 69 places of `pieces` (0xB656 bytes: the screen modes, the sound's and
//! the network's settings, the four players, the record table, the players' names and much else).
//! After the last piece the file must have "####", else fatal error 0x80ACFF01 "ral.dat out of
//! date" (the pieces have been copied by then). Then `saves::rebuild_car_lists`, and
//! `g_optionsLoaded` = 1. A file shorter than that is not noticed as such: what the scratch block
//! held decides. One caller, `system::load_game_data`. `saves::save_options` writes the file, the
//! same pieces the other way.

use crate::cells::{Cells, Store, load_cells, store_cells};
use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_file::read_file_to_store;
use crate::menu::option_work_store::OptionWork;
use crate::owners::Owners;
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use crate::settings::TEXT_ROOM;
use std::mem::offset_of;

// ---- game data ----

#[cfg(test)]
mod path_tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::saves::owner::SAVES.get().files.options_path,
        );
    }
}

/// The 69 pieces' lengths together.
pub const PIECES_LEN: usize = 0xB656;

/// A field of cells that only the game's thread reaches, read and written whole by bytes.
pub trait Held {
    fn size(&self) -> usize;
    fn load(&self, to: &mut [u8]) -> Result<(), BadInput>;
    fn store(&self, from: &[u8]) -> Result<(), BadInput>;
}

impl<T: Cells> Held for T {
    fn size(&self) -> usize {
        size_of::<T>()
    }

    fn load(&self, to: &mut [u8]) -> Result<(), BadInput> {
        load_cells(self, 0, to)
    }

    fn store(&self, from: &[u8]) -> Result<(), BadInput> {
        store_cells(self, 0, from)
    }
}

/// One piece of the options file: bytes of a field of the game's state, or (a slip kept) a run
/// that goes on past its field.
pub enum Piece<'a> {
    /// `len` bytes of `store` from `offset`.
    Field {
        store: &'a dyn Store,
        offset: usize,
        len: usize,
    },
    /// All of a field of the game's thread (a store of plain cells, which no other thread
    /// may see: not a `Store`).
    Held(&'a dyn Held),
    /// `len` bytes of a field of the game's thread from `offset` (a run over several of its
    /// fields).
    HeldPart {
        field: &'a dyn Held,
        offset: usize,
        len: usize,
    },
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

    /// All of a field of plain cells.
    pub fn held(field: &'a dyn Held) -> Self {
        Self::Held(field)
    }

    /// `len` bytes of a field from `offset`.
    pub fn part(store: &'a dyn Store, offset: usize, len: usize) -> Self {
        Self::Field { store, offset, len }
    }

    /// `len` bytes of a field of plain cells from `offset`: a run over several of its fields
    /// (`option_work`'s switches and levels).
    pub fn held_part(field: &'a dyn Held, offset: usize, len: usize) -> Self {
        Self::HeldPart { field, offset, len }
    }

    pub fn len(&self) -> usize {
        match *self {
            Self::Field { len, .. } | Self::HeldPart { len, .. } => len,
            Self::Held(field) => field.size(),
        }
    }

    pub fn read(&self) -> Result<Vec<u8>, BadInput> {
        match *self {
            Self::Field { store, offset, len } => {
                let mut bytes = vec![0; len];
                store.load(offset, &mut bytes)?;
                Ok(bytes)
            }
            Self::Held(field) => {
                let mut bytes = vec![0; field.size()];
                field.load(&mut bytes)?;
                Ok(bytes)
            }
            Self::HeldPart { field, offset, len } => {
                let mut bytes = vec![0; field.size()];
                field.load(&mut bytes)?;
                bytes
                    .get(offset..offset + len)
                    .map(<[u8]>::to_vec)
                    .ok_or(BadInput("a saved option past its field"))
            }
        }
    }

    pub fn write(&self, bytes: &[u8]) -> Result<(), BadInput> {
        match *self {
            Self::Field { store, offset, .. } => store.store(offset, bytes),
            Self::Held(field) => field.store(bytes),
            Self::HeldPart { field, offset, len } => {
                let mut all = vec![0; field.size()];
                field.load(&mut all)?;
                all.get_mut(offset..offset + len)
                    .ok_or(BadInput("an option past its field"))?
                    .copy_from_slice(
                        bytes
                            .get(..len)
                            .ok_or(BadInput("too few bytes for an option"))?,
                    );
                field.store(&all)
            }
        }
    }
}

/// The file's pieces after its head, in order: (where each is, its length).
#[rustfmt::skip]
pub fn pieces_of(o: Owners<'static>) -> [Piece<'static>; 69] {
    let race = o.race;
    let net = o.net;
    let audio = o.audio;
    let system = o.system;
    let saves = o.saves;
    let display = &o.gfx.display;
    let menus = o.menus;

    let names = |n: usize| Piece::part(&menus.play.names.player_names, 0x20 * n, 0x20);
    [
        Piece::whole(&system.start().run_count.runs),                            // how many times the game was run
        Piece::whole(&display.mode.menu_mode),                     // the menus' screen mode
        Piece::whole(&display.mode.race_mode),                     // the race's screen mode
        Piece::whole(&display.choices.mipmapping),                 // mipmapping, off or on
        Piece::whole(&audio.state.menu_sound_mode),                     // the menus' sound mode
        Piece::whole(&audio.state.race_sound_mode),                     // the race's sound mode
        Piece::whole(&audio.state.master_volume),                       // the master volume
        Piece::whole(&o.sound.menu.options.unread_option_f0),                     // a sound option kept in the file and read by no other code
        Piece::whole(&o.sound.menu.options.race_music_on),                 // the race music switch
        Piece::whole(&o.sound.menu.options.tune_turn),                   // which tune comes next
        Piece::whole(&o.sound.menu.options.music_volume),                // the music volume
        Piece::whole(&o.sound.menu.options.race_music_volume),           // the race music's volume
        Piece::whole(&net.setup().machine_name),                         // the machine's name
        Piece::whole(&net.setup().most_players),                          // the most players a hosted session takes
        Piece::whole(&net.setup().host_address),                         // the host's address (TCP/IP)
        Piece::whole(&net.setup().port),                                  // the TCP/IP port
        Piece::whole(&net.address().saved_connection),                     // the saved connection's GUID
        Piece::held(&saves.save_games.kept_a),                         // a saved-games word kept in the file and read by no other code
        Piece::held(&saves.save_games.kept_b),                         // a saved-games word kept in the file and read by no other code
        Piece::held(&o.service.set_ups.kept),                              // a car set-ups word kept in the file and read by no other code
        Piece::whole(&race.shared.tyre_change.record_name),                  // the tyre change record holder's name
        Piece::whole(&race.shared.tyre_change.record),                        // the tyre change record time
        Piece::held(&saves.files.options_kept[0]),                                   // four dwords nothing else reads
        Piece::held(&saves.files.options_kept[1]),
        Piece::held(&saves.files.options_kept[2]),
        Piece::held(&saves.files.options_kept[3]),
        Piece::whole(&menus.play.players.players[0]),                 // the four players' records
        Piece::whole(&menus.play.players.players[1]),
        Piece::whole(&menus.play.players.players[2]),
        Piece::whole(&menus.play.players.players[3]),
        Piece::whole(&audio.levels.master_volume_shown),           // the master volume shown
        Piece::whole(&menus.screens.option_work.music_level),                   // the music level
        Piece::whole(&menus.screens.option_work.other_volume),                  // read by the music "up" step
        Piece::whole(&menus.screens.option_work.menu_sounds_c),                 // the menu sounds' third volume
        Piece::whole(&audio.levels.menu_sounds_a),                 // the menu sounds' first volume
        Piece::whole(&menus.screens.option_work.speech_level),                  // the speech level
        Piece::whole(&audio.levels.sounds_a),                      // the sounds' first level
        Piece::whole(&audio.levels.sounds_b),                      // the sounds' second level
        Piece::held_part(&menus.screens.option_work, offset_of!(OptionWork, switch_0), 0x2C),      // the options pages' work copies, switch_0 .. headlights
        Piece::held_part(&menus.screens.option_work, offset_of!(OptionWork, audio_quality), 0x18),    // the audio quality, the 3D switches and the channels
        Piece::whole(&menus.tables.pages.services),                       // the services counted
        Piece::whole(&menus.screens.flow.kept),                          // kept by the event races
        Piece::whole(&race.shared.state.settings.a8_game),               // the race settings' first word
        Piece::part(&menus.play.arcade.levels_won, 0, 0x6),                    // the arcade levels won, a byte each
        Piece::whole(&menus.cars.numbers.first_numbers),                // the first list of car numbers
        Piece::part(&menus.cars.numbers.second_numbers, 0, 0x50),         // the second list of car numbers, 20 entries
        Piece::whole(&system.records().stage_tables.championship),              // the championship's records
        Piece::whole(&system.records().stage_tables.events[5].records),          // the events' record tables, in the file's order
        Piece::whole(&system.records().stage_tables.events[4].records),
        Piece::whole(&system.records().stage_tables.events[3].records),
        Piece::whole(&system.records().stage_tables.events[2].records),
        Piece::whole(&system.records().stage_tables.events[1].records),
        Piece::whole(&system.records().stage_tables.events[0].records),
        Piece::whole(&system.records().stage_tables.events[6].records),
        Piece::whole(&system.records().stage_tables.events[7].records),
        Piece::whole(&system.records().stage_tables.events[8].records),
        Piece::whole(&system.records().stage_tables.events[9].records),
        Piece::whole(&system.records().stage_tables.events[10].records),
        Piece::whole(&system.records().stage_tables.events[11].records),
        Piece::held(&saves.rally_times.rallies[0]),                  // the six rallies' best times
        Piece::held(&saves.rally_times.rallies[1]),
        Piece::held(&saves.rally_times.rallies[2]),
        Piece::held(&saves.rally_times.rallies[3]),
        Piece::held(&saves.rally_times.rallies[4]),
        Piece::held(&saves.rally_times.rallies[5]),
        names(0),                                               // the four players' names
        names(1),
        names(2),
        names(3),
    ]
}

/// The pieces on the game's own places.
#[cfg(test)]
pub fn pieces() -> [Piece<'static>; 69] {
    pieces_of(Owners::the())
}

/// Where the function works: the game's own places, or a test's.
pub struct Places<'a> {
    /// The buffer the file is read into.
    pub file: &'a crate::session::PlainBytes<65536>,
    pub loaded: &'a dyn crate::session::FlagLike,
    pub pieces: &'a [Piece<'a>],
    pub rebuild_car_lists: &'a mut dyn FnMut() -> Result<(), BadInput>,
}

/// `saves::load_options` (see the header).
pub fn load_options(path: &dyn Store, places: &mut Places) -> Result<bool, Refused> {
    places.loaded.set(false);
    let file = places.file;
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
    places.loaded.set(true);
    Ok(true)
}

pub fn load_game_options(o: Owners<'static>) -> Result<bool, Refused> {
    let menus = o.menus;
    let pieces = pieces_of(o);
    load_options(
        &o.saves.files.options_path,
        &mut Places {
            file: &o.system.work().files.buffer,
            loaded: &menus.screens.loop_flags.options_loaded,
            pieces: &pieces,
            rebuild_car_lists: &mut || rebuild_car_lists(menus),
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::saves::owner::saves;
    #[test]
    fn kept_starts_as_the_games() {
        let image =
            crate::test_support::first_values::bytes("saves.owner.SAVES.files.options_kept/20");
        let mine = unsafe {
            std::slice::from_raw_parts(saves().files.options_kept.as_ptr().cast::<u8>(), 20)
        };
        assert_eq!(mine, image);
    }

    /// The lengths of the pieces, in the original's order.
    #[rustfmt::skip]
    const OPTIONS: [usize; 69] = [0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x40, 0x4, 0x40, 0x4, 0x10, 0x4, 0x4, 0x4, 0x20, 0x4, 0x4, 0x4, 0x4, 0x4, 0x28C, 0x28C, 0x28C, 0x28C, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x2C, 0x18, 0x4, 0x4, 0x4, 0x6, 0xA0, 0x50, 0x2A48, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0xC30, 0xC30, 0xC30, 0xC30, 0xC30, 0xC30, 0x20, 0x20, 0x20, 0x20];

    /// Each piece has its length and lies whole in its field.
    #[test]
    fn pieces_are_the_options_places() {
        let pieces = super::pieces();
        assert_eq!(OPTIONS.iter().sum::<usize>(), super::PIECES_LEN);
        for (n, (&len, piece)) in OPTIONS.iter().zip(pieces.iter()).enumerate() {
            assert_eq!(piece.len(), len, "piece {n} has another length");
            match *piece {
                super::Piece::Field { store, offset, len } => {
                    assert!(offset + len <= store.size(), "piece {n} leaves its field");
                }
                super::Piece::Held(field) => assert_eq!(field.size(), len, "piece {n}"),
                super::Piece::HeldPart { field, offset, len } => {
                    assert!(offset + len <= field.size(), "piece {n} leaves its field");
                }
            }
        }
    }
}
