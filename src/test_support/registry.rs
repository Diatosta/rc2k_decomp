//! Test-only: the registry of the game state's stores, each with a name, so tests can read
//! and put back the game's state by store name (`state::walk`, `state::Snapshot`) and field
//! path rather than by address. Nothing in production uses it.
//!
//! The list is the stores' own (`listed_stores`); a repeated name is found at its first entry.

use crate::cells::Cells;

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
fn listed_stores() -> Vec<Listed> {
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
            on_game(&crate::system::owner::SYSTEM.work().numbers),
        ),
        listed!(
            "logfile.start_log.FREE_SPACE_KB",
            on_game(crate::logfile::start_log::FREE_SPACE_KB.get()),
        ),
        listed!("math.next_random.STATE", &crate::math::next_random::STATE),
        listed!(
            "system.owner.SYSTEM.start.kept",
            on_game(&crate::system::owner::SYSTEM.start().kept),
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
            on_game(&crate::strings::owner::STRINGS.get().language.locale),
        ),
        listed!(
            "strings.string_table.LEXICON",
            on_game(crate::strings::string_table::LEXICON.get()),
        ),
        listed!("strings.same_text.LOWER", &crate::strings::same_text::LOWER),
        listed!(
            "strings.owner.STRINGS.language.plain_letters",
            on_game(&crate::strings::owner::STRINGS.get().language.plain_letters),
        ),
        listed!(
            "strings.owner.STRINGS.lexicon.empty_message",
            on_game(&crate::strings::owner::STRINGS.get().lexicon.empty_message),
        ),
        listed!(
            "strings.units.UNITS",
            on_game(crate::strings::units::UNITS.get())
        ),
        listed!(
            "strings.owner.STRINGS.lexicon.work",
            on_game(&crate::strings::owner::STRINGS.get().lexicon.work),
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
            on_game(&crate::strings::owner::STRINGS.get().drivers.key),
        ),
        listed!(
            "strings.owner.STRINGS.drivers.names",
            on_game(&crate::strings::owner::STRINGS.get().drivers.names),
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
            on_game(&crate::strings::owner::STRINGS.get().text.blend_mode),
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
            "hud.owner.HUD.block",
            on_game(&crate::hud::owner::HUD.get().block),
        ),
        listed!(
            "gfx.owner.GFX.pictures.decoder",
            on_game(&crate::gfx::owner::GFX.get().pictures.decoder),
        ),
        listed!(
            "strings.owner.STRINGS.lexicon.memory",
            on_game(&crate::strings::owner::STRINGS.get().lexicon.memory),
        ),
        listed!(
            "gfx.owner.GFX.sheet",
            on_game(&crate::gfx::owner::GFX.get().sheet),
        ),
        listed!(
            "gfx.owner.GFX.pictures.pcx",
            on_game(&crate::gfx::owner::GFX.get().pictures.pcx),
        ),
        listed!(
            "gfx.owner.GFX.png.setting",
            on_game(&crate::gfx::owner::GFX.get().png.setting),
        ),
        listed!(
            "gfx.owner.GFX.fonts.glyph_tables",
            on_game(&crate::gfx::owner::GFX.get().fonts.glyph_tables),
        ),
        listed!(
            "gfx.owner.GFX.pictures.showing",
            on_game(&crate::gfx::owner::GFX.get().pictures.showing),
        ),
        listed!(
            "gfx.owner.GFX.drawing.blit",
            on_game(&crate::gfx::owner::GFX.get().drawing.blit),
        ),
        listed!(
            "gfx.owner.GFX.rescale",
            on_game(&crate::gfx::owner::GFX.get().rescale),
        ),
        listed!(
            "gfx.owner.GFX.strips",
            on_game(&crate::gfx::owner::GFX.get().strips),
        ),
        listed!(
            "gfx.owner.GFX.drawing.row_table",
            on_game(&crate::gfx::owner::GFX.get().drawing.row_table),
        ),
        listed!(
            "gfx.owner.GFX.pixel.formats",
            on_game(&crate::gfx::owner::GFX.get().pixel.formats),
        ),
        listed!(
            "gfx.owner.GFX.drawing.thirds",
            on_game(&crate::gfx::owner::GFX.get().drawing.thirds),
        ),
        listed!(
            "gfx.owner.GFX.display.modes",
            on_game(&crate::gfx::owner::GFX.get().display.modes),
        ),
        listed!(
            "gfx.owner.GFX.pixel.converters",
            on_game(&crate::gfx::owner::GFX.get().pixel.converters),
        ),
        listed!(
            "gfx.owner.GFX.reports",
            on_game(&crate::gfx::owner::GFX.get().reports),
        ),
        listed!(
            "gfx.owner.GFX.shots",
            on_game(&crate::gfx::owner::GFX.get().shots),
        ),
        listed!(
            "gfx.owner.GFX.pictures.picture",
            on_game(&crate::gfx::owner::GFX.get().pictures.picture),
        ),
        listed!(
            "gfx.owner.GFX.grading",
            on_game(&crate::gfx::owner::GFX.get().grading),
        ),
        listed!(
            "gfx.owner.GFX.drawing.bar",
            on_game(&crate::gfx::owner::GFX.get().drawing.bar),
        ),
        listed!(
            "gfx.owner.GFX.png.file",
            on_game(&crate::gfx::owner::GFX.get().png.file),
        ),
        listed!(
            "gfx.owner.GFX.png.header",
            on_game(&crate::gfx::owner::GFX.get().png.header),
        ),
        listed!(
            "gfx.owner.GFX.pictures.planes",
            on_game(&crate::gfx::owner::GFX.get().pictures.planes),
        ),
        listed!(
            "gfx.owner.GFX.display.cards",
            on_game(&crate::gfx::owner::GFX.get().display.cards),
        ),
        listed!(
            "gfx.owner.GFX.card.upload",
            on_game(&crate::gfx::owner::GFX.get().card.upload),
        ),
        listed!(
            "install.owner.INSTALL.vars",
            on_game_from(&crate::install::owner::install().vars, "._gap_529dad"),
            0x4b6,
        ),
        listed!(
            "menu.owner.MENU_OWNER.panels.notice",
            on_game(&crate::menu::owner::MENU_OWNER.get().panels.notice),
        ),
        listed!(
            "install.owner.INSTALL.drive_rows",
            on_game(&crate::install::owner::install().drive_rows),
        ),
        listed!(
            "net.owner.NET_OWNER.vars",
            &crate::net::owner::NET_OWNER.vars,
        ),
        listed!(
            "net.owner.NET_OWNER.link",
            crate::net::owner::NET_OWNER.link.cells(),
        ),
        listed!(
            "net.owner.NET_OWNER.main.edit",
            on_game(&crate::net::owner::NET_OWNER.main.get().edit),
        ),
        listed!(
            "net.owner.NET_OWNER.main.setup",
            on_game(&crate::net::owner::NET_OWNER.main.get().setup),
        ),
        listed!(
            "net.owner.NET_OWNER.main.address",
            on_game(&crate::net::owner::NET_OWNER.main.get().address),
        ),
        listed!(
            "net.owner.NET_OWNER.main.joined",
            on_game(&crate::net::owner::NET_OWNER.main.get().joined),
        ),
        listed!(
            "net.owner.NET_OWNER.main.lobby",
            on_game(&crate::net::owner::NET_OWNER.main.get().lobby),
        ),
        listed!(
            "net.owner.NET_OWNER.button",
            on_game(crate::net::owner::NET_OWNER.button.get()),
        ),
        listed!(
            "gfx.owner.GFX.fonts.typefaces",
            on_game(&crate::gfx::owner::GFX.get().fonts.typefaces),
        ),
        listed!(
            "saves.owner.SAVES.save_games",
            on_game(&crate::saves::owner::SAVES.get().save_games),
        ),
        listed!(
            "service.owner.SERVICE.set_ups",
            on_game(&crate::service::owner::SERVICE.get().set_ups),
        ),
        listed!(
            "race.owner.RACE.shared.replays",
            on_game(crate::race::owner::RACE.shared.replays()),
        ),
        listed!(
            "install.owner.INSTALL.texts",
            &crate::install::owner::install().texts,
        ),
        listed!(
            "saves.owner.SAVES.files.options_path",
            on_game(&crate::saves::owner::SAVES.get().files.options_path),
        ),
        listed!(
            "saves.owner.SAVES.files.savegame_template",
            on_game(&crate::saves::owner::SAVES.get().files.savegame_template),
        ),
        listed!(
            "service.owner.SERVICE.template",
            on_game(&crate::service::owner::SERVICE.get().template),
        ),
        listed!(
            "race.owner.RACE.replay.snapshot",
            on_game(crate::race::owner::RACE.replay.snapshot()),
        ),
        listed!(
            "menu.owner.MENU_OWNER.panels.films",
            on_game(&crate::menu::owner::MENU_OWNER.get().panels.films),
        ),
        listed!(
            "screens.draw_message_box.BOX",
            on_game(crate::screens::draw_message_box::BOX.get()),
        ),
        listed!(
            "race.owner.RACE.frame.credits",
            on_game(crate::race::owner::RACE.frame.credits()),
        ),
        listed!(
            "system.check_developer_machine.DEVELOPER",
            on_game(crate::system::check_developer_machine::DEVELOPER.get()),
        ),
        listed!(
            "gfx.owner.GFX.card.texture_tables",
            &crate::gfx::owner::TEXTURE_TABLES,
        ),
        listed!("film.NO_FMV", on_game(crate::film::NO_FMV.get())),
        listed!(
            "gfx.owner.GFX.fonts.game_fonts_memory",
            on_game(&crate::gfx::owner::GFX.get().fonts.game_fonts_memory),
        ),
        listed!(
            "scene.owner.SCENE.icon_upload",
            on_game(crate::scene::owner::SCENE.icon_upload()),
        ),
        listed!(
            "race.owner.RACE.frame.map",
            on_game(crate::race::owner::RACE.frame.map()),
        ),
        listed!(
            "gfx.owner.GFX.drawing.surfaces",
            on_game(&crate::gfx::owner::GFX.get().drawing.surfaces),
        ),
        listed!(
            "gfx.owner.GFX.locked",
            on_game(&crate::gfx::owner::GFX.get().locked),
        ),
        listed!(
            "gfx.owner.GFX.card.windscreen_drop_batch",
            on_game(&crate::gfx::owner::GFX.get().card.windscreen_drop_batch),
        ),
        listed!(
            "system.owner.SYSTEM.session.screen",
            on_game(crate::system::owner::SYSTEM.session.screen.get()),
        ),
        listed!(
            "system.owner.SYSTEM.session.flags",
            &crate::system::owner::SYSTEM.session.flags,
        ),
        listed!(
            "system.owner.SYSTEM.session.host",
            &crate::system::owner::SYSTEM.session.host,
        ),
        listed!(
            "system.owner.SYSTEM.session.key_clock",
            &crate::system::owner::SYSTEM.session.key_clock,
        ),
        listed!(
            "system.owner.SYSTEM.window_loop",
            on_game(crate::system::owner::SYSTEM.window_loop()),
        ),
        listed!(
            "system.owner.SYSTEM.machine",
            on_game(crate::system::owner::SYSTEM.machine()),
        ),
        listed!(
            "system.owner.SYSTEM.clocks",
            on_game(crate::system::owner::SYSTEM.clocks()),
        ),
        listed!(
            "audio.owner.AUDIO_OWNER.state",
            &crate::audio::owner::AUDIO_OWNER.state,
        ),
        listed!(
            "audio.owner.AUDIO_OWNER.sample_records",
            &crate::audio::owner::AUDIO_OWNER.sample_records,
        ),
        listed!(
            "audio.owner.AUDIO_OWNER.environment",
            &crate::audio::owner::AUDIO_OWNER.environment,
        ),
        listed!(
            "saves.owner.SAVES.files.options_kept",
            on_game(&crate::saves::owner::SAVES.get().files.options_kept),
        ),
        listed!(
            "sound.store.WHEEL_SURFACES",
            on_game(crate::sound::store::WHEEL_SURFACES.get()),
        ),
        listed!(
            "race.owner.RACE.replay.seal_key",
            on_game(crate::race::owner::RACE.replay.seal_key()),
        ),
        listed!(
            "sound.owner.SOUND.race.shared",
            &crate::sound::owner::SOUND.threads.shared,
        ),
        listed!(
            "sound.owner.SOUND.race.sounds",
            on_game(crate::sound::owner::SOUND.race.sounds.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.clock",
            on_game(crate::sound::owner::SOUND.race.clock.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.surface",
            on_game(crate::sound::owner::SOUND.race.surface.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.stage",
            on_game(crate::sound::owner::SOUND.race.stage.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.spot_picks",
            on_game(crate::sound::owner::SOUND.race.spot_picks.get()),
        ),
        listed!(
            "sound.owner.SOUND.menu.settings_applied",
            on_game(crate::sound::owner::SOUND.menu.settings_applied.get()),
        ),
        listed!(
            "input.owner.INPUT.force_feedback",
            &crate::input::owner::INPUT.force_feedback,
        ),
        listed!(
            "sound.owner.SOUND.menu.options",
            &crate::sound::owner::SOUND.menu.options,
        ),
        listed!(
            "sound.owner.SOUND.menu.cd_audio",
            on_game(crate::sound::owner::SOUND.menu.cd_audio.get()),
        ),
        listed!(
            "input.owner.INPUT.devices",
            &crate::input::owner::INPUT.devices,
        ),
        listed!(
            "input.owner.INPUT.keyboard",
            on_game(crate::input::owner::INPUT.keyboard()),
        ),
        listed!(
            "input.owner.INPUT.mouse",
            on_game(crate::input::owner::INPUT.mouse()),
        ),
        listed!(
            "sound.owner.SOUND.menu.next_sound_time",
            on_game(crate::sound::owner::SOUND.menu.next_sound_time.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.sounds_on",
            on_game(crate::sound::owner::SOUND.race.sounds_on.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.car_stream",
            on_game(crate::sound::owner::SOUND.race.car_stream.get()),
        ),
        listed!(
            "saves.owner.SAVES.savegame_preview",
            on_game(&crate::saves::owner::SAVES.get().savegame_preview),
        ),
        listed!(
            "screens.owner.SCREENS_OWNER.controls",
            on_game(&crate::screens::owner::SCREENS_OWNER.get().controls),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.players",
            on_game(&crate::menu::owner::MENU_OWNER.get().play.players),
        ),
        listed!(
            "menu.owner.MENU_OWNER.cars.table",
            on_game(&crate::menu::owner::MENU_OWNER.get().cars.table),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.cheats",
            on_game(&crate::menu::owner::MENU_OWNER.get().play.cheats),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.results",
            on_game(&crate::menu::owner::MENU_OWNER.get().play.results),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.arcade",
            on_game(&crate::menu::owner::MENU_OWNER.get().play.arcade),
        ),
        listed!(
            "menu.owner.MENU_OWNER.tables.captions",
            on_game(&crate::menu::owner::MENU_OWNER.get().tables.captions),
        ),
        listed!(
            "menu.owner.MENU_OWNER.tables.pages",
            on_game(&crate::menu::owner::MENU_OWNER.get().tables.pages),
        ),
        listed!(
            "menu.owner.MENU_OWNER.cars.numbers",
            on_game(&crate::menu::owner::MENU_OWNER.get().cars.numbers),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.flow",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.flow),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.loop_flags",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.loop_flags),
        ),
        listed!(
            "menu.owner.MENU_OWNER.keys.tables",
            on_game(&crate::menu::owner::MENU_OWNER.get().keys.tables),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.option_work",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.option_work),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.champ_view",
            on_game(&crate::menu::owner::MENU_OWNER.get().play.champ_view),
        ),
        listed!(
            "menu.owner.MENU_OWNER.play.names",
            on_game(&crate::menu::owner::MENU_OWNER.get().play.names),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.event",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.event),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.draw_work",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.draw_work),
        ),
        listed!(
            "system.owner.SYSTEM.records.stage_tables",
            on_game(&crate::system::owner::SYSTEM.records().stage_tables),
        ),
        listed!(
            "menu.owner.MENU_OWNER.cars.drivers",
            on_game(&crate::menu::owner::MENU_OWNER.get().cars.drivers),
        ),
        listed!(
            "saves.owner.SAVES.rally_times",
            on_game(&crate::saves::owner::SAVES.get().rally_times),
        ),
        listed!(
            "saves.owner.SAVES.best_times",
            on_game(&crate::saves::owner::SAVES.get().best_times),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.items",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.items),
        ),
        listed!(
            "menu.owner.MENU_OWNER.tables.menus.labels",
            on_game(&crate::menu::owner::MENU_OWNER.get().tables.menus.labels),
        ),
        listed!(
            "race.owner.RACE.shared.tyre_change",
            on_game(crate::race::owner::RACE.shared.tyre_change()),
        ),
        listed!(
            "race.owner.RACE.replay.seal_work",
            on_game_from(crate::race::owner::RACE.replay.seal_work(), ".subkeys[0]"),
            0x98,
        ),
        listed!(
            "system.owner.SYSTEM.start.error_mode",
            on_game(&crate::system::owner::SYSTEM.start().error_mode),
        ),
        listed!(
            "system.owner.SYSTEM.start.texts",
            on_game(&crate::system::owner::SYSTEM.start().texts),
        ),
        listed!(
            "input.owner.INPUT.guids",
            record_from(&crate::input::owner::INPUT.guids, ".device2"),
            0x1c0,
        ),
        listed!(
            "sound.owner.SOUND.menu.menu_sounds",
            record_from(&crate::sound::owner::SOUND.threads.menu_sounds, ".list"),
            0x28,
        ),
        listed!(
            "sound.owner.SOUND.race.sets",
            &crate::sound::owner::SOUND.threads.sets,
            0x9ac,
        ),
        listed!(
            "sound.owner.SOUND.race.request",
            on_game(crate::sound::owner::SOUND.race.request.get()),
        ),
        listed!(
            "sound.owner.SOUND.race.cue_lists",
            on_game_from(crate::sound::owner::SOUND.race.cue_lists.get(), ".table[0]"),
        ),
        listed!(
            "sound.owner.SOUND.race.kinds",
            on_game(crate::sound::owner::SOUND.race.kinds.get()),
            0x13c8,
        ),
        listed!(
            "sound.owner.SOUND.race.surface_sets",
            on_game_from(
                crate::sound::owner::SOUND.race.surface_sets.get(),
                ".sets[0]"
            ),
            0x1c0,
        ),
        listed!(
            "sound.owner.SOUND.race.spot_voices",
            on_game(crate::sound::owner::SOUND.race.spot_voices.get()),
        ),
        listed!(
            "install.owner.INSTALL.run",
            on_game_from(&crate::install::owner::install().run, ".same_version"),
        ),
        listed!(
            "install.owner.INSTALL.names",
            on_game(&crate::install::owner::install().names),
            0x8a7,
        ),
        listed!(
            "install.owner.INSTALL.pages",
            on_game_from(&crate::install::owner::install().pages, ".layout"),
            0x1a25,
        ),
        listed!(
            "install.owner.INSTALL.paths",
            on_game_from(&crate::install::owner::install().paths, ".game_dir"),
            0x600,
        ),
        listed!(
            "install.owner.INSTALL.palette",
            on_game_from(&crate::install::owner::install().palette, ".palette"),
            0x300,
        ),
        listed!(
            "system.owner.SYSTEM.start.run_count",
            on_game(&crate::system::owner::SYSTEM.start().run_count),
        ),
        listed!(
            "install.owner.INSTALL.sku",
            on_game(&crate::install::owner::install().sku),
        ),
        listed!(
            "strings.owner.STRINGS.text.buffer",
            on_game(&crate::strings::owner::STRINGS.get().text.buffer),
        ),
        listed!(
            "system.owner.SYSTEM.work.files",
            on_game(&crate::system::owner::SYSTEM.work().files),
        ),
        listed!(
            "gfx.owner.GFX.display.mode",
            on_game(&crate::gfx::owner::GFX.get().display.mode),
        ),
        listed!(
            "gfx.owner.GFX.drawing.draw_surface",
            on_game(&crate::gfx::owner::DRAW_SURFACE),
        ),
        listed!(
            "d3d.owner.D3D.cards",
            on_game(crate::d3d::owner::D3D.cards.get()),
        ),
        listed!(
            "gfx.owner.GFX.fonts.font_block",
            on_game(&crate::gfx::owner::GFX.get().fonts.font_block),
        ),
        listed!(
            "audio.owner.AUDIO_OWNER.sample_names",
            &crate::audio::owner::AUDIO_OWNER.sample_names,
        ),
        listed!(
            "gfx.owner.GFX.display.choices",
            on_game(&crate::gfx::owner::GFX.get().display.choices),
        ),
        listed!(
            "gfx.owner.GFX.fonts.jade",
            on_game(&crate::gfx::owner::GFX.get().fonts.jade),
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
            on_game(crate::race::owner::RACE.shared.buffers.reads.get()),
        ),
        listed!(
            "race.owner.RACE.shared.viewer",
            on_game(crate::race::owner::RACE.shared.viewer_buffers()),
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
            &crate::race::owner::RACE.replay.viewer_and_kinds,
        ),
        listed!(
            "race.owner.RACE.frame.lights_and_gear",
            on_game(crate::race::owner::RACE.frame.lights_and_gear.get()),
        ),
        listed!(
            "race.owner.RACE.replay.car_blocks",
            on_game(crate::race::owner::RACE.replay.car_blocks.get()),
        ),
        listed!(
            "race.owner.RACE.replay.replay_work",
            on_game(crate::race::owner::RACE.replay.replay_work()),
        ),
        listed!(
            "race.groups_store.PANEL_TAIL",
            on_game_from(
                crate::race::owner::RACE.frame.panel_and_overlay.get(),
                ".panel.split_6",
            ),
        ),
        listed!(
            "race.owner.RACE.physics.split_tables",
            on_game(crate::race::owner::RACE.physics.split_tables()),
        ),
        listed!(
            "race.owner.RACE.course.league",
            on_game(crate::race::owner::RACE.course.league()),
        ),
        listed!(
            "race.owner.RACE.course.lights_data",
            on_game(crate::race::owner::RACE.course.lights_data()),
        ),
        listed!(
            "race.owner.RACE.course.level_paths",
            on_game(crate::race::owner::RACE.course.level_paths()),
        ),
        listed!(
            "race.owner.RACE.physics.pace",
            on_game(crate::race::owner::RACE.physics.pace()),
        ),
        listed!(
            "race.owner.RACE.physics.ground_probe",
            on_game(crate::race::owner::RACE.physics.ground_probe.get()),
        ),
        listed!(
            "race.owner.RACE.frame.panel_clock",
            on_game(crate::race::owner::RACE.frame.panel_clock()),
        ),
        listed!(
            "scene.owner.SCENE.objects.slots",
            on_game(crate::scene::owner::SCENE.objects.slots.get()),
        ),
        listed!(
            "scene.owner.SCENE.light_tables",
            on_game(crate::scene::owner::SCENE.light_tables.get()),
        ),
        listed!(
            "race.owner.RACE.frame.error_list",
            on_game(crate::race::owner::RACE.frame.error_list()),
        ),
        listed!(
            "race.owner.RACE.course.flare_tables",
            on_game(crate::race::owner::RACE.course.flare_tables()),
        ),
        listed!(
            "race.owner.RACE.frame.shadow_skids",
            on_game(crate::race::owner::RACE.frame.shadow_skids.get()),
        ),
        listed!(
            "race.owner.RACE.replay.finish",
            on_game(crate::race::owner::RACE.replay.finish.get()),
        ),
        listed!(
            "race.owner.RACE.shared.replay_file",
            on_game(crate::race::owner::RACE.shared.replay_file()),
        ),
        listed!(
            "race.owner.RACE.replay.replay_view",
            on_game(crate::race::owner::RACE.replay.replay_view()),
        ),
        listed!(
            "race.owner.RACE.course.numbers",
            on_game(crate::race::owner::RACE.course.numbers.get()),
        ),
        listed!(
            "race.owner.RACE.frame.scratch",
            on_game(crate::race::owner::RACE.frame.scratch()),
        ),
        listed!(
            "race.owner.RACE.course.lists",
            on_game(crate::race::owner::RACE.course.lists.get()),
        ),
        listed!(
            "race.owner.RACE.physics.car_specs",
            on_game(crate::race::owner::RACE.physics.car_specs()),
        ),
        listed!(
            "race.owner.RACE.course.course",
            on_game(crate::race::owner::RACE.course.course.get()),
        ),
        listed!(
            "scene.owner.SCENE.objects.tables",
            on_game(crate::scene::owner::SCENE.objects.tables.get()),
        ),
        listed!(
            "scene.owner.SCENE.textures.places",
            on_game(crate::scene::owner::SCENE.textures.places.get()),
        ),
        listed!(
            "race.owner.RACE.frame.matrices",
            on_game(crate::race::owner::RACE.frame.matrices.get()),
        ),
        listed!(
            "race.owner.RACE.shared.tables",
            on_game(crate::race::owner::RACE.shared.tables()),
        ),
        listed!(
            "race.owner.RACE.frame.view_and_records",
            on_game(crate::race::owner::RACE.frame.view_and_records.get()),
        ),
        listed!(
            "race.owner.RACE.physics.camera",
            on_game(crate::race::owner::RACE.physics.camera()),
        ),
        listed!(
            "race.owner.RACE.course.stage_prefs",
            on_game(crate::race::owner::RACE.course.stage_prefs.get()),
        ),
        listed!(
            "scene.owner.SCENE.file_numbers",
            on_game(crate::scene::owner::SCENE.file_numbers()),
        ),
        listed!(
            "race.owner.RACE.shared.physics_tables",
            on_game(crate::race::owner::RACE.shared.physics_tables.get()),
        ),
        listed!(
            "race.owner.RACE.shared.vars",
            on_game(crate::race::owner::RACE.shared.vars.get()),
        ),
        listed!(
            "scene.owner.SCENE.drawing.scene",
            on_game(crate::scene::owner::SCENE.drawing.scene.get()),
        ),
        listed!(
            "scene.owner.SCENE.lights.state",
            on_game(crate::scene::owner::SCENE.lights.state.get()),
        ),
        listed!(
            "race.owner.RACE.shared.view",
            on_game(crate::race::owner::RACE.shared.view.get()),
        ),
        listed!(
            "race.owner.RACE.course.level_grid",
            on_game(crate::race::owner::RACE.course.level_grid.get()),
        ),
        listed!(
            "race.owner.RACE.frame.frame_work",
            on_game(crate::race::owner::RACE.frame.frame_work.get()),
        ),
        listed!(
            "race.owner.RACE.frame.sound_spots",
            on_game(crate::race::owner::RACE.frame.sound_spots()),
        ),
        listed!(
            "race.owner.RACE.physics.contact",
            on_game(crate::race::owner::RACE.physics.contact.get()),
        ),
        listed!(
            "race.owner.RACE.replay.replay_frames",
            on_game(crate::race::owner::RACE.replay.replay_frames()),
        ),
        listed!(
            "race.owner.RACE.replay.staging",
            on_game(crate::race::owner::RACE.replay.staging()),
        ),
        listed!(
            "race.owner.RACE.shared.car",
            on_game(crate::race::owner::RACE.shared.car()),
        ),
        listed!(
            "race.owner.RACE.shared.work",
            on_game(crate::race::owner::RACE.shared.work.get()),
        ),
        listed!(
            "race.owner.RACE.frame.text_state",
            on_game(crate::race::owner::RACE.frame.text_state()),
        ),
        listed!(
            "race.owner.RACE.frame.clock_ticks",
            on_game(crate::race::owner::RACE.frame.clock_ticks()),
        ),
        listed!(
            "race.owner.RACE.course.kind_records",
            on_game(crate::race::owner::RACE.course.kind_records()),
        ),
        listed!(
            "scene.owner.SCENE.textures.kinds",
            on_game(crate::scene::owner::SCENE.textures.kinds.get()),
        ),
        listed!(
            "race.owner.RACE.frame.counter",
            on_game(crate::race::owner::RACE.frame.counter()),
        ),
        listed!(
            "race.owner.RACE.shared.switches",
            on_game(crate::race::owner::RACE.shared.switches.get()),
        ),
        listed!(
            "scene.owner.SCENE.draw_switches",
            on_game(crate::scene::owner::SCENE.draw_switches.get()),
        ),
        listed!(
            "scene.owner.SCENE.lights.work",
            on_game(crate::scene::owner::SCENE.lights.work.get()),
        ),
        listed!(
            "scene.owner.SCENE.drawing.object_work",
            on_game(crate::scene::owner::SCENE.object_work()),
        ),
        listed!(
            "race.owner.RACE.physics.ground",
            on_game(crate::race::owner::RACE.physics.ground()),
        ),
        listed!(
            "race.owner.RACE.frame.draw_batches",
            on_game(crate::race::owner::RACE.frame.draw_batches()),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.work",
            on_game(&crate::effects::owner::EFFECTS.get().setup.work),
        ),
        listed!(
            "menu.owner.MENU_OWNER.screens.state",
            on_game(&crate::menu::owner::MENU_OWNER.get().screens.state),
        ),
        listed!(
            "race.owner.RACE.shared.state",
            &crate::race::owner::RACE.shared.state,
        ),
        listed!(
            "race.owner.RACE.physics.damage",
            on_game(crate::race::owner::RACE.physics.damage()),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.inputs",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.inputs),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.view_sources",
            on_game(&crate::effects::owner::EFFECTS.get().frame.view_sources),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.clock",
            on_game(&crate::effects::owner::EFFECTS.get().setup.clock),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.frame",
            on_game(&crate::effects::owner::EFFECTS.get().setup.frame),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.floats",
            on_game(&crate::effects::owner::EFFECTS.get().setup.floats),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.kind_floats",
            on_game(&crate::effects::owner::EFFECTS.get().setup.kind_floats),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.settings",
            on_game(&crate::effects::owner::EFFECTS.get().setup.settings),
        ),
        listed!(
            "effects.owner.EFFECTS.pong.state",
            on_game(&crate::effects::owner::EFFECTS.get().pong.state),
        ),
        listed!(
            "effects.owner.EFFECTS.clipping.clip",
            on_game(&crate::effects::owner::EFFECTS.get().clipping.clip),
        ),
        listed!(
            "effects.owner.EFFECTS.pong.rules",
            on_game(&crate::effects::owner::EFFECTS.get().pong.rules),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.marks_kind",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.marks_kind),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.kept_view",
            on_game(&crate::effects::owner::EFFECTS.get().frame.kept_view),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_head",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spline_head),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_work",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spline_work),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_tables",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spline_tables),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_kinds",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spline_kinds),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_path",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spline_path),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spline_constants",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spline_constants),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.edge_mask",
            on_game(&crate::effects::owner::EFFECTS.get().masks.edge_mask),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.tilt",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.tilt),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.kind_boxes",
            on_game(&crate::effects::owner::EFFECTS.get().masks.kind_boxes),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.state",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.state),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.work",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.work),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.hex",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.hex),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.vars",
            on_game(&crate::effects::owner::EFFECTS.get().masks.vars),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.picks",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.picks),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.shape_work",
            on_game(
                &crate::effects::owner::EFFECTS
                    .get()
                    .windscreen_drops
                    .shape_work,
            ),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_drops.tuning",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_drops.tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.mask_constants",
            on_game(&crate::effects::owner::EFFECTS.get().setup.mask_constants),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.timing",
            on_game(
                &crate::effects::owner::EFFECTS
                    .get()
                    .windscreen_flakes
                    .timing,
            ),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.tables",
            on_game(
                &crate::effects::owner::EFFECTS
                    .get()
                    .windscreen_flakes
                    .tables,
            ),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.items",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_flakes.items),
        ),
        listed!(
            "effects.owner.EFFECTS.windscreen_flakes.state",
            on_game(&crate::effects::owner::EFFECTS.get().windscreen_flakes.state),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.edge_settings",
            on_game(&crate::effects::owner::EFFECTS.get().masks.edge_settings),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.rain",
            on_game(&crate::effects::owner::EFFECTS.get().weather.rain),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_tables",
            on_game(&crate::effects::owner::EFFECTS.get().weather.snow_tables),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow",
            on_game(&crate::effects::owner::EFFECTS.get().weather.snow),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_curve",
            on_game(&crate::effects::owner::EFFECTS.get().weather.snow_curve),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_vars",
            on_game(&crate::effects::owner::EFFECTS.get().weather.snow_vars),
        ),
        listed!(
            "effects.owner.EFFECTS.weather.snow_light",
            on_game(&crate::effects::owner::EFFECTS.get().weather.snow_light),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.places",
            on_game(&crate::effects::owner::EFFECTS.get().blocks.places),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.profile_rings",
            on_game(&crate::effects::owner::EFFECTS.get().paths.profile_rings),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.profile_tables",
            on_game(&crate::effects::owner::EFFECTS.get().paths.profile_tables),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.draw_work",
            on_game(&crate::effects::owner::EFFECTS.get().frame.draw_work),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.corners",
            on_game(&crate::effects::owner::EFFECTS.get().blocks.corners),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.view_vars",
            on_game(&crate::effects::owner::EFFECTS.get().frame.view_vars),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.draw_frame",
            on_game(&crate::effects::owner::EFFECTS.get().frame.draw_frame),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.sines",
            on_game(&crate::effects::owner::EFFECTS.get().paths.sines),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.flare_parts",
            on_game(&crate::effects::owner::EFFECTS.get().frame.flare_parts),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.wave_levels",
            on_game(&crate::effects::owner::EFFECTS.get().paths.wave_levels),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.wave",
            on_game(&crate::effects::owner::EFFECTS.get().paths.wave),
        ),
        listed!(
            "effects.owner.EFFECTS.frame.flare_scales",
            on_game(&crate::effects::owner::EFFECTS.get().frame.flare_scales),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.links",
            on_game(&crate::effects::owner::EFFECTS.get().blocks.links),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.corner_slopes",
            on_game(&crate::effects::owner::EFFECTS.get().blocks.corner_slopes),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.spread_random",
            on_game(&crate::effects::owner::EFFECTS.get().paths.spread_random),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.overlay_quad",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.overlay_quad),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.overlay_extras",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.overlay_extras),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.shadow_quad",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.shadow_quad),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.shadow_extra",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.shadow_extra),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.plane_quad",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.plane_quad),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.ground_names",
            on_game(&crate::effects::owner::EFFECTS.get().masks.ground_names),
            0xa6,
        ),
        listed!(
            "effects.owner.EFFECTS.masks.blocks",
            on_game(&crate::effects::owner::EFFECTS.get().masks.blocks),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.picture_copy",
            on_game(&crate::effects::owner::EFFECTS.get().masks.picture_copy),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.slots",
            on_game(&crate::effects::owner::EFFECTS.get().blocks.slots),
        ),
        listed!(
            "effects.owner.EFFECTS.blocks.slot_tuning",
            on_game(&crate::effects::owner::EFFECTS.get().blocks.slot_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.emitter",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.emitter),
        ),
        listed!(
            "effects.owner.EFFECTS.ring.work",
            on_game(&crate::effects::owner::EFFECTS.get().ring.work),
        ),
        listed!(
            "effects.owner.EFFECTS.ring.names",
            on_game(&crate::effects::owner::EFFECTS.get().ring.names),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.surface",
            on_game(&crate::effects::owner::EFFECTS.get().masks.surface),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.twin_state",
            on_game(&crate::effects::owner::EFFECTS.get().particles.twin_state),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind1_choice",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind1_choice),
        ),
        listed!(
            "effects.owner.EFFECTS.masks.list",
            on_game(&crate::effects::owner::EFFECTS.get().masks.list),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind5_set",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind5_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind11_set",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind11_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind21_set",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind21_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind1_set",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind1_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.first_tuning",
            on_game(&crate::effects::owner::EFFECTS.get().particles.first_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.second_tuning",
            on_game(&crate::effects::owner::EFFECTS.get().particles.second_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind6_set",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind6_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.kind13_set",
            on_game(&crate::effects::owner::EFFECTS.get().particles.kind13_set),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.nineteenth_tuning",
            on_game(
                &crate::effects::owner::EFFECTS
                    .get()
                    .particles
                    .nineteenth_tuning,
            ),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.twenty_second_tuning",
            on_game(
                &crate::effects::owner::EFFECTS
                    .get()
                    .particles
                    .twenty_second_tuning,
            ),
        ),
        listed!(
            "effects.owner.EFFECTS.ring.pool",
            on_game(&crate::effects::owner::EFFECTS.get().ring.pool),
        ),
        listed!(
            "effects.owner.EFFECTS.particles.puff_work",
            on_game(&crate::effects::owner::EFFECTS.get().particles.puff_work),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.burst_colours",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.burst_colours),
        ),
        listed!(
            "effects.owner.EFFECTS.kinds.particle_tuning",
            on_game(&crate::effects::owner::EFFECTS.get().kinds.particle_tuning),
        ),
        listed!(
            "effects.owner.EFFECTS.clipping.blend_work",
            on_game(&crate::effects::owner::EFFECTS.get().clipping.blend_work),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.morph_work",
            on_game(&crate::effects::owner::EFFECTS.get().paths.morph_work),
        ),
        listed!(
            "effects.owner.EFFECTS.paths.lines_work",
            on_game(&crate::effects::owner::EFFECTS.get().paths.lines_work),
        ),
        listed!(
            "effects.owner.EFFECTS.setup.clock_k",
            on_game(&crate::effects::owner::EFFECTS.get().setup.clock_k),
        ),
        listed!(
            "effects.owner.EFFECTS.clipping.glow",
            on_game(&crate::effects::owner::EFFECTS.get().clipping.glow),
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

/// The stores in the list's order (the golden file numbers a repeated name in this order).
pub fn listed() -> &'static [Entry] {
    static LIST: std::sync::OnceLock<Vec<Entry>> = std::sync::OnceLock::new();
    LIST.get_or_init(|| {
        listed_stores()
            .into_iter()
            .map(|l| Entry {
                name: l.name,
                store: l.store,
                len: l.len,
            })
            .collect()
    })
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
