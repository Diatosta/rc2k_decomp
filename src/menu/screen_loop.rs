//! The loop every menu screen runs, from entering the screen to leaving it. The screens
//! differ only in what their `Screen` describes; each has a file of its own that holds it.
//!
//! On entry the screen's `before` steps run. Set-up (again after every action that does not
//! leave the screen): the `set_up_first` steps, the highlight comes back from the screen's own
//! variable, the `prepare` steps, the menu and function lists are put in place, the
//! `background` steps, one frame is drawn, the `after_first_frame` steps.
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`; then
//! `menu::handle_keys`: 1 next frame, 2 (Escape) leave, 3 (Enter) the highlighted item's action
//! runs (`menu::run_item_action`, the highlight saved to the screen's variable around it); then
//! leave if `loop_flags.ask_leave`, `event.event_over` or `loop_flags.leave_screen` (cleared) is
//! set, else next frame if `draw_work.action_mode` is 2, else leave if `loop_flags.event_left`
//! is set, else set up again; anything else leaves. Optional per-screen behaviour is described
//! at the fields of `Screen`.
//!
//! Leaving: on a screen that asks (`confirm`), `menu::confirm_leaving` puts the question when
//! `left_by_key` is set; unless the answer is yes the `on_refusal` steps run and the screen is
//! set up again; after a yes the `confirmed` steps run. Otherwise the highlight is saved and the
//! `after` steps run. Wherever the highlight is saved or taken back, the Escape flags are
//! cleared if `draw_work.escape_pressed` is set (`clear_escape`).
//!
//! A refusal goes back to the loop's caller; a failed `Step::CallOrFatal` is the fatal error.

use crate::errors::{FatalError, Refused};
use crate::input::take_key::take_key;
use crate::logfile::program_log::log;
use crate::memory::session::{DwordLike, FlagLike, PlainFloat};
use crate::menu::handle_keys::handle_keys;
use crate::menu::lists_store::{CarList, CarListParts};
use crate::menu::menus_store::MenuId;
use crate::menu::owner::Menus;
use crate::menu::owner::{Screens, Tables};
use crate::menu::run_item_action::run_item_action;
use crate::menu::run_list::run_list;
use crate::menu::screen_id::ScreenId;
use crate::menu::update_blink::update_blink;
use crate::owners::Owners;
use crate::system::owner::System;
pub use rc2k_core::menu::keys::MenuSound;
use rc2k_core::menu::keys::Pressed;
pub use rc2k_core::menu::screen_loop::{ActionResult, Exit};
use rc2k_core::menu::screen_loop::{AskShape, Place, ScreenHost, Shape, screen_loop as core_loop};

/// The frame routine a screen runs each frame (named in the screen's description).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameRoutine {
    /// `menu::frame_callback`: every menu screen's.
    Menu,
    /// `menu::inspection_frame`: the inspection area's after its first frame.
    Inspection,
}

/// A game variable that a screen's steps write, named as data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenVar {
    /// `play.arcade.level`.
    ArcadeLevel,
    /// `play.arcade.stage`.
    ArcadeStage,
    /// `play.champ_view.stage_mode`.
    StageMode,
    /// `play.cheats.notice_due`.
    NoticeDue,
    /// `screens.draw_work.action_mode`.
    ActionMode,
    /// `screens.event.clears_escape_flag`.
    ClearsEscapeFlag,
    /// `screens.event.current_player`.
    CurrentPlayer,
    /// `screens.event.stage_in_section`.
    EventIndex,
    /// `screens.event.section`.
    EventSection,
    /// `screens.event.service.next`.
    ServiceNext,
    /// `screens.event.stage.group`.
    StageGroup,
    /// `screens.event.stage.number`.
    StageNumber,
    /// `screens.event.stage.sub`.
    StageSub,
    /// `screens.flow.left_service`.
    LeftService,
    /// `screens.flow.quiet`.
    Quiet,
    /// The highlighted item, `screens.loop_flags.item`.
    Highlight,
    /// The highlight a screen keeps between visits: `screens.items.current[slot]`.
    SavedItem(usize),
    /// `tables.pages.yes_no_answer`.
    YesNoAnswer,
}

impl ScreenVar {
    /// The variable in `menus`.
    pub fn cell(self, menus: &Menus) -> &dyn DwordLike {
        let screens = &menus.screens;
        match self {
            ScreenVar::ArcadeLevel => &menus.play.arcade.level,
            ScreenVar::ArcadeStage => &menus.play.arcade.stage,
            ScreenVar::StageMode => &menus.play.champ_view.stage_mode,
            ScreenVar::NoticeDue => &menus.play.cheats.notice_due,
            ScreenVar::ActionMode => &screens.draw_work.action_mode,
            ScreenVar::ClearsEscapeFlag => &screens.event.clears_escape_flag,
            ScreenVar::CurrentPlayer => &screens.event.current_player,
            ScreenVar::EventIndex => &screens.event.stage_in_section,
            ScreenVar::EventSection => &screens.event.section,
            ScreenVar::ServiceNext => &screens.event.service.next,
            ScreenVar::StageGroup => &screens.event.stage.group,
            ScreenVar::StageNumber => &screens.event.stage.number,
            ScreenVar::StageSub => &screens.event.stage.sub,
            ScreenVar::LeftService => &screens.flow.left_service,
            ScreenVar::Quiet => &screens.flow.quiet,
            ScreenVar::Highlight => &screens.loop_flags.item,
            ScreenVar::SavedItem(slot) => &screens.items.current[slot],
            ScreenVar::YesNoAnswer => &menus.tables.pages.yes_no_answer,
        }
    }
}

/// A game flag that a screen's steps write or its leaving reads, named as data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenFlag {
    /// `screens.draw_work.escape_pressed`.
    EscapePressed,
    /// `screens.event.event_over`.
    EventOver,
    /// `screens.flow.asking`: the "leave?" question is on.
    Asking,
    /// `screens.loop_flags.ask_leave`.
    AskLeave,
    /// `screens.loop_flags.event_left`.
    EventLeft,
    /// `screens.loop_flags.leave_screen`.
    LeaveScreen,
    /// `screens.loop_flags.left_with_carry`.
    LeftWithCarry,
    /// `screens.state.menus.skip_edit_values`.
    SkipEditValues,
    /// `screens.state.menus.standings_shown`.
    StandingsShown,
    /// `tables.captions.left_by_key`.
    LeftByKey,
}

impl ScreenFlag {
    /// The flag in `menus`.
    pub fn cell(self, menus: &Menus) -> &dyn FlagLike {
        let screens = &menus.screens;
        match self {
            ScreenFlag::EscapePressed => &screens.draw_work.escape_pressed,
            ScreenFlag::EventOver => &screens.event.event_over,
            ScreenFlag::Asking => &screens.flow.asking,
            ScreenFlag::AskLeave => &screens.loop_flags.ask_leave,
            ScreenFlag::EventLeft => &screens.loop_flags.event_left,
            ScreenFlag::LeaveScreen => &screens.loop_flags.leave_screen,
            ScreenFlag::LeftWithCarry => &screens.loop_flags.left_with_carry,
            ScreenFlag::SkipEditValues => &screens.state.menus.skip_edit_values,
            ScreenFlag::StandingsShown => &screens.state.menus.standings_shown,
            ScreenFlag::LeftByKey => &menus.tables.captions.left_by_key,
        }
    }
}

/// An integer variable of the front end's scene that a screen's steps write: its view, its
/// clip edges, its object lighting.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SceneVar {
    /// The view's x offset.
    ViewOffsetX,
    /// The view's y offset.
    ViewOffsetY,
    /// The view's zoom.
    ViewZoom,
    /// The clip's left edge.
    ClipLeft,
    /// The clip's right edge.
    ClipRight,
    /// The lowest light.
    LightLowest,
    /// The highest light.
    LightHighest,
    /// The light's level.
    LightLevel,
    /// The light's first turn.
    LightTurn0,
    /// The light's second turn.
    LightTurn1,
    /// The strength of the object's light.
    ObjectLightStrength,
}

impl SceneVar {
    /// The variable in the front end's scene.
    pub fn cell(self, scratch: &crate::scene::workers::DrawingScratch) -> &dyn DwordLike {
        let window = &scratch.front_end_polygon().window;
        let light = &scratch.front_end().light;
        match self {
            SceneVar::ViewOffsetX => &window.view_offset[0],
            SceneVar::ViewOffsetY => &window.view_offset[1],
            SceneVar::ViewZoom => &window.view_zoom,
            SceneVar::ClipLeft => &window.clip_edges[2],
            SceneVar::ClipRight => &window.clip_edges[3],
            SceneVar::LightLowest => &light.light_lowest,
            SceneVar::LightHighest => &light.light_highest,
            SceneVar::LightLevel => &light.light_level,
            SceneVar::LightTurn0 => &light.light_turns[0],
            SceneVar::LightTurn1 => &light.light_turns[1],
            SceneVar::ObjectLightStrength => &scratch.front_end().object.light_strength,
        }
    }
}

/// A float variable of the front end's scene that a screen's steps write.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SceneFloat {
    /// The clip's left edge as the clipper reads it.
    ClipLeft,
    /// The clip's right edge as the clipper reads it.
    ClipRight,
}

impl SceneFloat {
    /// The variable in the front end's scene.
    pub fn cell(self, scratch: &crate::scene::workers::DrawingScratch) -> &PlainFloat {
        let window = &scratch.front_end_polygon().window;
        match self {
            SceneFloat::ClipLeft => &window.clip_screen[2],
            SceneFloat::ClipRight => &window.clip_screen[3],
        }
    }
}

/// A text of the game that a screen's description names (a picture's path), found when
/// asked.
pub type Text = fn(&Menus) -> &dyn crate::memory::session::TextPlace;

/// Something a screen does at one of the loop's fixed places.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    /// Calls the game function at this RVA (one of `call`'s).
    Call(Callee),
    /// Writes a game variable.
    Write(ScreenVar, u32),
    /// Sets or clears a game flag.
    WriteFlag(ScreenFlag, bool),
    /// Sets the menu's sound variable.
    Sound(MenuSound),
    /// Writes an integer variable of the front end's scene.
    WriteScene(SceneVar, u32),
    /// Writes a float variable of the front end's scene.
    WriteSceneF32(SceneFloat, f32),
    /// Calls the game function, and if it fails, that is the fatal error with this code (and no
    /// message).
    CallOrFatal(Callee, u32),
    /// Something only this screen does, written out in its file.
    Do(fn(&Entry) -> Result<(), Refused>),
    /// A step that reads nothing of the menus.
    DoPlain(fn() -> Result<(), Refused>),
    /// A step that reads only the screens' state.
    DoScreens(fn(&Screens) -> Result<(), Refused>),
    /// A step that reads only the menus' state.
    DoMenus(fn(&Menus) -> Result<(), Refused>),
    /// A step that reads the menus' state and the system's.
    DoMenusSystem(fn(&Menus, &System) -> Result<(), Refused>),
    /// A step that puts the screen's 3D scene in place: the menus, the front end's view and
    /// the owners the scene's preparation reads.
    DoSetUp(fn(&SetUp) -> Result<(), Refused>),
    /// A flow step that reads only the menus' tables.
    DoTablesFlow(fn(&Tables) -> Result<Flow, Refused>),
    /// A flow step that reads only the screens' state.
    DoScreensFlow(fn(&Screens) -> Result<Flow, Refused>),
    /// A flow step that reads only the system's state.
    DoSystemFlow(fn(&System) -> Result<Flow, Refused>),
    /// The same for a step that can also send the loop somewhere: a name editor called once
    /// a frame that keeps the keys from the menu while a name is typed. Only looked at in
    /// `each_frame`.
    DoFlow(fn(&Entry) -> Result<Flow, Refused>),
}
impl Step {
    /// Runs the step if it only writes a variable (to `menus`, or the front end's scene):
    /// whether it did.
    pub fn write(&self, menus: &Menus, scratch: &crate::scene::workers::DrawingScratch) -> bool {
        match *self {
            Step::Write(variable, value) => variable.cell(menus).set(value),
            Step::WriteFlag(flag, on) => flag.cell(menus).set(on),
            Step::Sound(sound) => menus.screens.entry.set_sound(sound),
            Step::WriteScene(variable, value) => variable.cell(scratch).set(value),
            Step::WriteSceneF32(variable, value) => variable.cell(scratch).set(value),
            Step::Call(_)
            | Step::CallOrFatal(..)
            | Step::Do(_)
            | Step::DoPlain(_)
            | Step::DoScreens(_)
            | Step::DoMenus(_)
            | Step::DoMenusSystem(_)
            | Step::DoSetUp(_)
            | Step::DoTablesFlow(_)
            | Step::DoScreensFlow(_)
            | Step::DoSystemFlow(_)
            | Step::DoFlow(_) => return false,
        }
        true
    }
}
pub use rc2k_core::menu::screen_loop::Flow;

/// One of the screen's function lists: the table, and the variable that points at it.
#[derive(Clone, Copy, Debug)]
pub struct List {
    pub pointer: fn(&Menus) -> &crate::menu::fn_list::ListCell,
    pub table: fn(&Menus) -> crate::menu::fn_list::FnList,
    /// Run with `menu::run_list` at set-up, before its pointer is stored. Such a list is not
    /// put in place again before each frame; the others are.
    pub run: bool,
}

/// How a screen is asked about when it is left (only the screens that ask have one).
#[derive(Clone, Copy)]
pub struct Asking {
    /// What runs after a "no" (the screen is then set up again).
    pub on_refusal: &'static [Step],
    /// A variable that, when it is 1, has the question put as well (`loop_flags.ask_leave` in the
    /// menus after the results screens).
    pub also_if: Option<ScreenFlag>,
    /// A variable that, when it is 1, keeps the question from being put although
    /// `captions.left_by_key` is set.
    pub unless: Option<fn(&Owners<'static>) -> u32>,
    /// The question is put whenever the screen is left, and after a "yes" the loop returns
    /// at once: the highlight is not saved.
    pub always: bool,
    /// What runs before the question is put.
    pub before: &'static [Step],
    /// What runs after a "yes".
    pub confirmed: &'static [Step],
    /// When the question is not put, the highlight is not saved and the Escape flag is left
    /// alone: only the `after` steps run.
    pub unasked_skips_save: bool,
}

impl Asking {
    /// Nothing runs around the question, which is put by `captions.left_by_key` alone.
    pub const DEFAULT: Asking = Asking {
        on_refusal: &[],
        also_if: None,
        unless: None,
        always: false,
        before: &[],
        confirmed: &[],
        unasked_skips_save: false,
    };
}

pub use rc2k_core::menu::screen_loop::ActionCheck;

/// What differs in how a screen takes its keys and their actions.
#[derive(Clone, Copy)]
pub struct Keys {
    /// The Escape flag `draw_work.escape_pressed` is looked at at the top of every frame, before
    /// the quit switch, and the screen is left when it is 1.
    pub escape_leaves: bool,
    /// What is looked at after an action.
    pub action_check: ActionCheck,
    /// The variable the highlight is kept in around an item's action, where it is not
    /// `saved_item`.
    /// The slot of `screens.items.current` it is kept in.
    pub action_saved_item: Option<usize>,
    /// The menu whose item's action is run, where it is not `menu`: three loops name a
    /// neighbouring screen's menu there (a 1999 slip kept as is).
    pub action_menu: Option<fn(&Menus) -> MenuId>,
}

impl Keys {
    /// The screen's own menu everywhere, Escape not looked at, the standard action check.
    pub const DEFAULT: Keys = Keys {
        escape_leaves: false,
        action_check: ActionCheck::Standard,
        action_saved_item: None,
        action_menu: None,
    };
}

/// What differs in a screen's frames.
#[derive(Clone, Copy)]
pub struct Frames {
    pub callback: FrameRoutine,
    /// The frame callback of every frame but the first after set-up, where it is another
    /// one: the inspection area's loop is the only one.
    pub later_callback: Option<FrameRoutine>,
    /// The lists put in place before the first frame after set-up, where they are not the
    /// same as before every other frame: 35 loops put their 3D list in the unread list at
    /// set-up and before each frame, and a list of their own in the scene list only here.
    pub first_frame_lists: Option<&'static [List]>,
    /// "Next frame" (after `handle_keys` returns 1, and after an action that leaves 2 in
    /// `draw_work.action_mode`) goes to the `after_first_frame` steps, not past them. A `Flow`
    /// those steps return is honoured on every screen.
    pub next_frame_repeats_first: bool,
    /// Steps before the first frame after set-up.
    pub before_first_frame: &'static [Step],
    /// Steps before every other frame.
    pub before_frame: &'static [Step],
    /// Steps right after `update_blink` at the top of a frame; a `Flow` is honoured.
    pub after_blink: &'static [Step],
    /// Steps of the screen's own once a frame, after the frame and before the keys.
    pub each_frame: &'static [Step],
    /// `entry.menu_sound` = 0x6F is stored once a frame, before the frame.
    pub clears_menu_sound: bool,
}

impl Frames {
    /// The menu's frame callback every frame, the lists the same before every frame, no
    /// steps of the screen's own.
    pub const DEFAULT: Frames = Frames {
        callback: FrameRoutine::Menu,
        later_callback: None,
        first_frame_lists: None,
        next_frame_repeats_first: false,
        before_first_frame: &[],
        before_frame: &[],
        after_blink: &[],
        each_frame: &[],
        clears_menu_sound: false,
    };
}

/// What one screen is: data. A screen is `Screen::new(module_path!(), menu, saved_item)` and the
/// fields that differ; `ScreenId` names it.
#[derive(Clone, Copy)]
pub struct Screen {
    /// The module path of the screen's file (`module_path!()`), or the path a shared description
    /// stands in for, the name its log lines carry.
    pub module_path: &'static str,
    /// The screen's menu.
    pub menu: fn(&Menus) -> MenuId,
    /// The variable the highlight is kept in between visits.
    /// The slot of `screens.items.current` it is kept in.
    pub saved_item: usize,
    pub lists: &'static [List],
    /// Steps run once, on entering the screen.
    pub before: &'static [Step],
    pub set_up_first: &'static [Step],
    pub prepare: &'static [Step],
    /// Steps between putting the lists in place and the background.
    pub after_lists: &'static [Step],
    pub background: &'static [Step],
    pub after_first_frame: &'static [Step],
    /// Some if leaving is asked about.
    pub asking: Option<Asking>,
    pub after: &'static [Step],
    /// A variable that, when it is 1 at the moment of leaving, makes the loop return at once:
    /// no question, the highlight not saved, no `after` steps.
    pub leave_at_once: Option<ScreenFlag>,
    /// On a screen that does not ask: what runs first on leaving, before the highlight is
    /// saved (the mode menu calls `menu::go_back` there when `captions.left_by_key` is set).
    pub leaving: &'static [Step],
    /// The variable the highlight is saved in on leaving, where it is not `saved_item`.
    pub leave_saved_item: Option<usize>,
    /// What runs last of all, with the carried values back as they were at entry.
    pub after_restored: &'static [Step],
    /// What the loop returns, where it returns something: read from the game's
    /// variables as the loop leaves them.
    pub result: Option<fn(&Menus) -> u32>,
    /// `captions.left_by_key` is cleared and the highlight put in place (with the Escape check)
    /// after the first frame, not at the start of set-up, and set-up stores neither
    /// `entry.menu_sound` nor `loop_flags.leave_screen` after the lists: the main menu.
    pub late_highlight: bool,
    pub keys: Keys,
    pub frames: Frames,
}

impl Screen {
    /// What the loop needs to know of the screen.
    pub fn shape(&self) -> Shape {
        Shape {
            late_highlight: self.late_highlight,
            escape_leaves: self.keys.escape_leaves,
            action_check: self.keys.action_check,
            clears_menu_sound: self.frames.clears_menu_sound,
            next_frame_repeats_first: self.frames.next_frame_repeats_first,
            asking: self.asking.as_ref().map(|asking| AskShape {
                always: asking.always,
                unasked_skips_save: asking.unasked_skips_save,
            }),
        }
    }

    /// A screen with its menu and the variable its highlight is kept in, and every other
    /// field off or empty; a screen writes what differs with `..Screen::new(module_path!(), ..)`.
    pub const fn new(
        module_path: &'static str,
        menu: fn(&Menus) -> MenuId,
        saved_item: usize,
    ) -> Screen {
        Screen {
            module_path,
            menu,
            saved_item,
            lists: &[],
            before: &[],
            set_up_first: &[],
            prepare: &[],
            after_lists: &[],
            background: &[],
            after_first_frame: &[],
            asking: None,
            after: &[],
            leave_at_once: None,
            leaving: &[],
            leave_saved_item: None,
            after_restored: &[],
            result: None,
            late_highlight: false,
            keys: Keys::DEFAULT,
            frames: Frames::DEFAULT,
        }
    }
}

/// What a screen is entered with that its steps read: the list of cars its carousel shows
/// (`scene::set_up_carousel` reads it for menus 0xD and 0xE).
/// It also carries the menus' owner, which the steps read.
#[derive(Clone, Copy)]
pub struct Entry {
    pub menus: &'static Menus,
    pub cars: Option<CarList>,
    /// The front end's slot of polygon work (its window is the screens' view).
    pub front_end: &'static crate::scene::polygon_work::PolygonWork,
    /// Every owner: the screens reach the others (`owners.system`, `owners.race`, ..) through it.
    pub owners: Owners<'static>,
}

impl Entry {
    /// The view's offset and zoom a front-end screen draws with.
    pub fn set_view(&self, x: i32, y: i32, zoom: u32) {
        let window = &self.front_end.window;
        window.view_offset[0].set(x as u32);
        window.view_offset[1].set(y as u32);
        window.view_zoom.set(zoom);
    }

    /// Entered with nothing carried in.
    pub fn new(o: Owners<'static>) -> Self {
        Entry::with_cars(o, None)
    }
    /// What a set-up step reads of this entry.
    pub fn set_up(&self) -> SetUp<'_> {
        SetUp {
            menus: self.menus,
            cars: self.cars,
            front_end: self.front_end,
            race: self.owners.race,
            scene: self.owners.scene,
            system: self.owners.system,
        }
    }

    /// The same entry, carrying the list of cars the carousel shows.
    pub fn with_list(&self, cars: Option<CarList>) -> Self {
        Entry {
            menus: self.menus,
            cars,
            front_end: self.front_end,
            owners: self.owners,
        }
    }
    /// Entered with the list of cars the carousel shows.
    pub fn with_cars(o: Owners<'static>, cars: Option<CarList>) -> Self {
        Entry {
            menus: o.menus,
            cars,
            front_end: o.scene.scratch.front_end_polygon(),
            owners: o,
        }
    }
}

/// What a screen's set-up reads to put its 3D scene in place: the menus, the list of cars its
/// carousel shows, the front end's view, and the owners the scene's preparation reads.
#[derive(Clone, Copy)]
pub struct SetUp<'a> {
    pub menus: &'a Menus,
    pub cars: Option<CarList>,
    front_end: &'a crate::scene::polygon_work::PolygonWork,
    race: &'a crate::race::owner::Race,
    scene: &'a crate::scene::owner::Scene,
    system: &'a System,
}

impl SetUp<'_> {
    /// The view's offset and zoom the screen draws with.
    pub fn set_view(&self, x: i32, y: i32, zoom: u32) {
        let window = &self.front_end.window;
        window.view_offset[0].set(x as u32);
        window.view_offset[1].set(y as u32);
        window.view_zoom.set(zoom);
    }

    /// The clip's left and right edges (also as the floats the clipper reads).
    pub fn set_clip_sides(&self, left: i32, right: i32) {
        let window = &self.front_end.window;
        window.clip_edges[2].set(left as u32);
        window.clip_edges[3].set(right as u32);
        window.clip_screen[2].set(left as f32);
        window.clip_screen[3].set(right as f32);
    }

    /// Prepares the menu scene.
    pub fn prepare_scene(&self) {
        crate::scene::prepare_menu_scene::prepare_menu_scene(
            self.race,
            self.scene,
            self.system,
            self.scene.scratch.front_end(),
        );
    }

    /// Sets up the carousel with the list of cars the screen carries.
    pub fn carousel(&self) -> Result<(), Refused> {
        self.carousel_with(self.cars)
    }

    /// Sets up the carousel with the given list of cars.
    pub fn carousel_with(&self, cars: Option<CarList>) -> Result<(), Refused> {
        crate::scene::set_up_carousel::set_up_carousel(
            CarListParts::of(self.menus),
            &self.scene.scratch,
            cars,
        )
    }
}

/// Whether the game is quitting.
pub fn quitting(system: &System) -> bool {
    system.session.flags.quitting.is_set()
}

/// `scene::prepare_menu_scene`.
pub fn prepare_scene(owners: &Owners<'static>) {
    crate::scene::prepare_menu_scene::prepare_menu_scene(
        owners.race,
        owners.scene,
        owners.system,
        owners.scene.scratch.front_end(),
    );
}

/// `scene::set_up_carousel`, with the list of cars menus 0xD and 0xE show.
/// (`_scene`: the loops pass it; the carousel's set-up writes the front end's slot.)
pub fn carousel(entry: &Entry, cars: Option<CarList>) -> Result<(), Refused> {
    crate::scene::set_up_carousel::set_up_carousel(
        CarListParts::of(entry.menus),
        &entry.owners.scene.scratch,
        cars,
    )
}

/// A line for the game's log at level 1 with its line break.
pub fn log_line(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 1);
}

/// A game function a screen step runs with what it is handed.
#[derive(Clone, Copy, Debug)]
pub enum Callee {
    /// Takes the pending key.
    TakeKey,
    /// Loads the screen's background picture from the given path.
    LoadBackground(Text),
    /// Prepares the menu scene.
    PrepareScene,
    /// Applies the controller page's choice.
    ApplyController,
    /// Lets go of the controller the page was reading.
    ReleaseController,
    /// Draws the map of the stage choice.
    DrawStageMap,
    /// Stops the music stream.
    StopStream,
    /// Plays tune 4.
    PlayTune4,
    /// Plays tune 9.
    PlayTune9,
    /// Stops the music.
    StopMusic,
    /// Steps the car sound stream on.
    StepCarStream,
    /// Starts the car sound stream.
    PlayCarStream,
    /// Whether the given controller axis has moved.
    AxisMoved(u32),
    /// Ranks the players by their stage times.
    RankStageTimes,
    /// Ranks the players by their totals.
    RankTotals,
    /// Scores the results of four players.
    ScoreFourPlayers,
    /// Scores the results of two players.
    ScoreTwoPlayers,
    /// Plays the speech for the stage shown.
    PlayStageSpeech,
    /// Plays the speech for the rally shown.
    PlayRallySpeech,
    /// Loads the welcome screen's picture.
    LoadWelcomePicture,
    /// Loads the service area's background.
    LoadServiceBackground,
    /// Marks the stage chosen.
    MarkStageChoice,
    /// Runs the player name editor.
    EditPlayerName,
    /// Runs the set-up name editor.
    EditSetupName,
    /// Runs the saved game name editor.
    EditSavegameName,
    /// Runs the replay name editor.
    EditReplayName,
    /// Checks the name entered against the cheat names.
    CheckCheatName,
    /// Loads the slideshow's picture.
    LoadSlideshowPicture,
    /// Loads the car screen's background.
    LoadCarBackground,
    /// Loads a picture from the given path.
    LoadPicture(Text),
    /// Converts the loaded picture to the screen's format.
    ConvertPicture,
    /// Sets up the front end.
    SetUpFrontEnd,
    /// Sets up the 3D engine.
    SetUpEngine,
    /// Sets up the front end's scene.
    SetUpFrontEndScene,
}

impl Callee {
    /// Runs the function: whether it failed (false for those that cannot).
    pub fn run(self, entry: &Entry) -> Result<bool, Refused> {
        let menus = entry.menus;
        let (audio, input, sound) = (entry.owners.audio, entry.owners.input, entry.owners.sound);
        let play = &menus.play;
        let screens = &menus.screens;
        use crate::menu::{controller_items as ci, results_setups as rs, stage_view_steps as v};
        use crate::text::draw_text::GameCanvas;
        match self {
            Self::TakeKey => {
                take_key(input);
            }
            // It never fails.
            Self::LoadBackground(path) => {
                crate::menu::load_background::load_background_in(
                    path(menus),
                    &mut GameCanvas::new(entry.owners.gfx),
                    menus,
                    entry.owners.gfx,
                    entry.owners.system,
                )?;
            }
            Self::PrepareScene => prepare_scene(&entry.owners),
            Self::ApplyController => ci::apply_controller(menus, input)?,
            Self::ReleaseController => ci::release_controller(input),
            Self::DrawStageMap => crate::menu::stage_choice::draw_stage_map(entry.owners)?,
            Self::StopStream => crate::sound::stop_stream::stop_stream(entry.owners),
            Self::PlayTune4 => {
                crate::sound::play_tune_4::play_tune_4(entry.owners);
            }
            Self::PlayTune9 => {
                crate::sound::play_tune_9::play_tune_9(entry.owners);
            }
            Self::StopMusic => crate::sound::stop_music::stop_music(entry.owners),
            Self::StepCarStream => {
                crate::sound::step_car_stream::step_car_stream_in_game(entry.owners)?;
            }
            Self::PlayCarStream => crate::sound::play_car_stream::play_car_stream(
                &mut |name, volume| {
                    crate::sound::start_stream::start_stream(entry.owners, name, volume);
                },
                menus,
                sound,
                &audio.sample_names,
            )?,
            Self::AxisMoved(axis) => {
                crate::input::axes::axis_moved(axis, input)?;
                // The rest values taken, the page waits a moment before it reads the axes.
                if axis == 0 {
                    crate::system::pause::hold(entry.owners, 250);
                }
            }
            Self::RankStageTimes => rs::rank_stage_times(menus)?,
            Self::RankTotals => rs::rank_totals(menus)?,
            Self::ScoreFourPlayers => rs::score_four_players(menus, entry.owners.system)?,
            Self::ScoreTwoPlayers => rs::score_two_players(menus, entry.owners.system)?,
            Self::PlayStageSpeech => v::play_stage_speech(entry.owners)?,
            Self::PlayRallySpeech => v::play_rally_speech(entry.owners)?,
            Self::LoadWelcomePicture => v::load_welcome_picture(entry.owners)?,
            Self::LoadServiceBackground => crate::service::load_background::load_background_in(
                menus,
                entry.owners.gfx,
                entry.owners.system,
            )?,
            Self::MarkStageChoice => {
                crate::menu::mark_stage_choice::mark_stage_choice(&play.champ_view);
            }
            Self::EditPlayerName => {
                crate::menu::edit_player_name::edit_player_name(
                    entry.owners.input,
                    menus,
                    screens,
                    &mut GameCanvas::new(entry.owners.gfx),
                )?;
            }
            Self::EditSetupName => {
                crate::service::edit_setup_name::edit_setup_name(
                    entry.owners.input,
                    crate::strings::append_string_by_id::StringOwners::of(entry.owners),
                    menus,
                    &menus.service,
                    &mut GameCanvas::new(entry.owners.gfx),
                )?;
            }
            Self::EditSavegameName => crate::menu::edit_savegame_name::edit_savegame_name(
                entry.owners.input,
                crate::strings::append_string_by_id::StringOwners::of(entry.owners),
                menus,
                entry.owners.saves,
                &mut GameCanvas::new(entry.owners.gfx),
                &mut |slot| {
                    crate::menu::edit_savegame_name::save_game(menus, entry.owners.saves, slot);
                },
            )?,
            Self::EditReplayName => {
                return crate::menu::edit_replay_name::edit_replay_name(
                    entry.owners.input,
                    crate::strings::append_string_by_id::StringOwners::of(entry.owners),
                    menus,
                    entry.owners.saves,
                    entry.owners.race,
                    &mut GameCanvas::new(entry.owners.gfx),
                );
            }
            Self::CheckCheatName => {
                crate::menu::check_cheat_name::check_cheat_name(entry.owners)?;
            }
            // The slideshow screen's set-up leaves the way to look 0 or (after an action) a
            // small number: always on.
            Self::LoadSlideshowPicture => {
                crate::menu::load_slideshow_picture::load_slideshow_picture_in(entry.owners, 0)?;
            }
            Self::LoadCarBackground => {
                crate::menu::load_car_background::load_car_background(
                    menus,
                    entry.owners.gfx,
                    entry.owners.system,
                )?;
            }
            Self::LoadPicture(path) => {
                return Ok(!crate::gfx::load_picture::load_picture_with(
                    path(menus),
                    entry.owners.gfx,
                    entry.owners.system,
                )?);
            }
            Self::ConvertPicture => {
                crate::gfx::convert_picture::convert_picture_now_in(entry.owners.gfx)?;
                return Ok(false);
            }
            Self::SetUpFrontEnd => {
                crate::race::load_times::begin("front end: sprites, names, best times");
                crate::menu::set_up_front_end::set_up_front_end(entry.owners)?;
                crate::race::load_times::end();
            }
            Self::SetUpEngine => {
                crate::scene::set_up_engine::set_up_engine_in(
                    entry.owners.race,
                    entry.owners.scene,
                )?;
            }
            Self::SetUpFrontEndScene => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(entry.owners)?;
            }
        }
        Ok(false)
    }
}

/// Clears the Escape flags if they are set, whether or not one of the four screens that set
/// `event.clears_escape_flag` (start rally, options, player 1 details, information) has been
/// seen. Fixed bug (`wiki/Fixed-bugs.md`, Escape on the "leave?" question leaves the screen
/// anyway): before, a "no" given with Escape left the screen below at its next frame.
/// Returns whether it was set (and so `leave_screen` was cleared with it).
fn clear_escape(screens: &Screens) -> bool {
    let pressed = screens.draw_work.escape_pressed.is_set();
    if pressed {
        screens.draw_work.escape_pressed.set(false);
        screens.loop_flags.leave_screen.set(false);
    }
    pressed
}

/// A call whose failure is the fatal error `code`.
pub fn call_or_fatal(entry: &Entry, function: Callee, code: u32) -> Result<(), Refused> {
    if { function.run(entry) }? {
        log(&format!("a screen loop raises fatal error {code:#x}"));
        return Err(FatalError { code, message: "" }.into());
    }
    Ok(())
}

/// Runs the steps of one place. Anything but `Flow::On` comes from a `DoFlow` step, after
/// which the rest of the place is not run.
fn run(steps: &[Step], entry: &Entry) -> Result<Flow, Refused> {
    let menus = entry.menus;
    for step in steps {
        if step.write(menus, &entry.owners.scene.scratch) {
            continue;
        }
        match *step {
            Step::DoFlow(what) => match what(entry)? {
                Flow::On => {}
                elsewhere => return Ok(elsewhere),
            },
            Step::Call(function) => {
                function.run(entry)?;
            }
            Step::CallOrFatal(function, code) => call_or_fatal(entry, function, code)?,
            Step::DoTablesFlow(what) => match what(&menus.tables)? {
                Flow::On => {}
                elsewhere => return Ok(elsewhere),
            },
            Step::DoScreensFlow(what) => match what(&menus.screens)? {
                Flow::On => {}
                elsewhere => return Ok(elsewhere),
            },
            Step::DoSystemFlow(what) => match what(entry.owners.system)? {
                Flow::On => {}
                elsewhere => return Ok(elsewhere),
            },
            Step::DoMenusSystem(what) => what(menus, entry.owners.system)?,
            Step::DoSetUp(what) => what(&entry.set_up())?,
            Step::Do(what) => what(entry)?,
            Step::DoPlain(what) => what()?,
            Step::DoScreens(what) => what(&menus.screens)?,
            Step::DoMenus(what) => what(menus)?,
            Step::Write(..)
            | Step::WriteFlag(..)
            | Step::Sound(_)
            | Step::WriteScene(..)
            | Step::WriteSceneF32(..) => {}
        }
    }
    Ok(Flow::On)
}

/// The steps the screen runs at one place of the loop.
fn steps_at(screen: &Screen, place: Place) -> &'static [Step] {
    let asking = screen.asking.as_ref();
    match place {
        Place::Before => screen.before,
        Place::SetUpFirst => screen.set_up_first,
        Place::Prepare => screen.prepare,
        Place::AfterLists => screen.after_lists,
        Place::Background => screen.background,
        Place::BeforeFirstFrame => screen.frames.before_first_frame,
        Place::AfterFirstFrame => screen.after_first_frame,
        Place::AfterBlink => screen.frames.after_blink,
        Place::BeforeFrame => screen.frames.before_frame,
        Place::EachFrame => screen.frames.each_frame,
        Place::AskBefore => asking.map_or(&[][..], |asking| asking.before),
        Place::OnRefusal => asking.map_or(&[][..], |asking| asking.on_refusal),
        Place::Confirmed => asking.map_or(&[][..], |asking| asking.confirmed),
        Place::Leaving => screen.leaving,
        Place::After => screen.after,
        Place::AfterRestored => screen.after_restored,
    }
}

/// Puts the menu and the lists in place before a frame.
fn set_lists(menus: &Menus, screens: &Screens, screen: &Screen, lists: &[List]) {
    screens.loop_flags.menu.set((screen.menu)(menus));
    for list in lists.iter().filter(|list| !list.run) {
        (list.pointer)(menus).set((list.table)(menus));
    }
}

/// `screens::run_frame` with the screen's frame callback: the messages, then
/// (unless quitting) the frame, with the menu and the carried palette and x. Neither
/// changes a carried value; whether the frame was made is not read.
fn run_frame(o: Owners<'static>, callback: FrameRoutine, menu: MenuId) -> Result<(), Refused> {
    let frame = || match callback {
        FrameRoutine::Menu => crate::menu::frame_callback::frame_callback(o, menu),
        FrameRoutine::Inspection => crate::menu::inspection_frame::inspection_frame(o, menu),
    };
    match crate::platform::run_frame_now(o, frame) {
        Some(made) => made.map(|_| ()),
        None => Ok(()),
    }
}

/// The screen loop's host: what the game does at each place of `rc2k_core`'s loop, on this
/// screen's menus.
struct MenuHost<'a> {
    screen: &'a Screen,
    entry: &'a Entry,
    menu: MenuId,
    action_menu: MenuId,
}

impl MenuHost<'_> {
    fn screens(&self) -> &Screens {
        &self.entry.menus.screens
    }

    /// The variable the highlight is kept in around an item's action.
    fn action_item(&self) -> &dyn DwordLike {
        ScreenVar::SavedItem(
            self.screen
                .keys
                .action_saved_item
                .unwrap_or(self.screen.saved_item),
        )
        .cell(self.entry.menus)
    }
}

impl ScreenHost for MenuHost<'_> {
    type Error = Refused;

    fn run(&mut self, place: Place) -> Result<Flow, Refused> {
        run(steps_at(self.screen, place), self.entry)
    }

    fn restore_highlight(&mut self) {
        let menus = self.entry.menus;
        menus.tables.captions.left_by_key.set(false);
        self.screens().loop_flags.item.set(
            ScreenVar::SavedItem(self.screen.saved_item)
                .cell(menus)
                .get(),
        );
        clear_escape(self.screens());
    }

    fn put_menu_and_lists(&mut self) -> Result<(), Refused> {
        let menus = self.entry.menus;
        self.screens().loop_flags.menu.set(self.menu);
        for list in self.screen.lists {
            let table = (list.table)(menus);
            if list.run {
                run_list(self.entry.owners, table)?;
            }
            (list.pointer)(menus).set(table);
        }
        Ok(())
    }

    fn clear_menu_sound(&mut self) {
        self.screens().entry.set_sound(MenuSound::None);
    }

    fn clear_leave_screen(&mut self) {
        self.screens().loop_flags.leave_screen.set(false);
    }

    fn draw_set_up_frame(&mut self) -> Result<(), Refused> {
        let menus = self.entry.menus;
        crate::menu::draw_frame::draw_frame(self.entry.owners, self.menu)?;
        update_blink(menus, self.entry.owners.system);
        Ok(())
    }

    fn note_set_up_time(&mut self) {
        let clocks = self.entry.owners.system.clocks();
        clocks.setup_time.set(clocks.frame.get());
        self.entry
            .owners
            .race
            .shared
            .physics_tables
            .get()
            .scene_clock
            .set(0u32);
    }

    fn put_frame_lists(&mut self, first: bool) {
        let screen = self.screen;
        let lists = if first {
            screen.frames.first_frame_lists.unwrap_or(screen.lists)
        } else {
            screen.lists
        };
        set_lists(self.entry.menus, self.screens(), screen, lists);
    }

    fn run_frame(&mut self, first: bool) -> Result<(), Refused> {
        let frames = &self.screen.frames;
        let callback = if first {
            frames.callback
        } else {
            frames.later_callback.unwrap_or(frames.callback)
        };
        run_frame(self.entry.owners, callback, self.menu)
    }

    fn update_blink(&mut self) {
        update_blink(self.entry.menus, self.entry.owners.system);
    }

    fn escape_flag(&self) -> bool {
        self.screens().draw_work.escape_pressed.is_set()
    }

    fn quitting(&self) -> bool {
        quitting(self.entry.owners.system)
    }

    fn leave_on_close(&mut self) -> ! {
        // At once: a screen left as if chosen lets the flow behind it go on with what the
        // screen never set (a championship's next stage: a fatal error).
        crate::standalone::leave_on_close(self.entry.owners)
    }

    fn debug_serve(&mut self) -> Result<bool, Refused> {
        crate::debug::screens::serve(self.entry.owners)
    }

    fn note_frame_time(&mut self) {
        let clocks = self.entry.owners.system.clocks();
        self.entry
            .owners
            .race
            .shared
            .physics_tables
            .get()
            .scene_clock
            .set(clocks.frame.get().wrapping_sub(clocks.setup_time.get()));
    }

    fn handle_keys(&mut self) -> Result<Pressed, Refused> {
        // The handler is handed the time since set-up, as noted for this frame.
        handle_keys(
            self.entry.owners,
            self.entry
                .owners
                .race
                .shared
                .physics_tables
                .get()
                .scene_clock
                .get(),
        )
    }

    fn action_pending(&self) -> bool {
        self.screens().entry.sound() == MenuSound::Enter
    }

    fn highlight(&self) -> u32 {
        self.screens().loop_flags.item.get()
    }

    fn keep_action_item(&mut self, item: u32) {
        self.action_item().set(item);
    }

    fn clear_escape(&mut self) -> bool {
        clear_escape(self.screens())
    }

    fn run_item_action(&mut self) -> Result<ActionResult, Refused> {
        run_item_action(self.entry.owners, self.action_menu)
    }

    fn clear_left_by_key(&mut self) {
        self.entry.menus.tables.captions.left_by_key.set(false);
    }

    fn restore_action_item(&mut self) {
        let kept = self.action_item().get();
        self.screens().loop_flags.item.set(kept);
    }

    fn clear_answer(&mut self) {
        self.entry.menus.tables.pages.yes_no_answer.set(0u32);
    }

    fn take_key(&mut self) -> Result<(), Refused> {
        Callee::TakeKey.run(self.entry)?;
        Ok(())
    }

    fn leave_at_once(&self) -> bool {
        let menus = self.entry.menus;
        self.screen
            .leave_at_once
            .is_some_and(|flag| flag.cell(menus).is_set())
    }

    fn left_by_key(&self) -> bool {
        self.entry.menus.tables.captions.left_by_key.is_set()
    }

    fn ask_suppressed(&self) -> bool {
        let entry = self.entry;
        self.screen
            .asking
            .as_ref()
            .and_then(|asking| asking.unless)
            .is_some_and(|flag| flag(&entry.owners) == 1)
    }

    fn ask_also(&self) -> bool {
        let menus = self.entry.menus;
        self.screen
            .asking
            .as_ref()
            .and_then(|asking| asking.also_if)
            .is_some_and(|flag| flag.cell(menus).is_set())
    }

    fn confirm_leaving(&mut self) -> Result<(), Refused> {
        confirm_leaving(self.entry.owners)?;
        Ok(())
    }

    fn answered_yes(&self) -> bool {
        self.entry.menus.tables.pages.yes_no_answer.get() == 1
    }

    fn save_leave_item(&mut self) {
        let menus = self.entry.menus;
        let kept = self
            .screen
            .leave_saved_item
            .unwrap_or(self.screen.saved_item);
        ScreenVar::SavedItem(kept)
            .cell(menus)
            .set(self.screens().loop_flags.item.get());
    }
}

/// Runs the screen from `entry` to its leaving. `entry` is what the screen is entered with.
/// Returns why the screen was left, for the log.
pub fn screen_loop(_screens: &Screens, screen: &Screen, entry: &Entry) -> Result<Exit, Refused> {
    let menus = entry.menus;
    let menu = (screen.menu)(menus);
    let mut host = MenuHost {
        screen,
        entry,
        menu,
        action_menu: screen.keys.action_menu.map_or(menu, |other| other(menus)),
    };
    core_loop(&screen.shape(), &mut host)
}

/// `menu::confirm_leaving`: the question's screen (`menu::leave_question_loop`),
/// which reads nothing it is entered with.
pub fn confirm_leaving(o: Owners<'static>) -> Result<u32, Refused> {
    let mut asked = Ok(());
    let answer = {
        crate::menu::confirm_leaving::confirm_leaving(o.menus, &mut || {
            if o.system.session.flags.quitting.is_set() {
                // The window was closed: leaving is the answer. Fixed bug (`wiki/Fixed-bugs.md`,
                // the "leave?" question loops once the game is quitting).
                o.menus.tables.pages.yes_no_answer.set(1u32);
                return;
            }
            asked = run_screen(ScreenId::LeaveQuestion, &Entry::new(o)).map(|_| ());
        })
    };
    asked.map(|()| answer)
}

/// Runs a screen from the menus' tables, starting with nothing carried in.
pub fn screen_entry(o: Owners<'static>, id: ScreenId) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    let screen = id.screen();
    let name = screen
        .module_path
        .strip_prefix("rc2k_decomp::")
        .unwrap_or(screen.module_path);
    log(&format!("{name}: runs"));
    let exit = screen_loop(screens, screen, &Entry::new(o))?;
    log(&format!("{name}: has left the screen ({exit})"));
    Ok(())
}

/// A screen loop run from Rust (the event flows, `confirm_leaving`), entered with what the
/// screen reads from its caller (`Entry`: the list of cars of `select_car_loop` and
/// `car_specs_display_loop`). Returns the screen's result where it has one.
pub fn run_screen(id: ScreenId, entry: &Entry) -> Result<Option<u32>, Refused> {
    let screen = id.screen();
    let menus = entry.menus;
    let screens = &menus.screens;
    { screen_loop(screens, screen, entry) }?;
    Ok(screen.result.map(|result| result(menus)))
}

#[cfg(test)]
mod tests;
