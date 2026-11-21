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
/// record (the panel past its two lead bytes).
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
        listed!("owners.gfx.card.texture_tables", &o.gfx.card.texture_tables,),
        listed!(
            "owners.gfx.fonts.game_fonts_memory",
            on_game(&o.gfx.fonts.game_fonts_memory),
        ),
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
        listed!("owners.sound.race.shared", &o.sound.threads.shared,),
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
            "owners.saves.savegame_preview",
            on_game(&o.saves.savegame_preview),
        ),
        listed!(
            "owners.menus.screens.controls",
            on_game(&o.menus.screens.controls),
        ),
        listed!(
            "owners.menus.play.slots.pointers",
            on_game(&o.menus.play.slots.pointers),
        ),
        listed!(
            "owners.system.records.stage_tables",
            on_game(&o.system.records().stage_tables),
        ),
        listed!("owners.saves.rally_times", on_game(&o.saves.rally_times),),
        listed!("owners.saves.best_times", on_game(&o.saves.best_times),),
        listed!(
            "owners.menus.tables.menus.labels",
            on_game(&o.menus.tables.menus.labels),
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
            "owners.install.pages",
            on_game_from(&o.install.pages, ".texts[0].dx"),
            0xa25,
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
            "race.groups_store.PANEL_TAIL",
            on_game_from(o.race.frame.panel_and_overlay.get(), ".panel.split_6",),
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
            "owners.race.frame.shadow_skids",
            on_game(o.race.frame.shadow_skids.get()),
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
        listed!(
            "owners.race.course.lists",
            on_game(o.race.course.lists.get()),
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
            "owners.scene.textures.place_slots",
            on_game(o.scene.textures.place_slots.get()),
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
            "owners.race.course.stage_prefs",
            on_game(o.race.course.stage_prefs.get()),
        ),
        listed!(
            "owners.race.shared.physics_tables",
            on_game(o.race.shared.physics_tables.get()),
        ),
        listed!("owners.race.shared.vars", on_game(o.race.shared.vars.get()),),
        listed!(
            "owners.scene.drawing.scene",
            on_game(o.scene.drawing.scene.get()),
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
            "owners.race.physics.lit_faces",
            on_game(o.race.physics.lit_faces.get()),
        ),
        listed!("owners.race.shared.car", on_game(o.race.shared.car()),),
        listed!("owners.race.shared.work", on_game(o.race.shared.work.get()),),
        listed!(
            "owners.race.course.kind_records",
            on_game(o.race.course.kind_records()),
        ),
        listed!(
            "owners.scene.textures.kinds",
            on_game(o.scene.textures.kinds.get()),
        ),
        listed!(
            "owners.race.shared.switches",
            on_game(o.race.shared.switches.get()),
        ),
        listed!(
            "owners.race.physics.ground",
            on_game(o.race.physics.ground()),
        ),
        listed!("owners.race.shared.state", &o.race.shared.state,),
        listed!(
            "owners.effects.setup.frame",
            on_game(&o.race.effects.get().setup.frame),
        ),
        listed!(
            "owners.effects.masks.edge_mask",
            on_game(&o.race.effects.get().masks.edge_mask),
        ),
        listed!(
            "owners.effects.masks.kind_boxes",
            on_game(&o.race.effects.get().masks.kind_boxes),
        ),
        listed!(
            "owners.effects.windscreen_flakes.items",
            on_game(&o.race.effects.get().windscreen_flakes.items),
        ),
        listed!(
            "owners.effects.masks.edge_settings",
            on_game(&o.race.effects.get().masks.edge_settings),
        ),
        listed!(
            "owners.effects.weather.snow_curve",
            on_game(&o.race.effects.get().weather.snow_curve),
        ),
        listed!(
            "owners.effects.blocks.places",
            on_game(&o.race.effects.get().blocks.places),
        ),
        listed!(
            "owners.effects.blocks.corners",
            on_game(&o.race.effects.get().blocks.corners),
        ),
        listed!(
            "owners.effects.blocks.links",
            on_game(&o.race.effects.get().blocks.links),
        ),
        listed!(
            "owners.effects.kinds.overlay_extras",
            on_game(&o.race.effects.get().kinds.overlay_extras),
        ),
        listed!(
            "owners.effects.kinds.shadow_extra",
            on_game(&o.race.effects.get().kinds.shadow_extra),
        ),
        listed!(
            "owners.effects.masks.ground_names",
            on_game(&o.race.effects.get().masks.ground_names),
            0xa6,
        ),
        listed!(
            "owners.effects.ring.names",
            on_game(&o.race.effects.get().ring.names),
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
            "owners.effects.ring.pool",
            on_game(&o.race.effects.get().ring.pool),
        ),
        listed!(
            "owners.effects.kinds.particle_tuning",
            on_game(&o.race.effects.get().kinds.particle_tuning),
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
fn listed_of(o: Owners<'static>) -> &'static [Entry] {
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
