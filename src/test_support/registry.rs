//! Test-only: the registry of the game state's stores, each with a name, so tests can read
//! and put back the game's state by store name (`state::walk`, `state::Snapshot`) and field
//! path rather than by address. Nothing in production uses it.
//!
//! The list is the stores' own (`listed_stores`); a repeated name is found at its first entry.

use crate::cells::Cells;
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
    let copy: Box<T> = crate::cells::boxed_zeroed_cells();
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
        crate::session::assert_game_thread();
        super::paths::bytes(self.whole, path)
    }

    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::session::assert_game_thread();
        super::paths::set_bytes(self.whole, path, bytes)
    }

    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8> {
        crate::session::assert_game_thread();
        super::paths::field_range(self.whole, path, length)
    }

    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::session::assert_game_thread();
        super::paths::set_field_range(self.whole, path, bytes)
    }

    fn first_addr(&self, path: &str) -> Option<usize> {
        crate::session::assert_game_thread();
        super::paths::first_addr(self.whole, path)
    }

    fn keep(&self) -> Box<dyn Fn() + '_> {
        crate::session::assert_game_thread();
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
            "logfile.write_line.state.LOCK",
            &crate::logfile::write_line::state::LOCK,
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
            "system.owner.SYSTEM.work.numbers",
            on_game(&o.system.work().numbers),
        ),
        listed!(
            "logfile.start_log.FREE_SPACE_KB",
            on_game(crate::logfile::start_log::FREE_SPACE_KB.get()),
        ),
        listed!("math.next_random.STATE", &crate::math::next_random::STATE),
        listed!(
            "system.owner.SYSTEM.start.kept",
            on_game(&o.system.start().kept),
        ),
        listed!(
            "strings.copy_printable.KEEP_CASE",
            &crate::strings::copy_printable::KEEP_CASE,
        ),
        listed!(
            "strings.append_time.SHOW_HOURS",
            on_game(crate::strings::append_time::SHOW_HOURS.get()),
        ),
        listed!(
            "strings.format_seal_code.SEAL_TEXT",
            on_game(crate::strings::format_seal_code::SEAL_TEXT.get()),
        ),
        listed!(
            "strings.maybe_compile_lexicon.RECOMPILE_LEVEL",
            on_game(crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL.get()),
        ),
        listed!(
            "strings.owner.STRINGS.language.locale",
            on_game(&o.strings.language.locale),
        ),
        listed!(
            "strings.string_table.LEXICON",
            on_game(crate::strings::string_table::LEXICON.get()),
        ),
        listed!("strings.same_text.LOWER", &crate::strings::same_text::LOWER),
        listed!(
            "strings.owner.STRINGS.language.plain_letters",
            on_game(&o.strings.language.plain_letters),
        ),
        listed!(
            "strings.owner.STRINGS.lexicon.empty_message",
            on_game(&o.strings.lexicon.empty_message),
        ),
        listed!(
            "strings.units.UNITS",
            on_game(crate::strings::units::UNITS.get())
        ),
        listed!(
            "strings.owner.STRINGS.lexicon.work",
            on_game(&o.strings.lexicon.work),
        ),
        listed!("memory.state.MOST_HELD", &crate::memory::state::MOST_HELD),
        listed!("memory.state.HELD", &crate::memory::state::HELD),
        listed!("memory.state.TURNOVER", &crate::memory::state::TURNOVER),
        listed!(
            "memory.state.OWN_HEAP",
            on_game(crate::memory::state::OWN_HEAP.get())
        ),
        listed!(
            "memory.state.OWN_HEAP_KB",
            on_game(crate::memory::state::OWN_HEAP_KB.get()),
        ),
        listed!(
            "strings.owner.STRINGS.drivers.key",
            on_game(&o.strings.drivers.key),
        ),
        listed!(
            "strings.owner.STRINGS.drivers.names",
            on_game(&o.strings.drivers.names),
        ),
        listed!(
            "threading.thread_slot.SLOTS",
            &crate::threading::thread_slot::SLOTS,
        ),
        listed!(
            "text.blend_font.BLENDED_PALETTE",
            on_game(crate::text::blend_font::BLENDED_PALETTE.get()),
        ),
        listed!(
            "strings.owner.STRINGS.text.blend_mode",
            on_game(&o.strings.text.blend_mode),
        ),
        listed!(
            "text.blend_font.BLENDED_FONT",
            on_game(crate::text::blend_font::BLENDED_FONT.get()),
        ),
        listed!("files.state.FIND_HANDLE", &crate::files::state::FIND_HANDLE),
        listed!(
            "files.state.FIND_DATA",
            &crate::files::state::FIND_DATA,
            0x13e,
        ),
        listed!(
            "files.state.SKIPPED_ATTRIBUTES",
            &crate::files::state::SKIPPED_ATTRIBUTES,
        ),
        listed!("files.state.PACKED_PATH", &crate::files::state::PACKED_PATH),
        listed!("files.state.BYTES_READ", &crate::files::state::BYTES_READ),
        listed!("files.state.PATHS", &crate::files::state::PATHS),
        listed!("fatal.state.W32", &crate::fatal::state::W32),
        listed!("fatal.state.CODE", &crate::fatal::state::CODE),
        listed!("fatal.state.CAPTION", &crate::fatal::state::CAPTION),
        listed!("fatal.state.TEMPLATE", &crate::fatal::state::TEMPLATE),
        listed!("fatal.state.REPORTED", &crate::fatal::state::REPORTED),
        listed!("fatal.state.EXTRA", &crate::fatal::state::EXTRA),
        listed!("fatal.state.TEXT", &crate::fatal::state::TEXT),
        listed!("files.state.MEDIA_MODE", &crate::files::state::MEDIA_MODE),
        listed!("files.state.DIRECTORIES", &crate::files::state::DIRECTORIES),
        listed!(
            "race.owner.RACE.hud.block",
            on_game(&o.race.hud.get().block),
        ),
        listed!(
            "gfx.owner.GFX.pictures.decoder",
            on_game(&o.gfx.pictures.decoder),
        ),
        listed!(
            "strings.owner.STRINGS.lexicon.memory",
            on_game(&o.strings.lexicon.memory),
        ),
        listed!("gfx.owner.GFX.sheet", on_game(&o.gfx.sheet),),
        listed!("gfx.owner.GFX.pictures.pcx", on_game(&o.gfx.pictures.pcx),),
        listed!("gfx.owner.GFX.png.setting", on_game(&o.gfx.png.setting),),
        listed!(
            "gfx.owner.GFX.fonts.glyph_tables",
            on_game(&o.gfx.fonts.glyph_tables),
        ),
        listed!(
            "gfx.owner.GFX.pictures.showing",
            on_game(&o.gfx.pictures.showing),
        ),
        listed!("gfx.owner.GFX.drawing.blit", on_game(&o.gfx.drawing.blit),),
        listed!("gfx.owner.GFX.rescale", on_game(&o.gfx.rescale),),
        listed!("gfx.owner.GFX.strips", on_game(&o.gfx.strips),),
        listed!(
            "gfx.owner.GFX.drawing.row_table",
            on_game(&o.gfx.drawing.row_table),
        ),
        listed!("gfx.owner.GFX.pixel.formats", on_game(&o.gfx.pixel.formats),),
        listed!(
            "gfx.owner.GFX.drawing.thirds",
            on_game(&o.gfx.drawing.thirds),
        ),
        listed!("gfx.owner.GFX.display.modes", on_game(&o.gfx.display.modes),),
        listed!(
            "gfx.owner.GFX.pixel.converters",
            on_game(&o.gfx.pixel.converters),
        ),
        listed!("gfx.owner.GFX.reports", on_game(&o.gfx.reports),),
        listed!("gfx.owner.GFX.shots", on_game(&o.gfx.shots),),
        listed!(
            "gfx.owner.GFX.pictures.picture",
            on_game(&o.gfx.pictures.picture),
        ),
        listed!("gfx.owner.GFX.grading", on_game(&o.gfx.grading),),
        listed!("gfx.owner.GFX.drawing.bar", on_game(&o.gfx.drawing.bar),),
        listed!("gfx.owner.GFX.png.file", on_game(&o.gfx.png.file),),
        listed!("gfx.owner.GFX.png.header", on_game(&o.gfx.png.header),),
        listed!(
            "gfx.owner.GFX.pictures.planes",
            on_game(&o.gfx.pictures.planes),
        ),
        listed!("gfx.owner.GFX.display.cards", on_game(&o.gfx.display.cards),),
        listed!("gfx.owner.GFX.card.upload", on_game(&o.gfx.card.upload),),
        listed!(
            "install.owner.INSTALL.vars",
            on_game_from(&o.install.vars, "._gap_529dad"),
            0x4b6,
        ),
        listed!(
            "menu.owner.MENU_OWNER.panels.notice",
            on_game(&o.menus.panels.notice),
        ),
        listed!(
            "install.owner.INSTALL.drive_rows",
            on_game(&o.install.drive_rows),
        ),
        listed!("net.owner.NET_OWNER.vars", &o.net.vars,),
        listed!("net.owner.NET_OWNER.link", o.net.link.cells(),),
        listed!(
            "net.owner.NET_OWNER.main.settings",
            on_game(&o.net.main.get().settings),
        ),
        listed!(
            "net.owner.NET_OWNER.main.sessions",
            on_game(&o.net.main.get().sessions),
        ),
        listed!(
            "net.owner.NET_OWNER.main.setup",
            on_game(&o.net.main.get().setup),
        ),
        listed!(
            "net.owner.NET_OWNER.main.address",
            on_game(&o.net.main.get().address),
        ),
        listed!(
            "net.owner.NET_OWNER.main.joined",
            on_game(&o.net.main.get().joined),
        ),
        listed!(
            "net.owner.NET_OWNER.main.lobby",
            on_game(&o.net.main.get().lobby),
        ),
        listed!("net.owner.NET_OWNER.button", on_game(o.net.button.get()),),
        listed!(
            "gfx.owner.GFX.fonts.typefaces",
            on_game(&o.gfx.fonts.typefaces),
        ),
        listed!("saves.owner.SAVES.save_games", on_game(&o.saves.save_games),),
        listed!(
            "menu.owner.MENU_OWNER.service.set_ups",
            on_game(&o.menus.service.set_ups),
        ),
        listed!(
            "race.owner.RACE.shared.replays",
            on_game(o.race.shared.replays()),
        ),
        listed!("install.owner.INSTALL.texts", &o.install.texts,),
        listed!(
            "saves.owner.SAVES.files.options_path",
            on_game(&o.saves.files.options_path),
        ),
        listed!(
            "saves.owner.SAVES.files.savegame_template",
            on_game(&o.saves.files.savegame_template),
        ),
        listed!(
            "menu.owner.MENU_OWNER.service.template",
            on_game(&o.menus.service.template),
        ),
        listed!(
            "race.owner.RACE.replay.snapshot",
            on_game(o.race.replay.snapshot()),
        ),
        listed!(
            "menu.owner.MENU_OWNER.panels.films",
            on_game(&o.menus.panels.films),
        ),
        listed!(
            "screens.draw_message_box.BOX",
            on_game(crate::screens::draw_message_box::BOX.get()),
        ),
        listed!(
            "race.owner.RACE.frame.credits",
            on_game(o.race.frame.credits()),
        ),
        listed!(
            "system.check_developer_machine.DEVELOPER",
            on_game(crate::system::check_developer_machine::DEVELOPER.get()),
        ),
        listed!(
            "gfx.owner.GFX.card.texture_tables",
            &o.gfx.card.texture_tables,
        ),
        listed!("film.NO_FMV", on_game(crate::film::NO_FMV.get())),
        listed!(
            "gfx.owner.GFX.fonts.game_fonts_memory",
            on_game(&o.gfx.fonts.game_fonts_memory),
        ),
        listed!(
            "scene.owner.SCENE.icon_upload",
            on_game(o.scene.icon_upload()),
        ),
        listed!("race.owner.RACE.frame.map", on_game(o.race.frame.map()),),
        listed!(
            "gfx.owner.GFX.drawing.surfaces",
            on_game(&o.gfx.drawing.surfaces),
        ),
        listed!("gfx.owner.GFX.locked", on_game(&o.gfx.locked),),
        listed!(
            "gfx.owner.GFX.card.windscreen_drop_batch",
            on_game(&o.gfx.card.windscreen_drop_batch),
        ),
        listed!(
            "system.owner.SYSTEM.session.screen",
            on_game(o.system.session.screen.get()),
        ),
        listed!("system.owner.SYSTEM.session.flags", &o.system.session.flags,),
        listed!("system.owner.SYSTEM.session.host", &o.system.session.host,),
        listed!(
            "system.owner.SYSTEM.session.key_clock",
            &o.system.session.key_clock,
        ),
        listed!(
            "system.owner.SYSTEM.window_loop",
            on_game(o.system.window_loop()),
        ),
        listed!("system.owner.SYSTEM.machine", on_game(o.system.machine()),),
        listed!("system.owner.SYSTEM.clocks", on_game(o.system.clocks()),),
        listed!("audio.owner.AUDIO_OWNER.state", &o.audio.state,),
        listed!(
            "audio.owner.AUDIO_OWNER.sample_records",
            &o.audio.sample_records,
        ),
        listed!("audio.owner.AUDIO_OWNER.environment", &o.audio.environment,),
        listed!(
            "saves.owner.SAVES.files.options_kept",
            on_game(&o.saves.files.options_kept),
        ),
        listed!(
            "sound.store.WHEEL_SURFACES",
            on_game(crate::sound::store::WHEEL_SURFACES.get()),
        ),
        listed!(
            "race.owner.RACE.replay.seal_key",
            on_game(o.race.replay.seal_key()),
        ),
        listed!("sound.owner.SOUND.race.shared", &o.sound.threads.shared,),
        listed!(
            "sound.owner.SOUND.race.sounds",
            on_game(o.sound.race.sounds.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.clock",
            on_game(o.sound.race.clock.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.surface",
            on_game(o.sound.race.surface.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.stage",
            on_game(o.sound.race.stage.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.spot_picks",
            on_game(o.sound.race.spot_picks.get()),
        ),
        listed!(
            "sound.owner.SOUND.menu.settings_applied",
            on_game(o.sound.menu.settings_applied.get()),
        ),
        listed!("input.owner.INPUT.force_feedback", &o.input.force_feedback,),
        listed!("sound.owner.SOUND.menu.options", &o.sound.menu.options,),
        listed!(
            "sound.owner.SOUND.menu.cd_audio",
            on_game(o.sound.menu.cd_audio.get()),
        ),
        listed!("input.owner.INPUT.devices", &o.input.devices,),
        listed!("input.owner.INPUT.keyboard", on_game(o.input.keyboard()),),
        listed!("input.owner.INPUT.mouse", on_game(o.input.mouse()),),
        listed!(
            "sound.owner.SOUND.menu.next_sound_time",
            on_game(o.sound.menu.next_sound_time.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.sounds_on",
            on_game(o.sound.race.sounds_on.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.car_stream",
            on_game(o.sound.race.car_stream.get()),
        ),
        listed!(
            "saves.owner.SAVES.savegame_preview",
            on_game(&o.saves.savegame_preview),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.controls",
            on_game(&o.menus.screens.controls),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.slots",
            on_game(&o.menus.play.slots),
        ),
        listed!(
            "menu.owner.MENU_OWNER.cars.table",
            on_game(&o.menus.cars.table),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.cheats",
            on_game(&o.menus.play.cheats),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.results",
            on_game(&o.menus.play.results),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.arcade",
            on_game(&o.menus.play.arcade),
        ),
        listed!(
            "menu.owner.MENU_OWNER.tables.captions",
            on_game(&o.menus.tables.captions),
        ),
        listed!(
            "menu.owner.MENU_OWNER.tables.pages",
            on_game(&o.menus.tables.pages),
        ),
        listed!(
            "menu.owner.MENU_OWNER.cars.numbers",
            on_game(&o.menus.cars.numbers),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.flow",
            on_game(&o.menus.screens.flow),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.loop_flags",
            on_game(&o.menus.screens.loop_flags),
        ),
        listed!(
            "menu.owner.MENU_OWNER.keys.tables",
            on_game(&o.menus.keys.tables),
        ),
        listed!("menu.owner.MENU_OWNER.options", on_game(&o.menus.options),),
        listed!(
            "menu.owner.MENU_OWNER.screens.entry",
            on_game(&o.menus.screens.entry),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.champ_view",
            on_game(&o.menus.play.champ_view),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.event",
            on_game(&o.menus.screens.event),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.draw_work",
            on_game(&o.menus.screens.draw_work),
        ),
        listed!(
            "system.owner.SYSTEM.records.stage_tables",
            on_game(&o.system.records().stage_tables),
        ),
        listed!(
            "menu.owner.MENU_OWNER.cars.drivers",
            on_game(&o.menus.cars.drivers),
        ),
        listed!(
            "saves.owner.SAVES.rally_times",
            on_game(&o.saves.rally_times),
        ),
        listed!("saves.owner.SAVES.best_times", on_game(&o.saves.best_times),),
        listed!(
            "menu.owner.MENU_OWNER.screens.items",
            on_game(&o.menus.screens.items),
        ),
        listed!(
            "menu.owner.MENU_OWNER.tables.menus.labels",
            on_game(&o.menus.tables.menus.labels),
        ),
        listed!(
            "race.owner.RACE.shared.tyre_change",
            on_game(o.race.shared.tyre_change()),
        ),
        listed!(
            "race.owner.RACE.replay.seal_work",
            on_game_from(o.race.replay.seal_work(), ".subkeys[0]"),
            0x98,
        ),
        listed!(
            "system.owner.SYSTEM.start.error_mode",
            on_game(&o.system.start().error_mode),
        ),
        listed!(
            "system.owner.SYSTEM.start.texts",
            on_game(&o.system.start().texts),
        ),
        listed!(
            "input.owner.INPUT.guids",
            record_from(&o.input.guids, ".device2"),
            0x1c0,
        ),
        listed!(
            "sound.owner.SOUND.menu.menu_sounds",
            record_from(&o.sound.threads.menu_sounds, ".list"),
            0x28,
        ),
        listed!("sound.owner.SOUND.race.sets", &o.sound.threads.sets, 0x9ac,),
        listed!(
            "sound.owner.SOUND.race.request",
            on_game(o.sound.race.request.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.cue_lists",
            on_game_from(o.sound.race.cue_lists.get(), ".table[0]"),
        ),
        listed!(
            "sound.owner.SOUND.race.kinds",
            on_game(o.sound.race.kinds.get()),
            0x13c8,
        ),
        listed!(
            "sound.owner.SOUND.race.surface_sets",
            on_game_from(o.sound.race.surface_sets.get(), ".sets[0]"),
            0x1c0,
        ),
        listed!(
            "sound.owner.SOUND.race.spot_voices",
            on_game(o.sound.race.spot_voices.get()),
        ),
        listed!(
            "install.owner.INSTALL.run",
            on_game_from(&o.install.run, ".same_version"),
        ),
        listed!(
            "install.owner.INSTALL.names",
            on_game(&o.install.names),
            0x8a7,
        ),
        listed!(
            "install.owner.INSTALL.pages",
            on_game_from(&o.install.pages, ".layout"),
            0x1a25,
        ),
        listed!(
            "install.owner.INSTALL.paths",
            on_game_from(&o.install.paths, ".game_dir"),
            0x600,
        ),
        listed!(
            "install.owner.INSTALL.palette",
            on_game_from(&o.install.palette, ".palette"),
            0x300,
        ),
        listed!(
            "system.owner.SYSTEM.start.run_count",
            on_game(&o.system.start().run_count),
        ),
        listed!("install.owner.INSTALL.sku", on_game(&o.install.sku),),
        listed!(
            "strings.owner.STRINGS.text.buffer",
            on_game(&o.strings.text.buffer),
        ),
        listed!(
            "system.owner.SYSTEM.work.files",
            on_game(&o.system.work().files),
        ),
        listed!("gfx.owner.GFX.display.mode", on_game(&o.gfx.display.mode),),
        listed!(
            "gfx.owner.GFX.drawing.draw_surface",
            on_game(&o.gfx.drawing.draw_surface),
        ),
        listed!("gfx.owner.GFX.d3d.cards", on_game(o.gfx.d3d.cards.get()),),
        listed!(
            "gfx.owner.GFX.fonts.font_block",
            on_game(&o.gfx.fonts.font_block),
        ),
        listed!(
            "audio.owner.AUDIO_OWNER.sample_names",
            &o.audio.sample_names,
        ),
        listed!(
            "gfx.owner.GFX.display.choices",
            on_game(&o.gfx.display.choices),
        ),
        listed!(
            "gfx.owner.GFX.fonts.jade",
            on_game(&o.gfx.fonts.jade),
            0x1419,
        ),
        listed!(
            "workers.PARTS[0].vertices",
            on_game(&crate::workers::game_scratch().vertices),
        ),
        listed!(
            "workers.PARTS[0].polygon",
            on_game(&crate::workers::game_scratch().polygon),
        ),
        listed!(
            "race.owner.RACE.shared.buffers.reads",
            on_game(o.race.shared.buffers.reads.get()),
        ),
        listed!(
            "race.owner.RACE.shared.viewer",
            on_game(o.race.shared.viewer_buffers()),
        ),
        listed!(
            "screens.text_buffers_store.TEXT_BUFFERS",
            on_game(crate::screens::text_buffers_store::text_buffers()),
        ),
        listed!(
            "race.viewer_state_store.RECTANGLES",
            &crate::race::viewer_state_store::RECTANGLES,
        ),
        listed!(
            "race.owner.RACE.replay.viewer_and_kinds",
            &o.race.replay.viewer_and_kinds,
        ),
        listed!(
            "race.owner.RACE.frame.lights_and_gear",
            on_game(o.race.frame.lights_and_gear.get()),
        ),
        listed!(
            "race.owner.RACE.shared.car_blocks",
            on_game(o.race.shared.car_blocks.get()),
        ),
        listed!(
            "race.owner.RACE.replay.replay_work",
            on_game(o.race.replay.replay_work()),
        ),
        listed!(
            "race.groups_store.PANEL_TAIL",
            on_game_from(o.race.frame.panel_and_overlay.get(), ".panel.split_6",),
        ),
        listed!(
            "race.owner.RACE.physics.split_tables",
            on_game(o.race.physics.split_tables()),
        ),
        listed!(
            "race.owner.RACE.course.league",
            on_game(o.race.course.league()),
        ),
        listed!(
            "race.owner.RACE.course.lights_data",
            on_game(o.race.course.lights_data()),
        ),
        listed!(
            "race.owner.RACE.course.level_paths",
            on_game(o.race.course.level_paths()),
        ),
        listed!(
            "race.owner.RACE.physics.pace",
            on_game(o.race.physics.pace()),
        ),
        listed!(
            "race.owner.RACE.physics.ground_probe",
            on_game(o.race.physics.ground_probe.get()),
        ),
        listed!(
            "race.owner.RACE.frame.panel_clock",
            on_game(o.race.frame.panel_clock()),
        ),
        listed!(
            "scene.owner.SCENE.objects.slots",
            on_game(o.scene.objects.slots.get()),
        ),
        listed!(
            "scene.owner.SCENE.light_tables",
            on_game(o.scene.light_tables.get()),
        ),
        listed!(
            "race.owner.RACE.frame.error_list",
            on_game(o.race.frame.error_list()),
        ),
        listed!(
            "race.owner.RACE.course.flare_tables",
            on_game(o.race.course.flare_tables()),
        ),
        listed!(
            "race.owner.RACE.frame.shadow_skids",
            on_game(o.race.frame.shadow_skids.get()),
        ),
        listed!(
            "race.owner.RACE.replay.finish",
            on_game(o.race.replay.finish.get()),
        ),
        listed!(
            "race.owner.RACE.shared.replay_file",
            on_game(o.race.shared.replay_file()),
        ),
        listed!(
            "race.owner.RACE.replay.replay_view",
            on_game(o.race.replay.replay_view()),
        ),
        listed!(
            "race.owner.RACE.course.numbers",
            on_game(o.race.course.numbers.get()),
        ),
        listed!(
            "race.owner.RACE.frame.scratch",
            on_game(o.race.frame.scratch()),
        ),
        listed!(
            "race.owner.RACE.course.lists",
            on_game(o.race.course.lists.get()),
        ),
        listed!(
            "race.owner.RACE.physics.car_specs",
            on_game(o.race.physics.car_specs()),
        ),
        listed!(
            "race.owner.RACE.course.course",
            on_game(o.race.course.course.get()),
        ),
        listed!(
            "scene.owner.SCENE.objects.tables",
            on_game(o.scene.objects.tables.get()),
        ),
        listed!(
            "scene.owner.SCENE.textures.places",
            on_game(o.scene.textures.places.get()),
        ),
        listed!(
            "race.owner.RACE.frame.matrices",
            on_game(o.race.frame.matrices.get()),
        ),
        listed!(
            "race.owner.RACE.shared.tables",
            on_game(o.race.shared.tables()),
        ),
        listed!(
            "race.owner.RACE.frame.view_and_records",
            on_game(o.race.frame.view_and_records.get()),
        ),
        listed!(
            "race.owner.RACE.physics.camera",
            on_game(o.race.physics.camera()),
        ),
        listed!(
            "race.owner.RACE.course.stage_prefs",
            on_game(o.race.course.stage_prefs.get()),
        ),
        listed!(
            "scene.owner.SCENE.file_numbers",
            on_game(o.scene.file_numbers()),
        ),
        listed!(
            "race.owner.RACE.shared.physics_tables",
            on_game(o.race.shared.physics_tables.get()),
        ),
        listed!(
            "race.owner.RACE.shared.vars",
            on_game(o.race.shared.vars.get()),
        ),
        listed!(
            "scene.owner.SCENE.drawing.scene",
            on_game(o.scene.drawing.scene.get()),
        ),
        listed!(
            "scene.owner.SCENE.lights.state",
            on_game(o.scene.lights.state.get()),
        ),
        listed!(
            "race.owner.RACE.shared.view",
            on_game(o.race.shared.view.get()),
        ),
        listed!(
            "race.owner.RACE.course.level_grid",
            on_game(o.race.course.level_grid.get()),
        ),
        listed!(
            "race.owner.RACE.frame.frame_work",
            on_game(o.race.frame.frame_work.get()),
        ),
        listed!(
            "race.owner.RACE.frame.sound_spots",
            on_game(o.race.frame.sound_spots()),
        ),
        listed!(
            "race.owner.RACE.physics.contact",
            on_game(o.race.physics.contact.get()),
        ),
        listed!(
            "race.owner.RACE.replay.replay_frames",
            on_game(o.race.replay.replay_frames()),
        ),
        listed!(
            "race.owner.RACE.replay.staging",
            on_game(o.race.replay.staging()),
        ),
        listed!("race.owner.RACE.shared.car", on_game(o.race.shared.car()),),
        listed!(
            "race.owner.RACE.shared.work",
            on_game(o.race.shared.work.get()),
        ),
        listed!(
            "race.owner.RACE.frame.text_state",
            on_game(o.race.frame.text_state()),
        ),
        listed!(
            "race.owner.RACE.frame.clock_ticks",
            on_game(o.race.frame.clock_ticks()),
        ),
        listed!(
            "race.owner.RACE.course.kind_records",
            on_game(o.race.course.kind_records()),
        ),
        listed!(
            "scene.owner.SCENE.textures.kinds",
            on_game(o.scene.textures.kinds.get()),
        ),
        listed!(
            "race.owner.RACE.frame.counter",
            on_game(o.race.frame.counter()),
        ),
        listed!(
            "race.owner.RACE.shared.switches",
            on_game(o.race.shared.switches.get()),
        ),
        listed!(
            "scene.owner.SCENE.draw_switches",
            on_game(o.scene.draw_switches.get()),
        ),
        listed!(
            "scene.owner.SCENE.lights.work",
            on_game(o.scene.lights.work.get()),
        ),
        listed!(
            "scene.owner.SCENE.drawing.object_work",
            on_game(o.scene.object_work()),
        ),
        listed!(
            "race.owner.RACE.physics.ground",
            on_game(o.race.physics.ground()),
        ),
        listed!(
            "race.owner.RACE.frame.draw_batches",
            on_game(o.race.frame.draw_batches()),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.work",
            on_game(&o.race.effects.get().setup.work),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.state",
            on_game(&o.menus.screens.state),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.carousel",
            on_game(&o.menus.screens.carousel),
        ),
        listed!("race.owner.RACE.shared.state", &o.race.shared.state,),
        listed!(
            "race.owner.RACE.physics.damage",
            on_game(o.race.physics.damage()),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.inputs",
            on_game(&o.race.effects.get().windscreen_drops.inputs),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.view_sources",
            on_game(&o.race.effects.get().frame.view_sources),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.clock",
            on_game(&o.race.effects.get().setup.clock),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.frame",
            on_game(&o.race.effects.get().setup.frame),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.floats",
            on_game(&o.race.effects.get().setup.floats),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.kind_floats",
            on_game(&o.race.effects.get().setup.kind_floats),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.settings",
            on_game(&o.race.effects.get().setup.settings),
        ),
        listed!(
            "effects.owner.EFFECTS.pong.state",
            on_game(&o.race.effects.get().pong.state),
        ),
        listed!(
            "effects.owner.EFFECTS.clipping.clip",
            on_game(&o.race.effects.get().clipping.clip),
        ),
        listed!(
            "effects.owner.EFFECTS.pong.rules",
            on_game(&o.race.effects.get().pong.rules),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.marks_kind",
            on_game(&o.race.effects.get().kinds.marks_kind),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.kept_view",
            on_game(&o.race.effects.get().frame.kept_view),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_head",
            on_game(&o.race.effects.get().paths.spline_head),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_work",
            on_game(&o.race.effects.get().paths.spline_work),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_tables",
            on_game(&o.race.effects.get().paths.spline_tables),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_kinds",
            on_game(&o.race.effects.get().paths.spline_kinds),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_path",
            on_game(&o.race.effects.get().paths.spline_path),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_constants",
            on_game(&o.race.effects.get().paths.spline_constants),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.edge_mask",
            on_game(&o.race.effects.get().masks.edge_mask),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.tilt",
            on_game(&o.race.effects.get().windscreen_drops.tilt),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.kind_boxes",
            on_game(&o.race.effects.get().masks.kind_boxes),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.state",
            on_game(&o.race.effects.get().windscreen_drops.state),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.work",
            on_game(&o.race.effects.get().windscreen_drops.work),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.hex",
            on_game(&o.race.effects.get().windscreen_drops.hex),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.vars",
            on_game(&o.race.effects.get().masks.vars),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.picks",
            on_game(&o.race.effects.get().windscreen_drops.picks),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.shape_work",
            on_game(&o.race.effects.get().windscreen_drops.shape_work,),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.tuning",
            on_game(&o.race.effects.get().windscreen_drops.tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.mask_constants",
            on_game(&o.race.effects.get().setup.mask_constants),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.timing",
            on_game(&o.race.effects.get().windscreen_flakes.timing,),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.tables",
            on_game(&o.race.effects.get().windscreen_flakes.tables,),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.items",
            on_game(&o.race.effects.get().windscreen_flakes.items),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.state",
            on_game(&o.race.effects.get().windscreen_flakes.state),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.edge_settings",
            on_game(&o.race.effects.get().masks.edge_settings),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.rain",
            on_game(&o.race.effects.get().weather.rain),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_tables",
            on_game(&o.race.effects.get().weather.snow_tables),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow",
            on_game(&o.race.effects.get().weather.snow),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_curve",
            on_game(&o.race.effects.get().weather.snow_curve),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_vars",
            on_game(&o.race.effects.get().weather.snow_vars),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_light",
            on_game(&o.race.effects.get().weather.snow_light),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.places",
            on_game(&o.race.effects.get().blocks.places),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.profile_rings",
            on_game(&o.race.effects.get().paths.profile_rings),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.profile_tables",
            on_game(&o.race.effects.get().paths.profile_tables),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.draw_work",
            on_game(&o.race.effects.get().frame.draw_work),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.corners",
            on_game(&o.race.effects.get().blocks.corners),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.view_vars",
            on_game(&o.race.effects.get().frame.view_vars),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.draw_frame",
            on_game(&o.race.effects.get().frame.draw_frame),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.sines",
            on_game(&o.race.effects.get().paths.sines),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.flare_parts",
            on_game(&o.race.effects.get().frame.flare_parts),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.wave_levels",
            on_game(&o.race.effects.get().paths.wave_levels),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.wave",
            on_game(&o.race.effects.get().paths.wave),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.flare_scales",
            on_game(&o.race.effects.get().frame.flare_scales),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.links",
            on_game(&o.race.effects.get().blocks.links),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.corner_slopes",
            on_game(&o.race.effects.get().blocks.corner_slopes),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spread_random",
            on_game(&o.race.effects.get().paths.spread_random),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.overlay_quad",
            on_game(&o.race.effects.get().kinds.overlay_quad),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.overlay_extras",
            on_game(&o.race.effects.get().kinds.overlay_extras),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.shadow_quad",
            on_game(&o.race.effects.get().kinds.shadow_quad),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.shadow_extra",
            on_game(&o.race.effects.get().kinds.shadow_extra),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.plane_quad",
            on_game(&o.race.effects.get().kinds.plane_quad),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.ground_names",
            on_game(&o.race.effects.get().masks.ground_names),
            0xa6,
        ),
        listed!(
            "effects.owner.EFFECTS.masks.blocks",
            on_game(&o.race.effects.get().masks.blocks),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.picture_copy",
            on_game(&o.race.effects.get().masks.picture_copy),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.slots",
            on_game(&o.race.effects.get().blocks.slots),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.slot_tuning",
            on_game(&o.race.effects.get().blocks.slot_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.emitter",
            on_game(&o.race.effects.get().kinds.emitter),
        ),
        listed!(
            "effects.owner.EFFECTS.ring.work",
            on_game(&o.race.effects.get().ring.work),
        ),
        listed!(
            "effects.owner.EFFECTS.ring.names",
            on_game(&o.race.effects.get().ring.names),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.surface",
            on_game(&o.race.effects.get().masks.surface),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.twin_state",
            on_game(&o.race.effects.get().particles.twin_state),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind1_choice",
            on_game(&o.race.effects.get().particles.kind1_choice),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.list",
            on_game(&o.race.effects.get().masks.list),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind5_set",
            on_game(&o.race.effects.get().particles.kind5_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind11_set",
            on_game(&o.race.effects.get().particles.kind11_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind21_set",
            on_game(&o.race.effects.get().particles.kind21_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind1_set",
            on_game(&o.race.effects.get().particles.kind1_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.first_tuning",
            on_game(&o.race.effects.get().particles.first_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.second_tuning",
            on_game(&o.race.effects.get().particles.second_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind6_set",
            on_game(&o.race.effects.get().particles.kind6_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind13_set",
            on_game(&o.race.effects.get().particles.kind13_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.nineteenth_tuning",
            on_game(&o.race.effects.get().particles.nineteenth_tuning,),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.twenty_second_tuning",
            on_game(&o.race.effects.get().particles.twenty_second_tuning,),
        ),
        listed!(
            "effects.owner.EFFECTS.ring.pool",
            on_game(&o.race.effects.get().ring.pool),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.puff_work",
            on_game(&o.race.effects.get().particles.puff_work),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.burst_colours",
            on_game(&o.race.effects.get().kinds.burst_colours),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.particle_tuning",
            on_game(&o.race.effects.get().kinds.particle_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.clipping.blend_work",
            on_game(&o.race.effects.get().clipping.blend_work),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.morph_work",
            on_game(&o.race.effects.get().paths.morph_work),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.lines_work",
            on_game(&o.race.effects.get().paths.lines_work),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.clock_k",
            on_game(&o.race.effects.get().setup.clock_k),
        ),
        listed!(
            "effects.owner.EFFECTS.clipping.glow",
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
