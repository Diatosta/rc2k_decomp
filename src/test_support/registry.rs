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

/// A game-thread byte buffer (`PlainBuffer`) as a listed store: the path is always the whole
/// buffer (it has no fields and no cells to point at).
struct OnBuffer<const N: usize> {
    buffer: &'static crate::memory::plain_buffer::PlainBuffer<N>,
}

#[allow(
    unsafe_code,
    reason = "the registry's stores are shared as the owners are"
)]
// SAFETY: as `GameThread`: the data is reached only after the game-thread check.
unsafe impl<const N: usize> Sync for OnBuffer<N> {}

impl<const N: usize> Walk for OnBuffer<N> {
    fn path_bytes(&self, path: &str) -> Option<Vec<u8>> {
        crate::memory::session::assert_game_thread();
        path.is_empty().then(|| self.buffer.read().to_vec())
    }

    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::memory::session::assert_game_thread();
        let Ok(all) = <&[u8; N]>::try_from(bytes) else {
            return false;
        };
        path.is_empty() && {
            self.buffer.write(all);
            true
        }
    }

    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8> {
        crate::memory::session::assert_game_thread();
        if !path.is_empty() {
            return Vec::new();
        }
        self.buffer.read()[..length.min(N)].to_vec()
    }

    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::memory::session::assert_game_thread();
        path.is_empty() && self.buffer.write_at(0, bytes, "a registry store").is_ok()
    }

    fn first_addr(&self, _path: &str) -> Option<usize> {
        None
    }

    fn keep(&self) -> Box<dyn Fn() + '_> {
        crate::memory::session::assert_game_thread();
        let copy = self.buffer.read();
        Box::new(move || self.buffer.write(&copy))
    }
}

/// A game-thread memory list (`PlainList`) as a listed store: the path is always the whole
/// list, its four dwords (head, total, cursor, remaining) little-endian.
struct OnList {
    list: &'static crate::memory::PlainList,
}

#[allow(
    unsafe_code,
    reason = "the registry's stores are shared as the owners are"
)]
// SAFETY: as `GameThread`: the data is reached only after the game-thread check.
unsafe impl Sync for OnList {}

impl OnList {
    fn bytes(list: crate::memory::MemoryList) -> Vec<u8> {
        [list.head, list.total, list.cursor, list.remaining]
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }

    fn put(&self, bytes: &[u8]) -> bool {
        let Ok(all) = <&[u8; 16]>::try_from(bytes) else {
            return false;
        };
        let word = |n: usize| u32::from_le_bytes([all[n], all[n + 1], all[n + 2], all[n + 3]]);
        self.list.set(crate::memory::MemoryList {
            head: word(0),
            total: word(4),
            cursor: word(8),
            remaining: word(12),
        });
        true
    }
}

impl Walk for OnList {
    fn path_bytes(&self, path: &str) -> Option<Vec<u8>> {
        crate::memory::session::assert_game_thread();
        path.is_empty().then(|| Self::bytes(self.list.get()))
    }

    fn set_path_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::memory::session::assert_game_thread();
        path.is_empty() && self.put(bytes)
    }

    fn range_bytes(&self, path: &str, length: usize) -> Vec<u8> {
        crate::memory::session::assert_game_thread();
        if !path.is_empty() {
            return Vec::new();
        }
        let mut bytes = Self::bytes(self.list.get());
        bytes.truncate(length);
        bytes
    }

    fn set_range_bytes(&self, path: &str, bytes: &[u8]) -> bool {
        crate::memory::session::assert_game_thread();
        path.is_empty() && self.put(bytes)
    }

    fn first_addr(&self, _path: &str) -> Option<usize> {
        None
    }

    fn keep(&self) -> Box<dyn Fn() + '_> {
        crate::memory::session::assert_game_thread();
        let copy = self.list.get();
        Box::new(move || self.list.set(copy))
    }
}

/// A game-thread memory list as a listed store.
fn on_list(list: &'static crate::memory::PlainList) -> &'static OnList {
    Box::leak(Box::new(OnList { list }))
}

/// A game-thread byte buffer as a listed store.
fn on_buffer<const N: usize>(
    buffer: &'static crate::memory::plain_buffer::PlainBuffer<N>,
) -> &'static OnBuffer<N> {
    Box::leak(Box::new(OnBuffer { buffer }))
}

/// The whole of a game-thread owner's part as a listed store (lives for the program).
fn on_game<S: Cells>(whole: &'static S) -> &'static OnGame<S> {
    on_game_from(whole, "")
}

/// A game-thread owner's part from the field `first` on as a listed store.
fn on_game_from<S: Cells>(whole: &'static S, first: &'static str) -> &'static OnGame<S> {
    Box::leak(Box::new(OnGame { whole, first }))
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
            on_buffer(&o.system.work().numbers),
        ),
        listed!(
            "owners.race.shared.random.state",
            &o.race.shared.random.state
        ),
        listed!(
            "owners.strings.text.show_hours",
            on_game(&o.strings.text.show_hours),
        ),
        listed!(
            "owners.strings.text.buffer",
            on_buffer(&o.strings.text.buffer.text),
        ),
        listed!(
            "owners.strings.text.seal_text",
            on_buffer(&o.strings.text.seal_text),
        ),
        listed!(
            "owners.strings.lexicon.recompile_level",
            on_game(&o.strings.lexicon.recompile_level),
        ),
        listed!(
            "owners.strings.language.locale",
            on_buffer(&o.strings.language.locale),
        ),
        listed!(
            "owners.strings.lexicon.room",
            on_buffer(&o.strings.lexicon.room),
        ),
        listed!("strings.same_text.LOWER", &crate::strings::same_text::LOWER),
        listed!(
            "owners.strings.language.plain_letters",
            on_buffer(&o.strings.language.plain_letters),
        ),
        listed!(
            "owners.strings.lexicon.empty_message",
            on_buffer(&o.strings.lexicon.empty_message),
        ),
        listed!(
            "owners.strings.language.units",
            on_game(&o.strings.language.units)
        ),
        listed!(
            "owners.strings.lexicon.work",
            on_buffer(&o.strings.lexicon.work),
        ),
        listed!("memory.state.MOST_HELD", &crate::memory::state::MOST_HELD),
        listed!("memory.state.HELD", &crate::memory::state::HELD),
        listed!("memory.state.TURNOVER", &crate::memory::state::TURNOVER),
        listed!(
            "owners.strings.drivers.key",
            on_buffer(&o.strings.drivers.key),
        ),
        listed!(
            "owners.strings.drivers.names",
            on_buffer(&o.strings.drivers.names),
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
            "owners.strings.lexicon.kilobytes",
            on_game(&o.strings.lexicon.memory.kilobytes),
        ),
        listed!(
            "owners.strings.lexicon.list",
            on_list(&o.strings.lexicon.memory.memory),
        ),
        listed!("owners.gfx.png.setting", on_game(&o.gfx.png.setting),),
        listed!(
            "owners.gfx.pictures.showing",
            on_game(&o.gfx.pictures.showing),
        ),
        listed!(
            "owners.gfx.drawing.row_table",
            on_game(&o.gfx.drawing.row_table),
        ),
        listed!("owners.net.vars.receive_done", &o.net.vars.receive_done,),
        listed!("owners.net.vars.chat", &o.net.vars.chat,),
        listed!("owners.net.vars.chat_line", &o.net.vars.chat_line,),
        listed!("owners.net.vars.chat_text", &o.net.vars.chat_text,),
        listed!("owners.net.vars.event", &o.net.vars.event,),
        listed!("owners.net.vars.stage_item", &o.net.vars.stage_item,),
        listed!("owners.net.vars.all_raced", &o.net.vars.all_raced,),
        listed!("owners.net.vars.race_over", &o.net.vars.race_over,),
        listed!("owners.net.vars.sync_first", &o.net.vars.sync_first,),
        listed!("owners.net.vars.synced", &o.net.vars.synced,),
        listed!(
            "owners.net.main.settings.edit.edited",
            on_game(&o.net.main.get().settings.edit.edited),
        ),
        listed!(
            "owners.net.main.settings.edit.edit_width",
            on_game(&o.net.main.get().settings.edit.edit_width),
        ),
        listed!(
            "owners.net.main.settings.edit.text_before_edit",
            on_buffer(&o.net.main.get().settings.edit.text_before_edit),
        ),
        listed!(
            "owners.net.main.settings.most_players_text",
            on_buffer(&o.net.main.get().settings.most_players_text),
        ),
        listed!(
            "owners.net.main.settings.saved_name",
            on_buffer(&o.net.main.get().settings.saved_name),
        ),
        listed!(
            "owners.net.main.sessions.listed_at",
            on_game(&o.net.main.get().sessions.listed_at),
        ),
        listed!(
            "owners.net.main.setup.player_name",
            on_buffer(&o.net.main.get().setup.player_name),
        ),
        listed!(
            "owners.net.main.setup.machine_name",
            on_buffer(&o.net.main.get().setup.machine_name),
        ),
        listed!(
            "owners.net.main.setup.most_players",
            on_game(&o.net.main.get().setup.most_players),
        ),
        listed!(
            "owners.net.main.setup.session_user_values",
            on_buffer(&o.net.main.get().setup.session_user_values),
        ),
        listed!(
            "owners.net.main.setup.host_address",
            on_buffer(&o.net.main.get().setup.host_address),
        ),
        listed!(
            "owners.net.main.setup.port",
            on_game(&o.net.main.get().setup.port),
        ),
        listed!(
            "owners.net.main.setup.saved_connection",
            on_buffer(&o.net.main.get().setup.saved_connection),
        ),
        listed!(
            "owners.net.main.address.elements",
            on_game(&o.net.main.get().address.elements),
        ),
        listed!(
            "owners.net.main.address.address",
            on_game(&o.net.main.get().address.address),
        ),
        listed!(
            "owners.net.main.address.address_size",
            on_game(&o.net.main.get().address.address_size),
        ),
        listed!(
            "owners.net.main.joined.joined",
            on_buffer(&o.net.main.get().joined.joined),
        ),
        listed!(
            "owners.net.main.joined.joined_name",
            on_buffer(&o.net.main.get().joined.joined_name),
        ),
        listed!(
            "owners.net.main.joined.joined_most",
            on_game(&o.net.main.get().joined.joined_most),
        ),
        listed!(
            "owners.net.main.joined.joined_user_values",
            on_buffer(&o.net.main.get().joined.joined_user_values),
        ),
        listed!(
            "owners.net.main.lobby.edit.edited",
            on_game(&o.net.main.get().lobby.edit.edited),
        ),
        listed!(
            "owners.net.main.lobby.edit.edit_width",
            on_game(&o.net.main.get().lobby.edit.edit_width),
        ),
        listed!(
            "owners.net.main.lobby.edit.text_before_edit",
            on_buffer(&o.net.main.get().lobby.edit.text_before_edit),
        ),
        listed!(
            "owners.net.main.lobby.chat_input",
            on_buffer(&o.net.main.get().lobby.chat_input),
        ),
        listed!(
            "owners.net.main.lobby.listed_at",
            on_game(&o.net.main.get().lobby.listed_at),
        ),
        listed!(
            "owners.net.main.lobby.car_item",
            on_game(&o.net.main.get().lobby.car_item),
        ),
        listed!(
            "owners.net.main.lobby.mode",
            on_game(&o.net.main.get().lobby.mode),
        ),
        listed!(
            "owners.net.main.lobby.first_host",
            on_game(&o.net.main.get().lobby.first_host),
        ),
        listed!(
            "owners.net.main.lobby.send_buffer",
            on_buffer(&o.net.main.get().lobby.send_buffer),
        ),
        listed!(
            "owners.net.main.lobby.show_gaps",
            on_game(&o.net.main.get().lobby.show_gaps),
        ),
        listed!(
            "owners.net.main.lobby.data_size",
            on_game(&o.net.main.get().lobby.data_size),
        ),
        listed!(
            "owners.net.main.lobby.after_race",
            on_game(&o.net.main.get().lobby.after_race),
        ),
        listed!(
            "owners.net.main.lobby.racing",
            on_game(&o.net.main.get().lobby.racing),
        ),
        listed!(
            "owners.net.main.lobby.sync_clock",
            on_game(&o.net.main.get().lobby.sync_clock),
        ),
        listed!(
            "owners.net.main.lobby.sequence",
            on_game(&o.net.main.get().lobby.sequence),
        ),
        listed!(
            "owners.net.main.lobby.replay_kept",
            on_game(&o.net.main.get().lobby.replay_kept),
        ),
        listed!(
            "owners.net.main.lobby._pad_cells",
            on_buffer(&o.net.main.get().lobby._pad_cells),
        ),
        listed!("owners.net.button.text", on_game(&o.net.button.get().text),),
        listed!(
            "owners.net.button.flags",
            on_game(&o.net.button.get().flags),
        ),
        listed!("owners.net.button.x", on_game(&o.net.button.get().x),),
        listed!("owners.net.button.y", on_game(&o.net.button.get().y),),
        listed!(
            "owners.net.button.width",
            on_game(&o.net.button.get().width),
        ),
        listed!(
            "owners.net.button.tiles",
            on_game(&o.net.button.get().tiles),
        ),
        listed!("owners.net.button.item", on_game(&o.net.button.get().item),),
        listed!(
            "owners.saves.save_games.kept_a",
            on_game(&o.saves.save_games.kept_a),
        ),
        listed!(
            "owners.saves.save_games.file",
            on_buffer(&o.saves.save_games.file),
        ),
        listed!(
            "owners.saves.save_games.kept_b",
            on_game(&o.saves.save_games.kept_b),
        ),
        listed!(
            "owners.menus.service.set_ups.file",
            on_buffer(&o.menus.service.set_ups.file),
        ),
        listed!(
            "owners.menus.service.set_ups.names",
            on_buffer(&o.menus.service.set_ups.names),
        ),
        listed!(
            "owners.menus.service.set_ups.kept",
            on_game(&o.menus.service.set_ups.kept),
        ),
        listed!(
            "owners.saves.files.options_path",
            on_buffer(&o.saves.files.options_path),
        ),
        listed!(
            "owners.saves.files.savegame_template",
            on_buffer(&o.saves.files.savegame_template),
        ),
        listed!(
            "owners.menus.service.template",
            on_game(&o.menus.service.template),
        ),
        listed!(
            "owners.race.replay.snapshot",
            on_buffer(o.race.replay.snapshot()),
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
        listed!("owners.system.window_loop", on_game(o.system.window_loop()),),
        listed!("owners.system.machine", on_game(o.system.machine()),),
        listed!("owners.system.clocks", on_game(o.system.clocks()),),
        listed!(
            "owners.saves.files.options_kept",
            on_game(&o.saves.files.options_kept),
        ),
        listed!(
            "owners.input.keyboard.silent_keys",
            on_game(&o.input.keyboard().silent_keys),
        ),
        listed!(
            "owners.input.keyboard.compose",
            on_buffer(&o.input.keyboard().compose),
        ),
        listed!(
            "owners.input.keyboard.plain",
            on_buffer(&o.input.keyboard().plain),
        ),
        listed!(
            "owners.input.keyboard.shifted",
            on_buffer(&o.input.keyboard().shifted),
        ),
        listed!(
            "owners.input.keyboard.key_names",
            on_buffer(&o.input.keyboard().key_names),
        ),
        listed!(
            "owners.input.keyboard.unused",
            on_game(&o.input.keyboard()._unused_826c),
        ),
        listed!(
            "owners.sound.menu.next_sound_time",
            on_game(o.sound.menu.next_sound_time.get()),
        ),
        listed!(
            "owners.sound.race.sounds_on",
            on_game(o.sound.race.sounds_on.get()),
        ),
        listed!(
            "owners.menus.play.slots.pointers",
            on_game(&o.menus.play.slots.pointers),
        ),
        listed!(
            "owners.saves.best_times.list",
            on_list(&o.saves.best_times.memory),
        ),
        listed!(
            "owners.saves.best_times.allocated",
            on_game(&o.saves.best_times.allocated),
        ),
        listed!(
            "owners.saves.best_times.loaded",
            on_game(&o.saves.best_times.loaded),
        ),
        listed!(
            "owners.system.start.texts.version",
            on_buffer(&o.system.start().texts.version),
        ),
        listed!(
            "owners.system.start.texts.copyright",
            on_buffer(&o.system.start().texts.copyright),
        ),
        listed!(
            "owners.system.work.files",
            on_buffer(&o.system.work().files.buffer),
        ),
        listed!(
            "race.viewer_state_store.RECTANGLES",
            &crate::race::viewer_state_store::RECTANGLES,
        ),
        listed!(
            "owners.race.replay.marks_settings",
            on_game(o.race.replay.viewer_and_kinds.marks_settings.get()),
        ),
        listed!(
            "owners.race.replay.viewer_panel.untyped_tail",
            on_game(
                &o.race
                    .replay
                    .viewer_and_kinds
                    .viewer_panel
                    .get()
                    .untyped_tail
            ),
        ),
        listed!(
            "owners.race.replay.viewer_panel.texel_scale",
            on_game(
                &o.race
                    .replay
                    .viewer_and_kinds
                    .viewer_panel
                    .get()
                    .texel_scale
            ),
        ),
        listed!(
            "owners.race.replay.viewer_panel.reciprocal_width_800",
            on_game(
                &o.race
                    .replay
                    .viewer_and_kinds
                    .viewer_panel
                    .get()
                    .reciprocal_width_800
            ),
        ),
        listed!(
            "owners.race.replay.viewer_panel.reciprocal_height_600",
            on_game(
                &o.race
                    .replay
                    .viewer_and_kinds
                    .viewer_panel
                    .get()
                    .reciprocal_height_600
            ),
        ),
        listed!(
            "owners.race.shared.car_blocks",
            on_game(o.race.shared.car_blocks.get()),
        ),
        listed!(
            "owners.race.shared.replay_file.replays_on",
            on_game(&o.race.shared.replay_file().replays_on),
        ),
        listed!(
            "owners.race.shared.replay_file.no_replay_switch",
            on_game(&o.race.shared.replay_file().no_replay_switch),
        ),
        listed!(
            "owners.race.shared.replay_file.chunks_plain",
            on_game(&o.race.shared.replay_file().chunks_plain),
        ),
        listed!(
            "owners.race.shared.replay_file.replay_closed",
            on_game(&o.race.shared.replay_file().replay_closed),
        ),
        listed!(
            "owners.race.shared.replay_file.replay_open",
            on_game(&o.race.shared.replay_file().replay_open),
        ),
        listed!(
            "owners.race.shared.replay_file.replay_handle",
            on_game(&o.race.shared.replay_file().replay_handle),
        ),
        listed!(
            "owners.race.shared.replay_file.replay_at_end",
            on_game(&o.race.shared.replay_file().replay_at_end),
        ),
        listed!(
            "owners.race.shared.replay_file.hold_car_revs",
            on_game(&o.race.shared.replay_file().hold_car_revs),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_stopped",
            on_game(&o.race.shared.replay_file().viewer_stopped),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_mode",
            on_game(&o.race.shared.replay_file().viewer_mode),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_slot",
            on_game(&o.race.shared.replay_file().viewer_slot),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_sound",
            on_game(&o.race.shared.replay_file().viewer_sound),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_editing",
            on_game(&o.race.shared.replay_file().viewer_editing),
        ),
        listed!(
            "owners.race.shared.replay_file.season_code",
            on_game(&o.race.shared.replay_file().season_code),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_first_texture_a",
            on_game(&o.race.shared.replay_file().viewer_first_texture_a),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_list_texture",
            on_game(&o.race.shared.replay_file().viewer_list_texture),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_panel_texture",
            on_game(&o.race.shared.replay_file().viewer_panel_texture),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_device_texture",
            on_game(&o.race.shared.replay_file().viewer_device_texture),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_stage_loaded",
            on_game(&o.race.shared.replay_file().viewer_stage_loaded),
        ),
        listed!(
            "owners.race.shared.replay_file.viewer_fonts_built",
            on_game(&o.race.shared.replay_file().viewer_fonts_built),
        ),
        listed!(
            "owners.race.shared.replay_file.index.magic",
            on_game(&o.race.shared.replay_file().index.magic),
        ),
        listed!(
            "owners.race.shared.replay_file.index.seal",
            on_game(&o.race.shared.replay_file().index.seal),
        ),
        listed!(
            "owners.race.shared.replay_file.index.block_count",
            on_game(&o.race.shared.replay_file().index.block_count),
        ),
        listed!(
            "owners.race.shared.replay_file.index.frame_count",
            on_game(&o.race.shared.replay_file().index.frame_count),
        ),
        listed!(
            "owners.race.shared.replay_file.index.block_places",
            on_game(&o.race.shared.replay_file().index.block_places),
        ),
        listed!(
            "owners.race.course.grip_change",
            on_game(o.race.course.grip_change()),
        ),
        listed!(
            "owners.race.shared.vars.mud_scale_a",
            on_game(&o.race.shared.vars.get().mud_scale_a),
        ),
        listed!("owners.race.shared.car", on_game(o.race.shared.car()),),
        listed!("owners.race.shared.state", &o.race.shared.state,),
        listed!(
            "owners.effects.masks.edge_settings.factor",
            on_game(&o.race.effects.get().masks.edge_settings.factor),
        ),
        listed!(
            "owners.effects.masks.edge_settings.done",
            on_game(&o.race.effects.get().masks.edge_settings.done),
        ),
        listed!(
            "owners.effects.masks.edge_settings.scale",
            on_game(&o.race.effects.get().masks.edge_settings.scale),
        ),
        listed!(
            "owners.effects.masks.edge_settings.path",
            on_buffer(&o.race.effects.get().masks.edge_settings.path),
        ),
        listed!(
            "owners.effects.masks.edge_settings.tilt_scale",
            on_game(&o.race.effects.get().masks.edge_settings.tilt_scale),
        ),
        listed!(
            "owners.effects.masks.ground_names.ground",
            on_game(&o.race.effects.get().masks.ground_names.ground),
        ),
        listed!(
            "owners.effects.masks.ground_names.ground_path",
            on_game(&o.race.effects.get().masks.ground_names.ground_path),
        ),
        listed!(
            "owners.effects.masks.ground_names.names",
            on_buffer(&o.race.effects.get().masks.ground_names.names),
        ),
        listed!(
            "owners.effects.ring.names.surfdata",
            on_buffer(&o.race.effects.get().ring.names.surfdata),
        ),
        listed!(
            "owners.effects.ring.names.surfmask",
            on_buffer(&o.race.effects.get().ring.names.surfmask),
        ),
        listed!(
            "owners.effects.masks.list",
            on_list(&o.race.effects.get().masks.list.list),
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
