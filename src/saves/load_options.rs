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

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_file::read_file;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::pointers::GamePtr;
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use crate::settings::TEXT_ROOM;

/// `unk_4F66DD`, `unk_4F66E1`, `unk_4F66E5` (-1 at start), `unk_4F66E9`: four dwords the
/// options file keeps (here and in `saves::save_options`, by address); nothing else of the
/// game reads them (a table of the developers' variables names the first). The fifth
/// (0x4F66ED) is a gap. Split 2026-10-25 from sound's wheel surfaces after them.
pub static KEPT: [crate::session::Dword; 5] = [
    crate::session::Dword::new(0),
    crate::session::Dword::new(0),
    crate::session::Dword::new(0xFFFF_FFFF),
    crate::session::Dword::new(0),
    crate::session::Dword::new(0),
];

// ---- game data ----

/// IDA's address of the options file's name.
pub const PATH_START: usize = 0x5295D8;

/// `aVarRalDat`: the options file's name.
pub static PATH: crate::session::Bytes<12> = crate::session::Bytes::new(*b"var\\ral.dat\0");

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PATH, PATH_START, PATH_START + 12);
        assert_eq!(crate::moved::place(PATH_START), PATH.addr() as usize);
    }
}

/// The 69 pieces' lengths together.
pub const PIECES_LEN: usize = 0xB656;

/// The file's pieces after its head, in order: (where each is, its length).
#[rustfmt::skip]
pub fn pieces() -> [(GamePtr, usize); 69] {
    use crate::audio::store::AUDIO;
    use crate::gfx::choices_store::SCREEN_CHOICES;
    use crate::gfx::screen_state::SCREEN_MODE;
    use crate::menu::arcade_store::ARCADE_GAME;
    use crate::menu::flow_store::SCREEN_FLOW;
    use crate::menu::names_store::NAME_BUFFERS;
    use crate::menu::option_work_store::OPTION_WORK;
    use crate::menu::pages_store::MENU_PAGES;
    use crate::menu::players_store::PLAYER_TABLE;
    use crate::net::store::NET;
    use crate::race::state_store::RACE_STATE;
    use crate::race::tyre_store::TYRE_CHANGE;
    use crate::records::stages_store::STAGE_TABLES;
    use crate::service::read_setup_slot::SET_UPS;

    use crate::system::load_game_data::RUN_COUNT;
    let names = |n: u32| GamePtr::to(&NAME_BUFFERS.player_names).offset(0x20 * n).limited(0x20);
    [
        (GamePtr::to(&RUN_COUNT.runs), 0x4),                            // how many times the game was run
        (GamePtr::to(&SCREEN_MODE.menu_mode), 0x4),                     // the menus' screen mode
        (GamePtr::to(&SCREEN_MODE.race_mode), 0x4),                     // the race's screen mode
        (GamePtr::to(&SCREEN_CHOICES.mipmapping), 0x4),                 // mipmapping, off or on
        (GamePtr::to(&AUDIO.menu_sound_mode), 0x4),                     // the menus' sound mode
        (GamePtr::to(&AUDIO.race_sound_mode), 0x4),                     // the race's sound mode
        (GamePtr::to(&AUDIO.master_volume), 0x4),                       // the master volume
        (GamePtr::to(&crate::sound::owner::sound().menu.options.kept_f0), 0x4),                     // a sound option, not worked out
        (GamePtr::to(&crate::sound::owner::sound().menu.options.name_switch), 0x4),                 // the driver names switch
        (GamePtr::to(&crate::sound::owner::sound().menu.options.tune_turn), 0x4),                   // which tune comes next
        (GamePtr::to(&crate::sound::owner::sound().menu.options.music_volume), 0x4),                // the music volume
        (GamePtr::to(&crate::sound::owner::sound().menu.options.race_music_volume), 0x4),           // the race music's volume
        (GamePtr::to(&NET.machine_name), 0x40),                         // the machine's name
        (GamePtr::to(&NET.most_players), 0x4),                          // the most players a hosted session takes
        (GamePtr::to(&NET.host_address), 0x40),                         // the host's address (TCP/IP)
        (GamePtr::to(&NET.port), 0x4),                                  // the TCP/IP port
        (GamePtr::to(&NET.saved_connection), 0x10),                     // the saved connection's GUID
        (GamePtr::to(&crate::saves::owner::saves().save_games.kept_a), 0x4),                         // a saved-games word, not worked out
        (GamePtr::to(&crate::saves::owner::saves().save_games.kept_b), 0x4),                         // a saved-games word, not worked out
        (GamePtr::to(&SET_UPS.kept), 0x4),                              // a car set-ups word, not worked out
        (GamePtr::to(&TYRE_CHANGE.record_name), 0x20),                  // the tyre change record holder's name
        (GamePtr::to(&TYRE_CHANGE.record), 0x4),                        // the tyre change record time
        (GamePtr::to(&KEPT[0]), 0x4),                                   // four dwords nothing else reads
        (GamePtr::to(&KEPT[1]), 0x4),
        (GamePtr::to(&KEPT[2]), 0x4),
        (GamePtr::to(&KEPT[3]), 0x4),
        (GamePtr::to(&PLAYER_TABLE.players[0]), 0x28C),                 // the four players' records
        (GamePtr::to(&PLAYER_TABLE.players[1]), 0x28C),
        (GamePtr::to(&PLAYER_TABLE.players[2]), 0x28C),
        (GamePtr::to(&PLAYER_TABLE.players[3]), 0x28C),
        (GamePtr::to(&OPTION_WORK.master_volume_shown), 0x4),           // the master volume shown
        (GamePtr::to(&OPTION_WORK.music_level), 0x4),                   // the music level
        (GamePtr::to(&OPTION_WORK.other_volume), 0x4),                  // read by the music "up" step
        (GamePtr::to(&OPTION_WORK.menu_sounds_c), 0x4),                 // the menu sounds' third volume
        (GamePtr::to(&OPTION_WORK.menu_sounds_a), 0x4),                 // the menu sounds' first volume
        (GamePtr::to(&OPTION_WORK.speech_level), 0x4),                  // the speech level
        (GamePtr::to(&OPTION_WORK.sounds_a), 0x4),                      // the sounds' first level
        (GamePtr::to(&OPTION_WORK.sounds_b), 0x4),                      // the sounds' second level
        (GamePtr::to(&OPTION_WORK.switch_0).unbounded(), 0x2C),         // the options pages' work copies, switch_0 .. setting_4
        (GamePtr::to(&OPTION_WORK.audio_quality).unbounded(), 0x18),    // the audio quality, the 3D switches and the channels
        (GamePtr::to(&MENU_PAGES.services), 0x4),                       // the services counted
        (GamePtr::to(&SCREEN_FLOW.kept), 0x4),                          // kept by the event races
        (GamePtr::to(&RACE_STATE.settings.a8_game), 0x4),               // the race settings' first word
        (GamePtr::to(&ARCADE_GAME.levels_won), 0x6),                    // the arcade levels won, a byte each
        (GamePtr::to(&crate::menu::owner::menus().cars.numbers.first_numbers), 0xA0),                // the first list of car numbers
        (GamePtr::to(&crate::menu::owner::menus().cars.numbers.second_numbers[..20]), 0x50),         // the second list of car numbers, 20 entries
        (GamePtr::to(&STAGE_TABLES.championship), 0x2A48),              // the championship's records
        (GamePtr::to(&STAGE_TABLES.events[5].records), 0x47C),          // the events' record tables, in the file's order
        (GamePtr::to(&STAGE_TABLES.events[4].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[3].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[2].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[1].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[0].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[6].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[7].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[8].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[9].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[10].records), 0x47C),
        (GamePtr::to(&STAGE_TABLES.events[11].records), 0x47C),
        (GamePtr::to(&crate::saves::owner::saves().rally_times.rallies[0]), 0xC30),                  // the six rallies' best times
        (GamePtr::to(&crate::saves::owner::saves().rally_times.rallies[1]), 0xC30),
        (GamePtr::to(&crate::saves::owner::saves().rally_times.rallies[2]), 0xC30),
        (GamePtr::to(&crate::saves::owner::saves().rally_times.rallies[3]), 0xC30),
        (GamePtr::to(&crate::saves::owner::saves().rally_times.rallies[4]), 0xC30),
        (GamePtr::to(&crate::saves::owner::saves().rally_times.rallies[5]), 0xC30),
        (names(0), 0x20),                                               // the four players' names
        (names(1), 0x20),
        (names(2), 0x20),
        (names(3), 0x20),
    ]
}

// ---- Rust version ----

/// Where the function works: the game's own places, or a test's.
pub struct Places<'a> {
    pub loaded: &'a crate::session::Dword,
    pub pieces: &'a [(GamePtr, usize)],
    pub rebuild_car_lists: &'a mut dyn FnMut() -> Result<(), BadInput>,
}

/// `saves::load_options` (see the header).
pub fn load_options(path: crate::pointers::GamePtr, places: &mut Places) -> Result<bool, Refused> {
    places.loaded.set(0);
    let file = &crate::files::scratch::SCRATCH.buffer;
    let read = read_file(path, crate::pointers::GamePtr::to(file), TEXT_ROOM + 1, 0)?;
    use rc2k_core::saves::kept;
    let block = file.read_at(0, TEXT_ROOM as usize + 1, "the options file")?;
    if !read.ok || !kept::head_ok(&block, kept::OPTIONS) {
        return Ok(false);
    }
    let sizes: Vec<usize> = places.pieces.iter().map(|&(_, len)| len).collect();
    let (pieces, ended) = kept::pieces(&block, &sizes)?;
    for (&(place, _), piece) in places.pieces.iter().zip(pieces) {
        place.write_bytes(piece, "an option")?;
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
        crate::pointers::GamePtr::to(&PATH),
        &mut Places {
            loaded: &LOOP_FLAGS.options_loaded,
            pieces: &pieces,
            rebuild_car_lists: &mut || rebuild_car_lists(),
        },
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn kept_starts_as_the_games() {
        let image = crate::moved::image_bytes(0x4F66DD, 20);
        let mine = unsafe { std::slice::from_raw_parts(super::KEPT.as_ptr().cast::<u8>(), 20) };
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
        for (n, (&(at, len), &(place, size))) in OPTIONS.iter().zip(pieces.iter()).enumerate() {
            let at = at as usize;
            assert_eq!(size, len, "piece {n} ({at:#x}) has another length");
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
