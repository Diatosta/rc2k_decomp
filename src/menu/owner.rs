//! The menu module's owner: the menus' typed stores in one struct,
//! grouped by what they are for, in place of global stores. Reached through `menus()`, the
//! boundary for callers that do not take a group as a parameter yet.

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
use crate::menu::event_store::Event;
use crate::menu::flow_store::ScreenFlow;
use crate::menu::keys_store::KeyTables;
use crate::menu::list_state::ListState;
use crate::menu::loop_store::LoopFlags;
use crate::menu::menus_store::MenuTables;
use crate::menu::names_store::Names;
use crate::menu::options_store::MenuOptions;
use crate::menu::pages_store::MenuPages;
use crate::menu::play_films::Films;
use crate::menu::players_store::PlayerTable;
use crate::menu::results_store::Results;
use crate::menu::screen_items_store::ScreenItems;
use crate::menu::state_store::MenuState;
use crate::session::GameThread;

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
    pub event: Event,
    pub entry: EntryState,
    pub draw_work: DrawWork,
}

impl Screens {
    pub const fn new() -> Self {
        Screens {
            flow: ScreenFlow::new(),
            loop_flags: LoopFlags::new(),
            state: MenuState::new(),
            carousel: Carousel::new(),
            items: ScreenItems::new(),
            event: Event::new(),
            entry: EntryState::new(),
            draw_work: DrawWork::new(),
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
    pub players: PlayerTable,
    pub names: Names,
    pub cheats: Cheats,
}

impl Play {
    pub const fn new() -> Self {
        Play {
            arcade: Arcade::new(),
            champ_view: ChampView::new(),
            results: Results::new(),
            players: PlayerTable::new(),
            names: Names::new(),
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
}

/// The keyboard tables.
pub struct Keys {
    pub tables: KeyTables,
}

impl Keys {
    pub const fn new() -> Self {
        Keys {
            tables: KeyTables::new(),
        }
    }
}

/// The one instance. Reached through `menus()`; `moved` registers its stores' places.
pub static MENU_OWNER: GameThread<Menus> = GameThread::new(Menus::new());

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
        }
    }
}

/// The menus' state, for a caller that does not take it as a parameter yet.
pub fn menus() -> &'static Menus {
    MENU_OWNER.get()
}
