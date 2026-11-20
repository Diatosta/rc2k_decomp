//! The options as the game's state holds them: `capture` reads every owner's field into an
//! `rc2k_core::options::Options`, `apply` puts one back. This file is the one place that names
//! each owner field behind an option. The 69 pieces of `pieces_of` (0xB656 bytes: the screen
//! modes, the sound's and the network's settings, the four players, the record table, the
//! players' names and much else) are the old file's layout, kept as the frozen reference the
//! model is read through.

use crate::errors::{BadInput, Refused};
use crate::memory::cells::Store;
use crate::memory::session::{DwordLike, Run, Shared};
use crate::menu::player::Player;
use crate::owners::Owners;
use crate::records::stages_store::{RECORD_LEN, Record};
use crate::saves::best_times_store::TimeEntry;
use rc2k_core::options::Options;
use std::sync::atomic::Ordering::Relaxed;

/// The options as the game's stores hold them now.
pub fn capture(o: Owners<'static>) -> Result<Options, Refused> {
    let mut owned = Vec::new();
    for piece in pieces_of(o) {
        owned.push(piece.read()?);
    }
    let refs: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
    let mut options = Options::default();
    if !options.take_pieces(&refs) {
        return Err(BadInput("the options' pieces do not fit the model").into());
    }
    options.players.devices = crate::input::device_ids::kept(o.input);
    options.players.profiles = crate::input::profiles::kept(o.input);
    options.sound.film_volume = crate::platform::sound::FILM_VOLUME.load(Relaxed);
    options.sound.sound_in_background = crate::platform::sound::SOUND_IN_BACKGROUND.load(Relaxed);
    options.network.password.0 = crate::net::password::get();
    options.network.password.0.resize(0x40, 0);
    options.network.last_host.0 = crate::net::last_host::get();
    options.network.last_host.0.resize(0x40, 0);
    options.display.frame_limit = crate::platform::pacing::frame_limit();
    options.display.render_scale = crate::render::render_scale();
    options.display.renderer = crate::render::choice::wanted();
    options.display.draw_threads = crate::scene::workers::chosen_threads();
    options.display.quit_pictures = crate::drive::quit_question::pictures_on();
    // A race size beyond the game's table is kept as its size; the mode word has the table's
    // 800 x 600 in its place, which the old file reads.
    let race = o.gfx.display.mode.race_mode.get();
    options.display.race_mode = crate::gfx::monitor_sizes::kept_mode(race);
    options.display.race_size = crate::gfx::monitor_sizes::size_of_mode(race).map(|(w, h)| [w, h]);
    let custom = crate::gfx::monitor_sizes::custom();
    options.display.custom_width = custom.map(|c| c.0);
    options.display.custom_height = custom.map(|c| c.1);
    let window = crate::platform::window_mode::get();
    options.display.window_mode = (window != Default::default()).then(|| window.name().to_owned());
    let hud_box = crate::hud::box_ratio::get();
    options.display.hud_box = (hud_box != Default::default()).then(|| hud_box.name().to_owned());
    options.display.smooth_frames = crate::race::blend::smooth_on();
    options.display.split_layout = crate::race::split_layout::index();
    options.display.video = crate::race::render_row::kept();
    Ok(options)
}

/// Puts the options into the game's stores (nothing else: the car lists and the loaded flag
/// are the caller's).
pub fn apply(o: Owners<'static>, options: &Options) -> Result<(), Refused> {
    let pieces = options.to_pieces();
    for (place, bytes) in pieces_of(o).iter().zip(&pieces) {
        place.write(bytes)?;
    }
    crate::input::device_ids::keep(o.input, options.players.devices.clone());
    crate::input::profiles::keep(o.input, options.players.profiles.clone());
    crate::platform::sound::FILM_VOLUME.store(options.sound.film_volume, Relaxed);
    crate::platform::sound::SOUND_IN_BACKGROUND.store(options.sound.sound_in_background, Relaxed);
    crate::net::password::set(&options.network.password.0);
    crate::net::last_host::set(&options.network.last_host.0);
    crate::platform::pacing::set_frame_limit(options.display.frame_limit);
    crate::render::set_render_scale(options.display.render_scale);
    crate::render::choice::set_wanted(options.display.renderer.clone());
    crate::scene::workers::set_threads(options.display.draw_threads);
    crate::drive::quit_question::set_pictures_on(options.display.quit_pictures);
    let d = &options.display;
    crate::gfx::monitor_sizes::set_custom(d.custom_width.zip(d.custom_height));
    crate::platform::window_mode::set(
        d.window_mode
            .as_deref()
            .map(crate::platform::window_mode::WindowMode::of)
            .unwrap_or_default(),
    );
    crate::hud::box_ratio::set(
        d.hud_box
            .as_deref()
            .map(rc2k_core::hud::anchor::BoxRatio::of)
            .unwrap_or_default(),
    );
    // The size set by hand wins; else the size the file kept beyond the game's table.
    // A video's size wins over both.
    let wanted = crate::video::size()
        .or(crate::gfx::monitor_sizes::custom())
        .or(d.race_size.map(|[w, h]| (w, h)));
    // A phone with no size kept starts at its own screen's size (the game's list has only
    // sizes far below it).
    #[cfg(target_os = "ios")]
    let wanted = wanted.or_else(|| {
        (d.race_mode == Options::default().display.race_mode)
            .then(crate::platform::phone_default_size)
            .flatten()
    });
    if let Some((w, h)) = wanted {
        let race = o.gfx.display.mode.race_mode.get();
        if let Some(mode) =
            crate::gfx::monitor_sizes::race_mode_for(&o.gfx.display.modes, race, w, h)
        {
            o.gfx.display.mode.race_mode.set(mode);
        }
    }
    // The size the file kept beyond the game's table was the window's: the list offers it when
    // nothing else does.
    crate::gfx::monitor_sizes::set_window(d.race_size.map(|[w, h]| (w, h)));
    // A video's frames between the ticks need the blend, whatever the options say.
    crate::race::blend::set_smooth(options.display.smooth_frames || crate::video::active());
    crate::race::split_layout::set_index(options.display.split_layout);
    crate::race::render_row::set_kept(options.display.video.clone());
    Ok(())
}

/// One entry of the options file: named fields of the game's state, in the file's order.
pub(crate) enum Field<'a> {
    /// One dword.
    Dword(&'a dyn DwordLike),
    /// Dwords one after the other, each a field of its own.
    Dwords(Vec<&'a dyn DwordLike>),
    /// `len` bytes of a text, name or GUID, from byte `from` of its cells.
    Text {
        cells: &'a dyn Store,
        from: usize,
        len: usize,
    },
    /// A player's record.
    Player(&'a Shared<Player>),
    /// Records of the stage tables.
    Records(&'a [Record]),
    /// Best-time entries.
    TimeEntries(&'a [TimeEntry]),
}

/// Where a field's values go to, or come from: the file's bytes, in order.
trait Move {
    fn dword(&mut self, field: &dyn DwordLike) -> Result<(), BadInput>;
    /// The dwords of a run (a record), each in turn.
    fn words(&mut self, run: &dyn Run) -> Result<(), BadInput>;
    fn text(&mut self, cells: &dyn Store, from: usize, len: usize) -> Result<(), BadInput>;
    fn player(&mut self, field: &Shared<Player>) -> Result<(), BadInput>;
}

/// The file's bytes made from the fields.
struct Put(Vec<u8>);

impl Move for Put {
    fn dword(&mut self, field: &dyn DwordLike) -> Result<(), BadInput> {
        self.0.extend_from_slice(&field.get().to_le_bytes());
        Ok(())
    }

    fn words(&mut self, run: &dyn Run) -> Result<(), BadInput> {
        for i in 0..run.count() {
            self.0.extend_from_slice(&run.bits(i).to_le_bytes());
        }
        Ok(())
    }

    fn text(&mut self, cells: &dyn Store, from: usize, len: usize) -> Result<(), BadInput> {
        let at = self.0.len();
        self.0.resize(at + len, 0);
        cells.load(from, &mut self.0[at..])
    }

    fn player(&mut self, field: &Shared<Player>) -> Result<(), BadInput> {
        self.0.extend_from_slice(&field.get().to_le_bytes());
        Ok(())
    }
}

/// The fields set from the file's bytes.
struct Take<'b>(&'b [u8]);

impl Take<'_> {
    fn next(&mut self, len: usize) -> Result<&[u8], BadInput> {
        let (now, rest) = self
            .0
            .split_at_checked(len)
            .ok_or(BadInput("too few bytes for an option"))?;
        self.0 = rest;
        Ok(now)
    }

    fn dword(&mut self) -> Result<u32, BadInput> {
        let b = self.next(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

impl Move for Take<'_> {
    fn dword(&mut self, field: &dyn DwordLike) -> Result<(), BadInput> {
        field.set(Take::dword(self)?);
        Ok(())
    }

    fn words(&mut self, run: &dyn Run) -> Result<(), BadInput> {
        for i in 0..run.count() {
            run.set_bits(i, Take::dword(self)?);
        }
        Ok(())
    }

    fn text(&mut self, cells: &dyn Store, from: usize, len: usize) -> Result<(), BadInput> {
        cells.store(from, self.next(len)?)
    }

    fn player(&mut self, field: &Shared<Player>) -> Result<(), BadInput> {
        let bytes = self
            .next(size_of::<Player>())?
            .try_into()
            .map_err(|_| BadInput("too few bytes for a player"))?;
        field.set(Player::from_le_bytes(bytes));
        Ok(())
    }
}

/// A best-time entry's fields, in the file's order.
fn time_entry(entry: &TimeEntry, m: &mut impl Move) -> Result<(), BadInput> {
    for field in [
        &entry.unread_0,
        &entry.player,
        &entry.nation,
        &entry.name_id,
        &entry.car,
        &entry.stage_time,
        &entry.rally_time,
        &entry.group,
    ] {
        m.dword(field)?;
    }
    m.text(&entry.name, 0, entry.name.size())?;
    m.dword(&entry.is_player)?;
    m.dword(&entry.figure)?;
    for field in entry
        .unread_48
        .iter()
        .chain([&entry.unread_50])
        .chain(&entry.unread_54)
        .chain(&entry.checkpoints)
    {
        m.dword(field)?;
    }
    Ok(())
}

impl Field<'_> {
    /// How many bytes it takes in the file.
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Dword(_) => 4,
            Self::Dwords(fields) => 4 * fields.len(),
            Self::Text { len, .. } => *len,
            Self::Player(_) => size_of::<Player>(),
            Self::Records(records) => records.len() * RECORD_LEN as usize,
            Self::TimeEntries(entries) => size_of_val(*entries),
        }
    }

    fn walk(&self, m: &mut impl Move) -> Result<(), BadInput> {
        match self {
            Self::Dword(field) => m.dword(*field),
            Self::Dwords(fields) => fields.iter().try_for_each(|field| m.dword(*field)),
            Self::Text { cells, from, len } => m.text(*cells, *from, *len),
            Self::Player(player) => m.player(player),
            Self::Records(records) => records.iter().try_for_each(|record| m.words(record)),
            Self::TimeEntries(entries) => entries.iter().try_for_each(|entry| time_entry(entry, m)),
        }
    }

    /// Its bytes as the file holds them.
    pub(crate) fn read(&self) -> Result<Vec<u8>, BadInput> {
        let mut put = Put(Vec::with_capacity(self.len()));
        self.walk(&mut put)?;
        Ok(put.0)
    }

    /// Sets its fields from the file's bytes (the first `len()` of them).
    pub(crate) fn write(&self, bytes: &[u8]) -> Result<(), BadInput> {
        self.walk(&mut Take(bytes))
    }
}

/// The file's pieces after its head, in order, each a named field or a run of them.
#[rustfmt::skip]
fn pieces_of(o: Owners<'static>) -> [Field<'static>; 69] {
    let race = o.race;
    let net = o.net;
    let audio = o.audio;
    let system = o.system;
    let saves = o.saves;
    let display = &o.gfx.display;
    let menus = o.menus;

    let text = |cells: &'static dyn Store, len: usize| Field::Text { cells, from: 0, len };
    let work = &menus.options;
    let numbers = &menus.cars.numbers;
    let tables = &system.records().stage_tables;
    let names = |n: usize| Field::Text {
        cells: &menus.play.slots.names.player_names,
        from: 0x20 * n,
        len: 0x20,
    };
    [
        Field::Dword(&system.start().run_count.runs),                   // how many times the game was run
        Field::Dword(&display.mode.menu_mode),                     // the menus' screen mode
        Field::Dword(&display.mode.race_mode),                     // the race's screen mode
        Field::Dword(&display.choices.mipmapping),                 // mipmapping, off or on
        Field::Dword(&audio.state.menu_sound_mode),                     // the menus' sound mode
        Field::Dword(&audio.state.race_sound_mode),                     // the race's sound mode
        Field::Dword(&audio.state.master_volume),                       // the master volume
        Field::Dword(&o.sound.menu.options.unread_option_f0),                     // a sound option kept in the file and read by no other code
        Field::Dword(&o.sound.menu.options.race_music_on),                 // the race music switch
        Field::Dword(&audio.state.tune_turn),                   // which tune comes next
        Field::Dword(&o.sound.menu.options.music_volume),                // the music volume
        Field::Dword(&o.sound.menu.options.race_music_volume),           // the race music's volume
        text(&net.setup().machine_name, 0x40),                         // the machine's name
        Field::Dword(&net.setup().most_players),                          // the most players a hosted session takes
        text(&net.setup().host_address, 0x40),                         // the host's address (TCP/IP)
        Field::Dword(&net.setup().port),                                  // the TCP/IP port
        text(&net.setup().saved_connection, 0x10),                     // the saved connection's GUID
        Field::Dword(&saves.save_games.kept_a),                         // a saved-games word kept in the file and read by no other code
        Field::Dword(&saves.save_games.kept_b),                         // a saved-games word kept in the file and read by no other code
        Field::Dword(&o.menus.service.set_ups.kept),                              // a car set-ups word kept in the file and read by no other code
        text(&race.shared.tyre_change().record_name, 0x20),                  // the tyre change record holder's name
        Field::Dword(&race.shared.tyre_change().record),                        // the tyre change record time
        Field::Dword(&saves.files.options_kept[0]),                                   // four dwords nothing else reads
        Field::Dword(&saves.files.options_kept[1]),
        Field::Dword(&saves.files.options_kept[2]),
        Field::Dword(&saves.files.options_kept[3]),
        Field::Player(&menus.play.slots.records[0]),                 // the four players' records
        Field::Player(&menus.play.slots.records[1]),
        Field::Player(&menus.play.slots.records[2]),
        Field::Player(&menus.play.slots.records[3]),
        Field::Dword(&audio.levels.general_volume),           // the "general volume" slider
        Field::Dword(&work.menu_music_volume),                   // the "menu music" slider
        Field::Dword(&work.race_music_level),                  // the "race music" slider
        Field::Dword(&work.menu_effects_volume),                 // the "menu effects" slider
        Field::Dword(&audio.levels.race_effects_volume),                 // the "race effects" slider
        Field::Dword(&work.menu_voices_volume),                  // the "menu voices" slider
        Field::Dword(&audio.levels.codriver_voice_volume),                      // the "co-driver's voice" slider
        Field::Dword(&audio.levels.engine_volume),                      // the "car engine" slider
        Field::Dwords(vec![                                        // the options pages' work copies, mipmapping .. headlights
            &work.mipmapping, &work.buffering, &work.mode_changed, &work.modes_switch,
            &work.lens_flare, &work.unused_percent_a, &work.unused_percent_b,
            &work.reflections, &work.car_shadows, &work.car_detail, &work.headlights,
        ]),
        Field::Dwords(vec![                                        // the audio quality, the 3D switches and the channels
            &work.audio_quality, &work.ds3d, &work.eax, &work.retired_a3d, &work.dolby, &work.channels,
        ]),
        Field::Dword(&menus.play.champ_view.services),                       // the services counted
        Field::Dword(&menus.screens.flow.kept),                          // kept by the event races
        Field::Dword(&race.shared.state.settings.a8_game),               // the race settings' first word
        Field::Text { cells: &menus.play.arcade.levels_won, from: 0, len: 6 },                    // the arcade levels won, a byte each
        Field::Dwords(numbers.first_numbers.iter().map(|n| n as &dyn DwordLike).collect()),                // the first list of car numbers
        Field::Dwords(numbers.second_numbers[..20].iter().map(|n| n as &dyn DwordLike).collect()),         // the second list of car numbers, 20 entries
        Field::Records(&tables.championship),              // the championship's records
        Field::Records(&tables.events[5].records),          // the events' record tables, in the file's order
        Field::Records(&tables.events[4].records),
        Field::Records(&tables.events[3].records),
        Field::Records(&tables.events[2].records),
        Field::Records(&tables.events[1].records),
        Field::Records(&tables.events[0].records),
        Field::Records(&tables.events[6].records),
        Field::Records(&tables.events[7].records),
        Field::Records(&tables.events[8].records),
        Field::Records(&tables.events[9].records),
        Field::Records(&tables.events[10].records),
        Field::Records(&tables.events[11].records),
        Field::TimeEntries(&saves.rally_times.rallies[0]),                  // the six rallies' best times
        Field::TimeEntries(&saves.rally_times.rallies[1]),
        Field::TimeEntries(&saves.rally_times.rallies[2]),
        Field::TimeEntries(&saves.rally_times.rallies[3]),
        Field::TimeEntries(&saves.rally_times.rallies[4]),
        Field::TimeEntries(&saves.rally_times.rallies[5]),
        names(0),                                               // the four players' names
        names(1),
        names(2),
        names(3),
    ]
}

#[cfg(test)]
pub fn pieces() -> [Field<'static>; 69] {
    pieces_of(crate::test_support::own::owners())
}

#[cfg(test)]
mod tests {

    /// The lengths of the pieces, in the file's order.
    #[rustfmt::skip]
    const OPTIONS: [usize; 69] = [0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x40, 0x4, 0x40, 0x4, 0x10, 0x4, 0x4, 0x4, 0x20, 0x4, 0x4, 0x4, 0x4, 0x4, 0x28C, 0x28C, 0x28C, 0x28C, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x2C, 0x18, 0x4, 0x4, 0x4, 0x6, 0xA0, 0x50, 0x2A48, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0xC30, 0xC30, 0xC30, 0xC30, 0xC30, 0xC30, 0x20, 0x20, 0x20, 0x20];

    /// Each piece has its length and lies whole in its field.
    #[test]
    fn pieces_are_the_options_places() {
        let pieces = super::pieces();
        assert_eq!(
            OPTIONS.iter().sum::<usize>(),
            rc2k_core::options::model::PIECES_LEN
        );
        for (n, (&len, piece)) in OPTIONS.iter().zip(pieces.iter()).enumerate() {
            assert_eq!(piece.len(), len, "piece {n} has another length");
        }
    }

    /// The varied value of byte `i` of piece `n`.
    fn pattern(n: usize, i: usize, flip: u8) -> u8 {
        ((n * 37 + i * 11 + (i >> 8) * 5 + 5) as u8) ^ flip
    }

    fn fill(flip: u8) {
        for (n, piece) in super::pieces().iter().enumerate() {
            let bytes: Vec<u8> = (0..piece.len()).map(|i| pattern(n, i, flip)).collect();
            piece.write(&bytes).unwrap();
        }
    }

    fn file_now() -> Vec<u8> {
        let owned: Vec<Vec<u8>> = super::pieces().iter().map(|p| p.read().unwrap()).collect();
        let refs: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
        rc2k_core::saves::kept::options_file(0x1234_5678, &refs)
    }

    /// The options file of a state with a varied value in every byte of every piece is the
    /// bytes kept in `golden/options.bin` (made by the code before the pieces became named
    /// fields), and reading those bytes back gives the same state.
    #[test]
    fn options_file_of_a_varied_state_is_the_golden_file() {
        let _lock = crate::test_support::test_lock();
        let saved: Vec<Vec<u8>> = super::pieces().iter().map(|p| p.read().unwrap()).collect();
        fill(0);
        let file = file_now();
        if std::env::var_os("RC2K_WRITE_GOLDEN").is_some() {
            std::fs::write(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/saves/golden/options.bin"),
                &file,
            )
            .unwrap();
        }
        let golden: &[u8] = include_bytes!("golden/options.bin");
        let same = file == golden;
        // Another state, then the golden file's pieces put in.
        fill(0xFF);
        let sizes: Vec<usize> = super::pieces().iter().map(super::Field::len).collect();
        let (pieces, ended) = rc2k_core::saves::kept::pieces(golden, &sizes).unwrap();
        assert!(ended);
        for (place, bytes) in super::pieces().iter().zip(&pieces) {
            place.write(bytes).unwrap();
        }
        let loaded = file_now();
        for (place, bytes) in super::pieces().iter().zip(&saved) {
            place.write(bytes).unwrap();
        }
        assert!(same, "the options file changed (len {})", file.len());
        assert!(loaded == golden, "the golden file does not load back");
    }

    /// A state with a varied value in every byte of every piece, captured, overwritten by another
    /// state and applied again, is captured the same, and its pieces are the state's pieces.
    #[test]
    fn capture_then_apply_restores_the_whole_state() {
        let _lock = crate::test_support::test_lock();
        let o = crate::test_support::own::owners();
        let saved: Vec<Vec<u8>> = super::pieces().iter().map(|p| p.read().unwrap()).collect();
        fill(0);
        let first = super::capture(o).unwrap();
        let state: Vec<Vec<u8>> = super::pieces().iter().map(|p| p.read().unwrap()).collect();
        fill(0xFF);
        assert_ne!(super::capture(o).unwrap(), first);
        super::apply(o, &first).unwrap();
        let again = super::capture(o).unwrap();
        let restored: Vec<Vec<u8>> = super::pieces().iter().map(|p| p.read().unwrap()).collect();
        for (place, bytes) in super::pieces().iter().zip(&saved) {
            place.write(bytes).unwrap();
        }
        assert_eq!(again, first);
        assert_eq!(first.to_pieces(), state);
        assert_eq!(restored, state);
    }
}
