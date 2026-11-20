//! Test-only: the registry of the game state's stores, each with a name, so tests can read
//! and put back the game's state by store name (`state::walk`, `state::Snapshot`) and field
//! path rather than by address. Nothing in production uses it.
//!
//! The list is the stores' own (`listed_stores`); a repeated name is found at its first entry.

use crate::memory::cells::Cells;
use crate::owners::Owners;

/// One store of the list: its name, the store, and how many of its first bytes are the
/// game's where that is known (a store's record may be rounded up past the game's last byte).
struct Listed {
    name: &'static str,
    store: &'static (dyn Walk + Sync),
    len: Option<usize>,
}

/// A store of the list: `listed!(name, store)` where all its bytes are the game's,
/// `listed!(name, store, len)` where only its first `len` are.
macro_rules! listed {
    ($name:literal, $store:expr $(,)?) => {
        Listed {
            name: $name,
            store: $store,
            len: None,
        }
    };
    ($name:literal, $store:expr, $len:expr $(,)?) => {
        Listed {
            name: $name,
            store: $store,
            len: Some($len),
        }
    };
}

/// A part of a record of cells, from the field `first` on: a listed store that starts inside a
/// record (`SealWork`'s fields past its alignment lead, the panel past its two lead bytes).
/// Reads and writes go through the whole record's cells by field path.
struct RecordFrom<T: Cells + Sync + 'static> {
    whole: &'static T,
    first: &'static str,
}

/// A part of a game-thread owner (`session::GameThread`) as a listed store, from the field
/// `first` on (empty: the whole record): the record's cells are `!Sync`, so each access checks
/// the thread in debug builds, as `GameThread::get` does.
struct OnGame<S: 'static> {
    whole: &'static S,
    first: &'static str,
}

#[allow(
    unsafe_code,
    reason = "the registry's stores are shared as the owners are"
)]
// SAFETY: as `GameThread`: the data is reached only after the game-thread check.
unsafe impl<S> Sync for OnGame<S> {}

/// A listed store that tests reach by field path: the record's own `Cells`, whole where the
/// store is a part of it.
pub trait Walk {
    /// The bytes of the cells the field `path` of the record behind the store names
    /// (`paths::bytes`), whatever their place in the store.
    fn path_bytes(&self, path: &str) -> Option<Vec<u8>>;
    /// The cells `path` names set from `bytes`; says whether the path and the length fitted.
    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool;
    /// `length` bytes of cells from the field `path` on, in field order (`paths::field_range`).
    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8>;
    /// The cells `range_bytes` names set from `bytes`; says whether the record held that many.
    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool;
    /// The address of the first cell `path` names (`paths::first_addr`).
    fn first_addr(&self, path: &str) -> Option<usize>;
    /// A copy of every cell of the record behind the store; the function returned puts the
    /// copy back (a store that is a part of a record keeps the whole record).
    fn keep(&self) -> Box<dyn Fn() + '_>;
    /// The first field of the store when it is a part of a record that starts later (empty:
    /// the store is the whole record).
    fn first_field(&self) -> &'static str {
        ""
    }
}

/// A copy of every cell of `whole`, and the function that puts it back.
fn kept<T: Cells>(whole: &T) -> Box<dyn Fn() + '_> {
    // On the heap: a large record off the stack.
    let copy: Box<T> = crate::memory::cells::boxed_zeroed_cells();
    copy.copy_from(whole);
    Box::new(move || whole.copy_from(&copy))
}

impl<T: Cells> Walk for T {
    fn path_bytes(&self, path: &str) -> Option<Vec<u8>> {
        super::paths::bytes(self, path)
    }

    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        super::paths::set_bytes(self, path, bytes)
    }

    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8> {
        super::paths::field_range(self, path, length)
    }

    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        super::paths::set_field_range(self, path, bytes)
    }

    fn first_addr(&self, path: &str) -> Option<usize> {
        super::paths::first_addr(self, path)
    }

    fn keep(&self) -> Box<dyn Fn() + '_> {
        kept(self)
    }
}

impl<T: Cells + Sync + 'static> Walk for RecordFrom<T> {
    fn path_bytes(&self, path: &str) -> Option<Vec<u8>> {
        super::paths::bytes(self.whole, path)
    }

    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        super::paths::set_bytes(self.whole, path, bytes)
    }

    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8> {
        super::paths::field_range(self.whole, path, length)
    }

    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        super::paths::set_field_range(self.whole, path, bytes)
    }

    fn first_addr(&self, path: &str) -> Option<usize> {
        super::paths::first_addr(self.whole, path)
    }

    fn keep(&self) -> Box<dyn Fn() + '_> {
        kept(self.whole)
    }

    fn first_field(&self) -> &'static str {
        self.first
    }
}

impl<S: Cells> Walk for OnGame<S> {
    fn path_bytes(&self, path: &str) -> Option<Vec<u8>> {
        crate::memory::session::assert_game_thread();
        super::paths::bytes(self.whole, path)
    }

    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::memory::session::assert_game_thread();
        super::paths::set_bytes(self.whole, path, bytes)
    }

    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8> {
        crate::memory::session::assert_game_thread();
        super::paths::field_range(self.whole, path, length)
    }

    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::memory::session::assert_game_thread();
        super::paths::set_field_range(self.whole, path, bytes)
    }

    fn first_addr(&self, path: &str) -> Option<usize> {
        crate::memory::session::assert_game_thread();
        super::paths::first_addr(self.whole, path)
    }

    fn keep(&self) -> Box<dyn Fn() + '_> {
        crate::memory::session::assert_game_thread();
        kept(self.whole)
    }

    fn first_field(&self) -> &'static str {
        self.first
    }
}

/// The whole of a game-thread owner's part as a listed store (lives for the program).
fn on_game<S: Cells>(whole: &'static S) -> &'static OnGame<S> {
    on_game_from(whole, "")
}

/// A game-thread owner's part from the field `first` on as a listed store.
fn on_game_from<S: Cells>(whole: &'static S, first: &'static str) -> &'static OnGame<S> {
    Box::leak(Box::new(OnGame { whole, first }))
}

/// A record of atomic cells from the field `first` on as a listed store.
fn record_from<T: Cells + Sync>(whole: &'static T, first: &'static str) -> &'static RecordFrom<T> {
    Box::leak(Box::new(RecordFrom { whole, first }))
}

/// Every store, by the path of its owner (the static the store is a field of); a name repeats
/// where one record is listed in parts. Built on first use: the game-thread owners' parts are
/// reached through `GameThread::get`, which checks the thread in debug builds.
fn listed_stores(o: Owners<'static>) -> Vec<Listed> {
    vec![
        listed!(
            "logfile.write_line.state.ENABLED",
            &crate::logfile::write_line::state::ENABLED,
        ),
        listed!(
            "logfile.write_line.state.LEVEL",
            &crate::logfile::write_line::state::LEVEL,
        ),
        listed!(
            "logfile.write_line.state.INDENT",
            &crate::logfile::write_line::state::INDENT,
        ),
        listed!(
            "logfile.write_line.state.LINE_START",
            &crate::logfile::write_line::state::LINE_START,
        ),
        listed!(
            "owners.system.work.numbers",
            on_game(&o.system.work().numbers),
        ),
        listed!(
            "owners.race.shared.random.state",
            &o.race.shared.random.state
        ),
        listed!("owners.system.start.kept", on_game(&o.system.start().kept),),
        listed!(
            "owners.strings.text.show_hours",
            on_game(&o.strings.text.show_hours),
        ),
        listed!(
            "owners.strings.text.seal_text",
            on_game(&o.strings.text.seal_text),
        ),
        listed!(
            "owners.strings.lexicon.recompile_level",
            on_game(&o.strings.lexicon.recompile_level),
        ),
        listed!(
            "owners.strings.language.locale",
            on_game(&o.strings.language.locale),
        ),
        listed!(
            "owners.strings.lexicon.room",
            on_game(&o.strings.lexicon.room),
        ),
        listed!("strings.same_text.LOWER", &crate::strings::same_text::LOWER),
        listed!(
            "owners.strings.language.plain_letters",
            on_game(&o.strings.language.plain_letters),
        ),
        listed!(
            "owners.strings.lexicon.empty_message",
            on_game(&o.strings.lexicon.empty_message),
        ),
        listed!(
            "owners.strings.language.units",
            on_game(&o.strings.language.units)
        ),
        listed!(
            "owners.strings.lexicon.work",
            on_game(&o.strings.lexicon.work),
        ),
        listed!("memory.state.MOST_HELD", &crate::memory::state::MOST_HELD),
        listed!("memory.state.HELD", &crate::memory::state::HELD),
        listed!("memory.state.TURNOVER", &crate::memory::state::TURNOVER),
        listed!(
            "owners.strings.drivers.key",
            on_game(&o.strings.drivers.key),
        ),
        listed!(
            "owners.strings.drivers.names",
            on_game(&o.strings.drivers.names),
        ),
        listed!(
            "owners.gfx.fonts.blended_palette",
            on_game(&o.gfx.fonts.blended_palette),
        ),
        listed!(
            "owners.strings.text.blend_mode",
            on_game(&o.strings.text.blend_mode),
        ),
        listed!("files.state.PATHS", &crate::files::state::PATHS),
        listed!("fatal.state.CODE", &crate::fatal::state::CODE),
        listed!("fatal.state.CAPTION", &crate::fatal::state::CAPTION),
        listed!("fatal.state.REPORTED", &crate::fatal::state::REPORTED),
        listed!("fatal.state.TEXT", &crate::fatal::state::TEXT),
        listed!("files.state.MEDIA_MODE", &crate::files::state::MEDIA_MODE),
        listed!("files.state.DIRECTORIES", &crate::files::state::DIRECTORIES),
        listed!("owners.race.hud.block", on_game(&o.race.hud.get().block),),
        listed!(
            "owners.gfx.pictures.decoder",
            on_game(&o.gfx.pictures.decoder),
        ),
        listed!(
            "owners.strings.lexicon.memory",
            on_game(&o.strings.lexicon.memory),
        ),
        listed!("owners.gfx.sheet", on_game(&o.gfx.sheet),),
        listed!("owners.gfx.pictures.pcx", on_game(&o.gfx.pictures.pcx),),
        listed!("owners.gfx.png.setting", on_game(&o.gfx.png.setting),),
        listed!(
            "owners.gfx.fonts.glyph_tables",
            on_game(&o.gfx.fonts.glyph_tables),
        ),
        listed!(
            "owners.gfx.pictures.showing",
            on_game(&o.gfx.pictures.showing),
        ),
        listed!("owners.gfx.drawing.blit", on_game(&o.gfx.drawing.blit),),
        listed!("owners.gfx.rescale", on_game(&o.gfx.rescale),),
        listed!("owners.gfx.strips", on_game(&o.gfx.strips),),
        listed!(
            "owners.gfx.drawing.row_table",
            on_game(&o.gfx.drawing.row_table),
        ),
        listed!("owners.gfx.pixel.formats", on_game(&o.gfx.pixel.formats),),
        listed!("owners.gfx.drawing.thirds", on_game(&o.gfx.drawing.thirds),),
        listed!("owners.gfx.display.modes", on_game(&o.gfx.display.modes),),
        listed!("owners.gfx.reports", on_game(&o.gfx.reports),),
        listed!("owners.gfx.shots", on_game(&o.gfx.shots),),
        listed!(
            "owners.gfx.pictures.picture",
            on_game(&o.gfx.pictures.picture),
        ),
        listed!("owners.gfx.grading", on_game(&o.gfx.grading),),
        listed!("owners.gfx.drawing.bar", on_game(&o.gfx.drawing.bar),),
        listed!("owners.gfx.png.file", on_game(&o.gfx.png.file),),
        listed!("owners.gfx.png.header", on_game(&o.gfx.png.header),),
        listed!(
            "owners.gfx.pictures.planes",
            on_game(&o.gfx.pictures.planes),
        ),
        listed!("owners.gfx.card.upload", on_game(&o.gfx.card.upload),),
        listed!(
            "owners.install.vars",
            on_game_from(&o.install.vars, "._gap_529dad"),
            0x4b6,
        ),
        listed!(
            "owners.menus.panels.notice",
            on_game(&o.menus.panels.notice),
        ),
        listed!("owners.install.drive_rows", on_game(&o.install.drive_rows),),
        listed!("owners.net.vars", &o.net.vars,),
        listed!("owners.net.link", o.net.link.cells(),),
        listed!(
            "owners.net.main.settings",
            on_game(&o.net.main.get().settings),
        ),
        listed!(
            "owners.net.main.sessions",
            on_game(&o.net.main.get().sessions),
        ),
        listed!("owners.net.main.setup", on_game(&o.net.main.get().setup),),
        listed!(
            "owners.net.main.address",
            on_game(&o.net.main.get().address),
        ),
        listed!("owners.net.main.joined", on_game(&o.net.main.get().joined),),
        listed!("owners.net.main.lobby", on_game(&o.net.main.get().lobby),),
        listed!("owners.net.button", on_game(o.net.button.get()),),
        listed!(
            "owners.gfx.fonts.typefaces",
            on_game(&o.gfx.fonts.typefaces),
        ),
        listed!("owners.saves.save_games", on_game(&o.saves.save_games),),
        listed!(
            "owners.menus.service.set_ups",
            on_game(&o.menus.service.set_ups),
        ),
        listed!(
            "owners.race.shared.replays",
            on_game(o.race.shared.replays()),
        ),
        listed!("owners.install.texts", &o.install.texts,),
        listed!(
            "owners.saves.files.options_path",
            on_game(&o.saves.files.options_path),
        ),
        listed!(
            "owners.saves.files.savegame_template",
            on_game(&o.saves.files.savegame_template),
        ),
        listed!(
            "owners.menus.service.template",
            on_game(&o.menus.service.template),
        ),
        listed!(
            "owners.race.replay.snapshot",
            on_game(o.race.replay.snapshot()),
        ),
        listed!("owners.menus.panels.films", on_game(&o.menus.panels.films),),
        listed!("owners.race.frame.credits", on_game(o.race.frame.credits()),),
        listed!("owners.gfx.card.texture_tables", &o.gfx.card.texture_tables,),
        listed!(
            "owners.gfx.fonts.game_fonts_memory",
            on_game(&o.gfx.fonts.game_fonts_memory),
        ),
        listed!("owners.scene.icon_upload", on_game(o.scene.icon_upload()),),
        listed!("owners.race.frame.map", on_game(o.race.frame.map()),),
        listed!(
            "owners.gfx.drawing.surfaces",
            on_game(&o.gfx.drawing.surfaces),
        ),
        listed!("owners.gfx.locked", on_game(&o.gfx.locked),),
        listed!(
            "owners.gfx.card.windscreen_drop_batch",
            on_game(&o.gfx.card.windscreen_drop_batch),
        ),
        listed!(
            "owners.system.session.screen",
            on_game(o.system.session.screen.get()),
        ),
        listed!("owners.system.session.flags", &o.system.session.flags,),
        listed!("owners.system.session.host", &o.system.session.host,),
        listed!(
            "owners.system.session.key_clock",
            &o.system.session.key_clock,
        ),
        listed!(
            "owners.system.session.picture_clock",
            &o.system.session.picture_clock,
        ),
        listed!("owners.system.window_loop", on_game(o.system.window_loop()),),
        listed!("owners.system.machine", on_game(o.system.machine()),),
        listed!("owners.system.clocks", on_game(o.system.clocks()),),
        listed!("owners.audio.state", &o.audio.state,),
        listed!("owners.audio.sample_records", &o.audio.sample_records,),
        listed!(
            "owners.saves.files.options_kept",
            on_game(&o.saves.files.options_kept),
        ),
        listed!(
            "owners.race.replay.seal_key",
            on_game(o.race.replay.seal_key()),
        ),
        listed!("owners.sound.race.shared", &o.sound.threads.shared,),
        listed!(
            "owners.sound.race.sounds",
            on_game(o.sound.race.sounds.get()),
        ),
        listed!("owners.sound.race.clock", on_game(o.sound.race.clock.get()),),
        listed!(
            "owners.sound.race.surface",
            on_game(o.sound.race.surface.get()),
        ),
        listed!("owners.sound.race.stage", on_game(o.sound.race.stage.get()),),
        listed!(
            "owners.sound.race.spot_picks",
            on_game(o.sound.race.spot_picks.get()),
        ),
        listed!(
            "owners.sound.menu.settings_applied",
            on_game(o.sound.menu.settings_applied.get()),
        ),
        listed!("owners.input.force_feedback", &o.input.force_feedback,),
        listed!("owners.sound.menu.options", &o.sound.menu.options,),
        listed!("owners.input.devices", &o.input.devices,),
        listed!("owners.input.keyboard", on_game(o.input.keyboard()),),
        listed!("owners.input.mouse", on_game(o.input.mouse()),),
        listed!(
            "owners.sound.menu.next_sound_time",
            on_game(o.sound.menu.next_sound_time.get()),
        ),
        listed!(
            "owners.sound.race.sounds_on",
            on_game(o.sound.race.sounds_on.get()),
        ),
        listed!(
            "owners.sound.race.car_stream",
            on_game(o.sound.race.car_stream.get()),
        ),
        listed!(
            "owners.saves.savegame_preview",
            on_game(&o.saves.savegame_preview),
        ),
        listed!(
            "owners.menus.screens.controls",
            on_game(&o.menus.screens.controls),
        ),
        listed!("owners.menus.play.slots", on_game(&o.menus.play.slots),),
        listed!("owners.menus.cars.table", on_game(&o.menus.cars.table),),
        listed!("owners.menus.play.cheats", on_game(&o.menus.play.cheats),),
        listed!("owners.menus.play.results", on_game(&o.menus.play.results),),
        listed!("owners.menus.play.arcade", on_game(&o.menus.play.arcade),),
        listed!(
            "owners.menus.tables.captions",
            on_game(&o.menus.tables.captions),
        ),
        listed!("owners.menus.tables.pages", on_game(&o.menus.tables.pages),),
        listed!("owners.menus.cars.numbers", on_game(&o.menus.cars.numbers),),
        listed!("owners.menus.screens.flow", on_game(&o.menus.screens.flow),),
        listed!(
            "owners.menus.screens.loop_flags",
            on_game(&o.menus.screens.loop_flags),
        ),
        listed!("owners.menus.keys.tables", on_game(&o.menus.keys.tables),),
        listed!("owners.menus.options", on_game(&o.menus.options),),
        listed!(
            "owners.menus.screens.entry",
            on_game(&o.menus.screens.entry),
        ),
        listed!(
            "owners.menus.play.champ_view",
            on_game(&o.menus.play.champ_view),
        ),
        listed!(
            "owners.menus.screens.event",
            on_game(&o.menus.screens.event),
        ),
        listed!(
            "owners.menus.screens.draw_work",
            on_game(&o.menus.screens.draw_work),
        ),
        listed!(
            "owners.system.records.stage_tables",
            on_game(&o.system.records().stage_tables),
        ),
        listed!("owners.menus.cars.drivers", on_game(&o.menus.cars.drivers),),
        listed!("owners.saves.rally_times", on_game(&o.saves.rally_times),),
        listed!("owners.saves.best_times", on_game(&o.saves.best_times),),
        listed!(
            "owners.menus.screens.items",
            on_game(&o.menus.screens.items),
        ),
        listed!(
            "owners.menus.tables.menus.labels",
            on_game(&o.menus.tables.menus.labels),
        ),
        listed!(
            "owners.race.shared.tyre_change",
            on_game(o.race.shared.tyre_change()),
        ),
        listed!(
            "owners.race.replay.seal_work",
            on_game_from(o.race.replay.seal_work(), ".subkeys[0]"),
            0x98,
        ),
        listed!(
            "owners.system.start.texts",
            on_game(&o.system.start().texts),
        ),
        listed!(
            "owners.input.guids",
            record_from(&o.input.guids, ".device2"),
            0x1c0,
        ),
        listed!(
            "owners.sound.menu.menu_sounds",
            record_from(&o.sound.threads.menu_sounds, ".list"),
            0x28,
        ),
        listed!("owners.sound.race.sets", &o.sound.threads.sets, 0x9ac,),
        listed!(
            "owners.sound.race.request",
            on_game(o.sound.race.request.get()),
        ),
        listed!(
            "owners.sound.race.cue_lists",
            on_game_from(o.sound.race.cue_lists.get(), ".table[0]"),
        ),
        listed!(
            "owners.sound.race.kinds",
            on_game(o.sound.race.kinds.get()),
            0x13c8,
        ),
        listed!(
            "owners.sound.race.surface_sets",
            on_game_from(o.sound.race.surface_sets.get(), ".sets[0]"),
            0x1c0,
        ),
        listed!(
            "owners.sound.race.spot_voices",
            on_game(o.sound.race.spot_voices.get()),
        ),
        listed!(
            "owners.install.run",
            on_game_from(&o.install.run, ".same_version"),
        ),
        listed!("owners.install.names", on_game(&o.install.names), 0x8a7,),
        listed!(
            "owners.install.pages",
            on_game_from(&o.install.pages, ".texts[0].dx"),
            0xa25,
        ),
        listed!(
            "owners.install.paths",
            on_game_from(&o.install.paths, ".game_dir"),
            0x600,
        ),
        listed!(
            "owners.install.palette",
            on_game_from(&o.install.palette, ".palette"),
            0x300,
        ),
        listed!(
            "owners.system.start.run_count",
            on_game(&o.system.start().run_count),
        ),
        listed!("owners.install.sku", on_game(&o.install.sku),),
        listed!(
            "owners.strings.text.buffer",
            on_game(&o.strings.text.buffer),
        ),
        listed!("owners.system.work.files", on_game(&o.system.work().files),),
        listed!("owners.gfx.display.mode", on_game(&o.gfx.display.mode),),
        listed!(
            "owners.gfx.drawing.draw_surface",
            on_game(&o.gfx.drawing.draw_surface),
        ),
        listed!("owners.gfx.d3d.cards", on_game(o.gfx.d3d.cards.get()),),
        listed!(
            "owners.gfx.fonts.font_block",
            on_game(&o.gfx.fonts.font_block),
        ),
        listed!("owners.audio.sample_names", &o.audio.sample_names,),
        listed!(
            "owners.gfx.display.choices",
            on_game(&o.gfx.display.choices),
        ),
        listed!("owners.gfx.fonts.jade", on_game(&o.gfx.fonts.jade), 0x1419,),
        listed!(
            "workers.PARTS[0].vertices",
            on_game(&o.scene.scratch.game_scratch().vertices),
        ),
        listed!(
            "workers.PARTS[0].polygon",
            on_game(&o.scene.scratch.game_scratch().polygon),
        ),
        listed!(
            "owners.race.shared.buffers.reads",
            on_game(o.race.shared.buffers.reads.get()),
        ),
        listed!(
            "owners.race.shared.viewer",
            on_game(o.race.shared.viewer_buffers()),
        ),
        listed!(
            "race.viewer_state_store.RECTANGLES",
            &crate::race::viewer_state_store::RECTANGLES,
        ),
        listed!(
            "owners.race.replay.viewer_and_kinds",
            &o.race.replay.viewer_and_kinds,
        ),
        listed!(
            "owners.race.frame.lights_and_gear",
            on_game(o.race.frame.lights_and_gear.get()),
        ),
        listed!(
            "owners.race.shared.car_blocks",
            on_game(o.race.shared.car_blocks.get()),
        ),
        listed!(
            "owners.race.replay.replay_work",
            on_game(o.race.replay.replay_work()),
        ),
        listed!(
            "race.groups_store.PANEL_TAIL",
            on_game_from(o.race.frame.panel_and_overlay.get(), ".panel.split_6",),
        ),
        listed!(
            "owners.race.physics.split_tables",
            on_game(o.race.physics.split_tables()),
        ),
        listed!("owners.race.course.league", on_game(o.race.course.league()),),
        listed!(
            "owners.race.course.lights_data",
            on_game(o.race.course.lights_data()),
        ),
        listed!(
            "owners.race.course.level_paths",
            on_game(o.race.course.level_paths()),
        ),
        listed!("owners.race.physics.pace", on_game(o.race.physics.pace()),),
        listed!(
            "owners.race.physics.ground_probe",
            on_game(o.race.physics.ground_probe.get()),
        ),
        listed!(
            "owners.race.frame.panel_clock",
            on_game(o.race.frame.panel_clock()),
        ),
        listed!(
            "owners.scene.objects.slots",
            on_game(o.scene.objects.slots.get()),
        ),
        listed!(
            "owners.scene.light_tables",
            on_game(o.scene.light_tables.get()),
        ),
        listed!(
            "owners.race.frame.error_list",
            on_game(o.race.frame.error_list()),
        ),
        listed!(
            "owners.race.course.flare_tables",
            on_game(o.race.course.flare_tables()),
        ),
        listed!(
            "owners.race.frame.shadow_skids",
            on_game(o.race.frame.shadow_skids.get()),
        ),
        listed!(
            "owners.race.replay.finish",
            on_game(o.race.replay.finish.get()),
        ),
        listed!(
            "owners.race.shared.replay_file",
            on_game(o.race.shared.replay_file()),
        ),
        listed!(
            "owners.race.replay.replay_view",
            on_game(o.race.replay.replay_view()),
        ),
        listed!(
            "owners.race.course.numbers",
            on_game(o.race.course.numbers.get()),
        ),
        listed!("owners.race.frame.scratch", on_game(o.race.frame.scratch()),),
        listed!(
            "owners.race.course.lists",
            on_game(o.race.course.lists.get()),
        ),
        listed!(
            "owners.race.physics.car_specs",
            on_game(o.race.physics.car_specs()),
        ),
        listed!(
            "owners.race.course.course",
            on_game(o.race.course.course.get()),
        ),
        listed!(
            "owners.scene.objects.tables",
            on_game(o.scene.objects.tables.get()),
        ),
        listed!(
            "owners.scene.textures.places",
            on_game(o.scene.textures.places.get()),
        ),
        listed!(
            "owners.race.frame.matrices",
            on_game(o.race.frame.matrices.get()),
        ),
        listed!("owners.race.shared.tables", on_game(o.race.shared.tables()),),
        listed!(
            "owners.race.frame.view_and_records",
            on_game(o.race.frame.view_and_records.get()),
        ),
        listed!(
            "owners.race.physics.camera",
            on_game(o.race.physics.camera()),
        ),
        listed!(
            "owners.race.course.stage_prefs",
            on_game(o.race.course.stage_prefs.get()),
        ),
        listed!("owners.scene.file_numbers", on_game(o.scene.file_numbers()),),
        listed!(
            "owners.race.shared.physics_tables",
            on_game(o.race.shared.physics_tables.get()),
        ),
        listed!("owners.race.shared.vars", on_game(o.race.shared.vars.get()),),
        listed!(
            "owners.scene.drawing.scene",
            on_game(o.scene.drawing.scene.get()),
        ),
        listed!(
            "owners.scene.lights.state",
            on_game(o.scene.lights.state.get()),
        ),
        listed!("owners.race.shared.view", on_game(o.race.shared.view.get()),),
        listed!(
            "owners.race.course.level_grid",
            on_game(o.race.course.level_grid.get()),
        ),
        listed!(
            "owners.race.frame.frame_work",
            on_game(o.race.frame.frame_work.get()),
        ),
        listed!(
            "owners.race.frame.sound_spots",
            on_game(o.race.frame.sound_spots()),
        ),
        listed!(
            "owners.race.physics.contact",
            on_game(o.race.physics.contact.get()),
        ),
        listed!(
            "owners.race.replay.replay_frames",
            on_game(o.race.replay.replay_frames()),
        ),
        listed!(
            "owners.race.replay.staging",
            on_game(o.race.replay.staging()),
        ),
        listed!("owners.race.shared.car", on_game(o.race.shared.car()),),
        listed!("owners.race.shared.work", on_game(o.race.shared.work.get()),),
        listed!(
            "owners.race.frame.text_state",
            on_game(o.race.frame.text_state()),
        ),
        listed!(
            "owners.race.frame.clock_ticks",
            on_game(o.race.frame.clock_ticks()),
        ),
        listed!(
            "owners.race.course.kind_records",
            on_game(o.race.course.kind_records()),
        ),
        listed!(
            "owners.scene.textures.kinds",
            on_game(o.scene.textures.kinds.get()),
        ),
        listed!("owners.race.frame.counter", on_game(o.race.frame.counter()),),
        listed!(
            "owners.race.shared.switches",
            on_game(o.race.shared.switches.get()),
        ),
        listed!(
            "owners.scene.draw_switches",
            on_game(o.scene.draw_switches.get()),
        ),
        listed!(
            "owners.scene.lights.work",
            on_game(o.scene.lights.work.get()),
        ),
        listed!(
            "owners.scene.drawing.object_work",
            on_game(o.scene.object_work()),
        ),
        listed!(
            "owners.race.physics.ground",
            on_game(o.race.physics.ground()),
        ),
        listed!(
            "owners.race.frame.draw_batches",
            on_game(o.race.frame.draw_batches()),
        ),
        listed!(
            "owners.effects.setup.work",
            on_game(&o.race.effects.get().setup.work),
        ),
        listed!(
            "owners.menus.screens.state",
            on_game(&o.menus.screens.state),
        ),
        listed!(
            "owners.menus.screens.carousel",
            on_game(&o.menus.screens.carousel),
        ),
        listed!("owners.race.shared.state", &o.race.shared.state,),
        listed!(
            "owners.race.physics.damage",
            on_game(o.race.physics.damage()),
        ),
        listed!(
            "owners.effects.windscreen_drops.inputs",
            on_game(&o.race.effects.get().windscreen_drops.inputs),
        ),
        listed!(
            "owners.effects.frame.view_sources",
            on_game(&o.race.effects.get().frame.view_sources),
        ),
        listed!(
            "owners.effects.setup.clock",
            on_game(&o.race.effects.get().setup.clock),
        ),
        listed!(
            "owners.effects.setup.frame",
            on_game(&o.race.effects.get().setup.frame),
        ),
        listed!(
            "owners.effects.setup.floats",
            on_game(&o.race.effects.get().setup.floats),
        ),
        listed!(
            "owners.effects.setup.kind_floats",
            on_game(&o.race.effects.get().setup.kind_floats),
        ),
        listed!(
            "owners.effects.setup.settings",
            on_game(&o.race.effects.get().setup.settings),
        ),
        listed!(
            "owners.effects.pong.state",
            on_game(&o.race.effects.get().pong.state),
        ),
        listed!(
            "owners.effects.clipping.clip",
            on_game(&o.race.effects.get().clipping.clip),
        ),
        listed!(
            "owners.effects.pong.rules",
            on_game(&o.race.effects.get().pong.rules),
        ),
        listed!(
            "owners.effects.kinds.marks_kind",
            on_game(&o.race.effects.get().kinds.marks_kind),
        ),
        listed!(
            "owners.effects.frame.kept_view",
            on_game(&o.race.effects.get().frame.kept_view),
        ),
        listed!(
            "owners.effects.paths.spline_head",
            on_game(&o.race.effects.get().paths.spline_head),
        ),
        listed!(
            "owners.effects.paths.spline_work",
            on_game(&o.race.effects.get().paths.spline_work),
        ),
        listed!(
            "owners.effects.paths.spline_tables",
            on_game(&o.race.effects.get().paths.spline_tables),
        ),
        listed!(
            "owners.effects.paths.spline_kinds",
            on_game(&o.race.effects.get().paths.spline_kinds),
        ),
        listed!(
            "owners.effects.paths.spline_path",
            on_game(&o.race.effects.get().paths.spline_path),
        ),
        listed!(
            "owners.effects.paths.spline_constants",
            on_game(&o.race.effects.get().paths.spline_constants),
        ),
        listed!(
            "owners.effects.masks.edge_mask",
            on_game(&o.race.effects.get().masks.edge_mask),
        ),
        listed!(
            "owners.effects.windscreen_drops.tilt",
            on_game(&o.race.effects.get().windscreen_drops.tilt),
        ),
        listed!(
            "owners.effects.masks.kind_boxes",
            on_game(&o.race.effects.get().masks.kind_boxes),
        ),
        listed!(
            "owners.effects.windscreen_drops.state",
            on_game(&o.race.effects.get().windscreen_drops.state),
        ),
        listed!(
            "owners.effects.windscreen_drops.work",
            on_game(&o.race.effects.get().windscreen_drops.work),
        ),
        listed!(
            "owners.effects.windscreen_drops.hex",
            on_game(&o.race.effects.get().windscreen_drops.hex),
        ),
        listed!(
            "owners.effects.masks.vars",
            on_game(&o.race.effects.get().masks.vars),
        ),
        listed!(
            "owners.effects.windscreen_drops.picks",
            on_game(&o.race.effects.get().windscreen_drops.picks),
        ),
        listed!(
            "owners.effects.windscreen_drops.shape_work",
            on_game(&o.race.effects.get().windscreen_drops.shape_work,),
        ),
        listed!(
            "owners.effects.windscreen_drops.tuning",
            on_game(&o.race.effects.get().windscreen_drops.tuning),
        ),
        listed!(
            "owners.effects.setup.mask_constants",
            on_game(&o.race.effects.get().setup.mask_constants),
        ),
        listed!(
            "owners.effects.windscreen_flakes.timing",
            on_game(&o.race.effects.get().windscreen_flakes.timing,),
        ),
        listed!(
            "owners.effects.windscreen_flakes.tables",
            on_game(&o.race.effects.get().windscreen_flakes.tables,),
        ),
        listed!(
            "owners.effects.windscreen_flakes.items",
            on_game(&o.race.effects.get().windscreen_flakes.items),
        ),
        listed!(
            "owners.effects.windscreen_flakes.state",
            on_game(&o.race.effects.get().windscreen_flakes.state),
        ),
        listed!(
            "owners.effects.masks.edge_settings",
            on_game(&o.race.effects.get().masks.edge_settings),
        ),
        listed!(
            "owners.effects.weather.rain",
            on_game(&o.race.effects.get().weather.rain),
        ),
        listed!(
            "owners.effects.weather.snow_tables",
            on_game(&o.race.effects.get().weather.snow_tables),
        ),
        listed!(
            "owners.effects.weather.snow",
            on_game(&o.race.effects.get().weather.snow),
        ),
        listed!(
            "owners.effects.weather.snow_curve",
            on_game(&o.race.effects.get().weather.snow_curve),
        ),
        listed!(
            "owners.effects.weather.snow_vars",
            on_game(&o.race.effects.get().weather.snow_vars),
        ),
        listed!(
            "owners.effects.weather.snow_light",
            on_game(&o.race.effects.get().weather.snow_light),
        ),
        listed!(
            "owners.effects.blocks.places",
            on_game(&o.race.effects.get().blocks.places),
        ),
        listed!(
            "owners.effects.paths.profile_rings",
            on_game(&o.race.effects.get().paths.profile_rings),
        ),
        listed!(
            "owners.effects.paths.profile_tables",
            on_game(&o.race.effects.get().paths.profile_tables),
        ),
        listed!(
            "owners.effects.frame.draw_work",
            on_game(&o.race.effects.get().frame.draw_work),
        ),
        listed!(
            "owners.effects.blocks.corners",
            on_game(&o.race.effects.get().blocks.corners),
        ),
        listed!(
            "owners.effects.frame.view_vars",
            on_game(&o.race.effects.get().frame.view_vars),
        ),
        listed!(
            "owners.effects.frame.draw_frame",
            on_game(&o.race.effects.get().frame.draw_frame),
        ),
        listed!(
            "owners.effects.paths.sines",
            on_game(&o.race.effects.get().paths.sines),
        ),
        listed!(
            "owners.effects.frame.flare_parts",
            on_game(&o.race.effects.get().frame.flare_parts),
        ),
        listed!(
            "owners.effects.paths.wave_levels",
            on_game(&o.race.effects.get().paths.wave_levels),
        ),
        listed!(
            "owners.effects.paths.wave",
            on_game(&o.race.effects.get().paths.wave),
        ),
        listed!(
            "owners.effects.frame.flare_scales",
            on_game(&o.race.effects.get().frame.flare_scales),
        ),
        listed!(
            "owners.effects.blocks.links",
            on_game(&o.race.effects.get().blocks.links),
        ),
        listed!(
            "owners.effects.blocks.corner_slopes",
            on_game(&o.race.effects.get().blocks.corner_slopes),
        ),
        listed!(
            "owners.effects.paths.spread_random",
            on_game(&o.race.effects.get().paths.spread_random),
        ),
        listed!(
            "owners.effects.kinds.overlay_quad",
            on_game(&o.race.effects.get().kinds.overlay_quad),
        ),
        listed!(
            "owners.effects.kinds.overlay_extras",
            on_game(&o.race.effects.get().kinds.overlay_extras),
        ),
        listed!(
            "owners.effects.kinds.shadow_quad",
            on_game(&o.race.effects.get().kinds.shadow_quad),
        ),
        listed!(
            "owners.effects.kinds.shadow_extra",
            on_game(&o.race.effects.get().kinds.shadow_extra),
        ),
        listed!(
            "owners.effects.kinds.plane_quad",
            on_game(&o.race.effects.get().kinds.plane_quad),
        ),
        listed!(
            "owners.effects.masks.ground_names",
            on_game(&o.race.effects.get().masks.ground_names),
            0xa6,
        ),
        listed!(
            "owners.effects.masks.blocks",
            on_game(&o.race.effects.get().masks.blocks),
        ),
        listed!(
            "owners.effects.masks.picture_copy",
            on_game(&o.race.effects.get().masks.picture_copy),
        ),
        listed!(
            "owners.effects.blocks.slots",
            on_game(&o.race.effects.get().blocks.slots),
        ),
        listed!(
            "owners.effects.blocks.slot_tuning",
            on_game(&o.race.effects.get().blocks.slot_tuning),
        ),
        listed!(
            "owners.effects.kinds.emitter",
            on_game(&o.race.effects.get().kinds.emitter),
        ),
        listed!(
            "owners.effects.ring.work",
            on_game(&o.race.effects.get().ring.work),
        ),
        listed!(
            "owners.effects.ring.names",
            on_game(&o.race.effects.get().ring.names),
        ),
        listed!(
            "owners.effects.masks.surface",
            on_game(&o.race.effects.get().masks.surface),
        ),
        listed!(
            "owners.effects.particles.twin_state",
            on_game(&o.race.effects.get().particles.twin_state),
        ),
        listed!(
            "owners.effects.particles.kind1_choice",
            on_game(&o.race.effects.get().particles.kind1_choice),
        ),
        listed!(
            "owners.effects.masks.list",
            on_game(&o.race.effects.get().masks.list),
        ),
        listed!(
            "owners.effects.particles.kind5_set",
            on_game(&o.race.effects.get().particles.kind5_set),
        ),
        listed!(
            "owners.effects.particles.kind11_set",
            on_game(&o.race.effects.get().particles.kind11_set),
        ),
        listed!(
            "owners.effects.particles.kind21_set",
            on_game(&o.race.effects.get().particles.kind21_set),
        ),
        listed!(
            "owners.effects.particles.kind1_set",
            on_game(&o.race.effects.get().particles.kind1_set),
        ),
        listed!(
            "owners.effects.particles.first_tuning",
            on_game(&o.race.effects.get().particles.first_tuning),
        ),
        listed!(
            "owners.effects.particles.second_tuning",
            on_game(&o.race.effects.get().particles.second_tuning),
        ),
        listed!(
            "owners.effects.particles.kind6_set",
            on_game(&o.race.effects.get().particles.kind6_set),
        ),
        listed!(
            "owners.effects.particles.kind13_set",
            on_game(&o.race.effects.get().particles.kind13_set),
        ),
        listed!(
            "owners.effects.particles.nineteenth_tuning",
            on_game(&o.race.effects.get().particles.nineteenth_tuning,),
        ),
        listed!(
            "owners.effects.particles.twenty_second_tuning",
            on_game(&o.race.effects.get().particles.twenty_second_tuning,),
        ),
        listed!(
            "owners.effects.ring.pool",
            on_game(&o.race.effects.get().ring.pool),
        ),
        listed!(
            "owners.effects.particles.puff_work",
            on_game(&o.race.effects.get().particles.puff_work),
        ),
        listed!(
            "owners.effects.kinds.burst_colours",
            on_game(&o.race.effects.get().kinds.burst_colours),
        ),
        listed!(
            "owners.effects.kinds.particle_tuning",
            on_game(&o.race.effects.get().kinds.particle_tuning),
        ),
        listed!(
            "owners.effects.clipping.blend_work",
            on_game(&o.race.effects.get().clipping.blend_work),
        ),
        listed!(
            "owners.effects.paths.morph_work",
            on_game(&o.race.effects.get().paths.morph_work),
        ),
        listed!(
            "owners.effects.paths.lines_work",
            on_game(&o.race.effects.get().paths.lines_work),
        ),
        listed!(
            "owners.effects.setup.clock_k",
            on_game(&o.race.effects.get().setup.clock_k),
        ),
        listed!(
            "owners.effects.clipping.glow",
            on_game(&o.race.effects.get().clipping.glow),
        ),
    ]
}

/// A store: its name, its cells (by field path), and
/// how many bytes the game's range holds where that is known.
pub struct Entry {
    pub name: &'static str,
    pub store: &'static (dyn Walk + Sync),
    pub len: Option<usize>,
}

/// The stores of the owners `o` in the list's order (the golden file numbers a repeated name in
/// this order); the stores that belong to no owner are the same in every list.
pub fn listed_of(o: Owners<'static>) -> &'static [Entry] {
    let list: Vec<Entry> = listed_stores(o)
        .into_iter()
        .map(|l| Entry {
            name: l.name,
            store: l.store,
            len: l.len,
        })
        .collect();
    Box::leak(list.into_boxed_slice())
}

/// The stores of the owners the tests reach their state through (`Owners::the()`), built once
/// per test thread.
pub fn listed() -> &'static [Entry] {
    thread_local! {
        static LIST: &'static [Entry] = listed_of(Owners::the());
    }
    LIST.with(|list| *list)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A store's game range is not empty; a store that starts inside a record names a field the
    /// record has.
    #[test]
    fn every_store_holds_its_game_range() {
        let _g = crate::test_support::test_lock();
        for e in listed() {
            assert!(e.len.is_none_or(|len| len > 0), "{}", e.name);
            let first = e.store.first_field();
            if !first.is_empty() {
                assert!(e.store.path_bytes(first).is_some(), "{}: {first}", e.name);
            }
        }
    }
}
