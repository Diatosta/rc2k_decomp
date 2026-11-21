//! The menu module's owner: the menus' typed stores in one struct, grouped by what they are for.
//! `Owners::build` makes one and hands it down in `Owners`; each test thread has its own.

use crate::memory::session::GameThread;
use crate::menu::arcade_store::Arcade;
use crate::menu::build_notice::BuildNotice;
use crate::menu::captions_store::CaptionTables;
use crate::menu::car_lists_store::CarLists;
use crate::menu::car_numbers_store::CarNumbers;
use crate::menu::carousel_store::Carousel;
use crate::menu::cars_store::CarTable;
use crate::menu::champ_store::ChampView;
use crate::menu::cheats_store::Cheats;
use crate::menu::draw_work_store::DrawWork;
use crate::menu::drivers_store::DriverTable;
use crate::menu::entry_store::EntryState;
use crate::menu::event_store::EventState;
use crate::menu::flow_store::ScreenFlow;
use crate::menu::keys_store::KeyTables;
use crate::menu::list_state::ListState;
use crate::menu::lists_store::MenuLists;
use crate::menu::loop_store::LoopFlags;
use crate::menu::menus_store::MenuTables;
use crate::menu::options_store::MenuOptions;
use crate::menu::pages_store::MenuPages;
use crate::menu::play_films::Films;
use crate::menu::players_store::PlayerSlots;
use crate::menu::results_store::Results;
use crate::menu::screen_items_store::ScreenItems;
use crate::menu::state_store::MenuState;
use crate::menu::texts_store::MenuTexts;
use crate::screens::controls_store::ControlsScreen;
use crate::screens::text_buffers_store::TextBuffers;
use crate::service::owner::Service;
use std::cell::{Ref, RefCell, RefMut};

/// The cars, their numbers and the drivers.
pub struct Cars {
    pub table: CarTable,
    pub numbers: CarNumbers,
    pub drivers: DriverTable,
    pub lists: CarLists,
}

impl Cars {
    pub const fn new() -> Self {
        Cars {
            table: CarTable::new(),
            numbers: CarNumbers::new(),
            drivers: DriverTable::new(),
            lists: CarLists::new(),
        }
    }
}

/// The fixed pieces the menus show: film names and the build notice.
pub struct Panels {
    pub films: Films,
    pub notice: BuildNotice,
}

impl Panels {
    pub const fn new() -> Self {
        Panels {
            films: Films::new(),
            notice: BuildNotice::new(),
        }
    }
}

/// The screens' running state: the flow between screens, the loop's flags, the menu
/// state, the shown items, the event, the key entry and the drawing's work.
pub struct Screens {
    pub flow: ScreenFlow,
    pub loop_flags: LoopFlags,
    pub state: MenuState,
    /// The menu carousel, shared with the scene that draws it.
    pub carousel: Carousel,
    pub items: ScreenItems,
    pub event: EventState,
    pub entry: EntryState,
    pub draw_work: DrawWork,
    /// The controls page's variables (history, learning).
    pub controls: ControlsScreen,
    /// The section and stage the stage-details screen saved when it was entered, put back when
    /// it ends (a stack: the section below, the stage above).
    pub stage_view_kept: std::sync::Mutex<Vec<u32>>,
}

impl Screens {
    pub const fn new() -> Self {
        Screens {
            flow: ScreenFlow::new(),
            loop_flags: LoopFlags::new(),
            state: MenuState::new(),
            carousel: Carousel::new(),
            items: ScreenItems::new(),
            event: EventState::new(),
            entry: EntryState::new(),
            draw_work: DrawWork::new(),
            controls: ControlsScreen::new(),
            stage_view_kept: std::sync::Mutex::new(Vec::new()),
        }
    }
}

/// The menus' tables: the menu records, the captions
/// and the pages.
pub struct Tables {
    pub menus: MenuTables,
    pub captions: CaptionTables,
    pub pages: MenuPages,
}

impl Tables {
    pub const fn new() -> Self {
        Tables {
            menus: MenuTables::new(),
            captions: CaptionTables::new(),
            pages: MenuPages::new(),
        }
    }
}

/// What a game played from the menus keeps: the arcade, the championship view, the
/// results, the players, the name buffers and the cheats.
pub struct Play {
    pub arcade: Arcade,
    pub champ_view: ChampView,
    pub results: Results,
    pub slots: PlayerSlots,
    pub cheats: Cheats,
}

impl Play {
    pub const fn new() -> Self {
        Play {
            arcade: Arcade::new(),
            champ_view: ChampView::new(),
            results: Results::new(),
            slots: PlayerSlots::new(),
            cheats: Cheats::new(),
        }
    }
}

/// The menu module's state.
pub struct Menus {
    pub cars: Cars,
    pub panels: Panels,
    pub keys: Keys,
    pub options: MenuOptions,
    pub screens: Screens,
    pub tables: Tables,
    pub play: Play,
    /// The state every list menu shares.
    pub list: ListState,
    /// The service area's set-ups, repair flags and times.
    pub service: Service,
    /// The picture paths and speech names the screens load by.
    pub texts: MenuTexts,
    /// The lists and tables the menus name: constants, except the inspection's fault flags,
    /// which the menus write. The car lists, the repair flags and the carousel's textures are
    /// held by their writers (`CarLists`, `Service`, `Carousel`).
    pub lists: MenuLists,
    /// The lines the screens build their texts in.
    pub text_buffers: TextBuffers,
}

/// The keyboard tables.
pub struct Keys {
    /// Plain fields behind one `RefCell`: the game's thread alone reads and writes them, each
    /// borrow lasting one statement.
    tables: GameThread<RefCell<KeyTables>>,
}

impl Keys {
    /// The tables, to read.
    pub fn tables(&self) -> Ref<'_, KeyTables> {
        self.tables.get().borrow()
    }

    /// The tables, to write.
    pub fn tables_mut(&self) -> RefMut<'_, KeyTables> {
        self.tables.get().borrow_mut()
    }

    pub const fn new() -> Self {
        Keys {
            tables: GameThread::new(RefCell::new(KeyTables::new())),
        }
    }
}

impl Menus {
    /// The menus' state at start-up.
    pub const fn new() -> Menus {
        Menus {
            cars: Cars::new(),
            panels: Panels::new(),
            keys: Keys::new(),
            options: MenuOptions::new(),
            screens: Screens::new(),
            tables: Tables::new(),
            play: Play::new(),
            list: ListState::new(),
            service: Service::new(),
            texts: MenuTexts::new(),
            lists: MenuLists::new(),
            text_buffers: TextBuffers::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    /// `Owners::build` builds the owner on the stack before boxing it: it must stay small.
    #[test]
    fn menus_are_small_enough_to_build_on_the_stack() {
        let size = std::mem::size_of::<super::Menus>();
        assert!(size < 256 * 1024, "{size} bytes");
    }
}
