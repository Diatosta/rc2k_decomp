//! The loop every menu screen runs, from entering the screen to leaving it. The 83 screens
//! differ only in what their `Screen` describes; each has a file of its own that holds it,
//! and this is what they share.
//!
//! Once, on entry: the screen's `before` steps.
//!
//! Set-up (again after every action that does not leave the screen): the `set_up_first`
//! steps; `left_by_key` = 0; the highlight comes back from the screen's own variable; the
//! `prepare` steps; the menu and the function lists are put in place (a list marked `run`
//! is run once, here); `option_work.menu_sound` = 0x6F and `loop_flags.leave_screen` = 0; the `background` steps;
//! one frame is drawn (`menu::draw_frame`, `menu::update_blink`, then `screens::run_frame`
//! with the screen's frame callback, before which the lists are put in place again: the
//! `first_frame_lists` if the screen has any), with the time noted in `clocks.setup_time`; the
//! `after_first_frame` steps.
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`;
//! the milliseconds since set-up noted; then `menu::handle_keys`:
//!   1  next frame
//!   2  (Escape) leave
//!   3  (Enter) the highlighted item's action runs (`menu::run_item_action`, with the
//!      highlight saved to the screen's variable before and taken back from it after).
//!      Then: leave if `loop_flags.ask_leave` or `event.event_over` is 1, or if `loop_flags.leave_screen` is 1
//!      (which is cleared); else next frame if `draw_work.action_mode` is 2; else leave if
//!      `loop_flags.event_left` is 1 (clearing `loop_flags.leave_screen`); else set up again.
//!   else leave
//! The same action check is made at the top of a frame whose `option_work.menu_sound` is 0x14D.
//! Three things a screen may have besides (`escape_leaves`, `each_frame`,
//! `short_action_check`) are described at those fields of `Screen`.
//!
//! Leaving: nothing more happens if the screen's `leave_at_once` variable is 1. On a screen
//! that asks (`confirm`), if `left_by_key` is 1,
//! `menu::confirm_leaving` puts the question; unless the answer (`g_yesNoAnswer`) is 1, the
//! `on_refusal` steps run and the screen is set up again; after a "yes" the `confirmed`
//! steps run. Otherwise the highlight is saved and the `after` steps run. Two variants: the
//! question is put as well when the `ask_also_if` variable is 1; and with
//! `unasked_skips_save`, a screen left without the question only runs its `after` steps.
//! More of the same kind are at their fields of `Screen`: `before_asking`, `ask_unless`,
//! `leaving`, `leave_saved_item`, `keys_menu`, `after_restored` and `result` (two loops
//! return a value).
//!
//! Wherever the highlight is saved or taken back, `draw_work.escape_pressed` and `loop_flags.leave_screen` are
//! cleared if `event.clears_escape_flag` and `draw_work.escape_pressed` are both 1.
//!
//! What the loop calls is called directly: its own functions (`Callee` below, what the
//! steps name), the frame callbacks, `handle_keys` and the item's action
//! (`menu::run_item_action`). A refusal goes back to the loop's caller; a set carry after a
//! `Step::CallOrFatal` is the fatal error, raised by the caller too.

use crate::audio::owner::Audio;
use crate::errors::{FatalError, Refused};
use crate::framework::log;
use crate::input::owner::Input;
use crate::input::take_key::take_key;
use crate::menu::handle_keys::handle_keys;
use crate::menu::lists_store::CarList;
use crate::menu::menus_store::MenuId;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::owner::menus;
use crate::menu::run_item_action::{ActionResult, run_item_action};
use crate::menu::run_list::run_list;
use crate::menu::update_blink::update_blink;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::scene::owner::Scene;
use crate::session::{Dword, Float};
use crate::sound::owner::Sound;
use crate::system::owner::{System, system};
pub use rc2k_core::menu::keys::MenuSound;
use rc2k_core::menu::keys::Pressed;

// ---- game code the loop calls itself ----

/// The frame routine a screen runs each frame (the original kept its code address in the
/// screen's description).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameRoutine {
    /// `menu::frame_callback` (0x474D4B): every menu screen's.
    Menu,
    /// `menu::inspection_frame` (0x46D019): the inspection area's after its first frame.
    Inspection,
}

// ---- game data ----

// ---- what differs from screen to screen ----

/// A variable of the game that a screen's description names, found when asked (a description
/// is a constant, and a constant cannot hold a reference to a static).
pub type Cell = fn(&Menus) -> &Dword;

/// A text of the game that a screen's description names (a picture's path), found when
/// asked.
pub type Text = fn(&Menus) -> &dyn crate::session::TextPlace;

/// Something a screen does at one of the loop's fixed places.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    /// Calls the game function at this RVA (one of `call`'s).
    Call(Callee),
    /// Writes a game variable.
    Write(Cell, u32),
    /// Sets the menu's sound variable.
    Sound(MenuSound),
    /// Writes a scene variable.
    WriteScene(fn(&Scene) -> &Dword, u32),
    WriteSceneF32(fn(&Scene) -> &Float, f32),
    /// Writes a variable of the front end's object lighting.
    WriteLight(fn(&crate::scene::object_light::ObjectLight) -> &Dword, u32),
    /// A cell of the front end's slot's object placing.
    WriteObject(fn(&crate::scene::object_place::ObjectPlace) -> &Dword, u32),
    /// Calls the game function, and if it fails (the original's carry flag), that is the
    /// fatal error with this code (and no message).
    CallOrFatal(Callee, u32),
    /// Something only this screen does, written out in its file.
    Do(fn(&Entry) -> Result<(), Refused>),
    /// The same for a step that can also send the loop somewhere: a name editor called once
    /// a frame that keeps the keys from the menu while a name is typed. Only looked at in
    /// `each_frame`.
    DoFlow(fn(&Entry) -> Result<Flow, Refused>),
}
/// Where a `Step::DoFlow` sends the loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Nowhere: the loop goes on with what comes next.
    On,
    /// To the top of the loop, without the action check and the keys.
    NextFrame,
    Leave,
    SetUp,
}

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
    pub also_if: Option<Cell>,
    /// A variable that, when it is 1, keeps the question from being put although
    /// `captions.left_by_key` is 1.
    pub unless: Option<fn(&Menus) -> u32>,
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

/// What the loop looks at after a key opened the highlighted item's action.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionCheck {
    /// Leave if `loop_flags.ask_leave`, `event.event_over` or `loop_flags.leave_screen` is 1
    /// (the last is cleared); else next frame if `draw_work.action_mode` is 2; else leave if
    /// `loop_flags.event_left` is 1; else set up again.
    Standard,
    /// Only `loop_flags.ask_leave` and `loop_flags.leave_screen` are looked at; unless the screen
    /// is left it is set up again, whatever `draw_work.action_mode` and `loop_flags.event_left`
    /// hold.
    Short,
    /// Left when `loop_flags.ask_leave` is 1; when `event.event_over` is 1 the answer `g_yesNoAnswer` is
    /// cleared, a key taken and the screen set up again; left when `loop_flags.leave_screen` is 1, which
    /// stays 1; otherwise `option_work.menu_sound` = 0x6F and set up again.
    Restart,
}

/// What differs in how a screen takes its keys and their actions.
#[derive(Clone, Copy)]
pub struct Keys {
    /// The menu stored in `menu_texts.keys_menu` before `menu::handle_keys`, where it is not `menu`.
    pub menu: Option<fn(&Menus) -> MenuId>,
    /// The Escape flag `draw_work.escape_pressed` is looked at at the top of every frame, before the quit
    /// switch, and the screen is left when it is 1.
    pub escape_leaves: bool,
    /// What is looked at after an action.
    pub action_check: ActionCheck,
    /// The variable the highlight is kept in around an item's action, where it is not
    /// `saved_item`.
    pub action_saved_item: Option<Cell>,
    /// The menu whose item's action is run, where it is not `menu`: three loops name a
    /// neighbouring screen's menu there (the original's slip, reproduced).
    pub action_menu: Option<fn(&Menus) -> MenuId>,
}

impl Keys {
    /// The screen's own menu everywhere, Escape not looked at, the standard action check.
    pub const DEFAULT: Keys = Keys {
        menu: None,
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
    /// same as before every other frame: 35 loops put their 3D list in `g_unreadList` at
    /// set-up and before each frame, and a list of their own in `g_sceneList` only here.
    pub first_frame_lists: Option<&'static [List]>,
    /// "Next frame" (after `handle_keys` returns 1, and after an action that leaves 2 in
    /// `draw_work.action_mode`) goes to the `after_first_frame` steps, not past them. A `Flow` those
    /// steps return is honoured on every screen.
    pub next_frame_repeats_first: bool,
    /// Steps before the first frame after set-up.
    pub before_first_frame: &'static [Step],
    /// Steps before every other frame.
    pub before_frame: &'static [Step],
    /// Steps right after `update_blink` at the top of a frame; a `Flow` is honoured.
    pub after_blink: &'static [Step],
    /// Steps of the screen's own once a frame, after the frame and before the keys.
    pub each_frame: &'static [Step],
    /// The quit switch is not looked at; `option_work.menu_sound` = 0x6F is stored once a frame in
    /// that place.
    pub no_quit_check: bool,
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
        no_quit_check: false,
    };
}

/// What one screen is. A screen is `Screen::new(menu, saved_item)` and the fields that differ.
#[derive(Clone, Copy)]
pub struct Screen {
    /// off_...: the screen's menu.
    pub menu: fn(&Menus) -> MenuId,
    /// The variable the highlight is kept in between visits.
    pub saved_item: Cell,
    pub lists: &'static [List],
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
    pub leave_at_once: Option<Cell>,
    /// On a screen that does not ask: what runs first on leaving, before the highlight is
    /// saved (the mode menu calls `menu::go_back` there when `captions.left_by_key` is 1).
    pub leaving: &'static [Step],
    /// The variable the highlight is saved in on leaving, where it is not `saved_item`.
    pub leave_saved_item: Option<Cell>,
    /// What runs last of all, with the carried values back as they were at entry.
    pub after_restored: &'static [Step],
    /// What the loop returns, where it returns something: read from the game's
    /// variables as the loop leaves them.
    pub result: Option<fn(&Menus) -> u32>,
    /// `captions.left_by_key` is cleared and the highlight put in place (with the Escape check)
    /// after the first frame, not at the start of set-up, and set-up stores neither
    /// `option_work.menu_sound` nor `loop_flags.leave_screen` after the lists: the main menu.
    pub late_highlight: bool,
    pub keys: Keys,
    pub frames: Frames,
}

impl Screen {
    /// A screen with its menu and the variable its highlight is kept in, and every other
    /// field off or empty; a screen writes what differs with `..Screen::new(..)`.
    pub const fn new(menu: fn(&Menus) -> MenuId, saved_item: Cell) -> Screen {
        Screen {
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
pub struct Entry<'a> {
    pub menus: &'a Menus,
    pub cars: Option<CarList>,
    /// The other owners the steps and the loop use (fetched once, where the entry is made).
    pub scene: &'a Scene,
    /// The front end's slot of polygon work (its window is the screens' view).
    pub front_end: &'a crate::scene::polygon_work::PolygonWork,
    pub system: &'a System,
    pub race: &'a Race,
    pub sound: &'a Sound,
    pub input: &'a Input,
    pub audio: &'a Audio,
}

impl<'a> Entry<'a> {
    /// The view's offset and zoom a front-end screen draws with.
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

    /// Entered with nothing carried in.
    pub fn new(menus: &'a Menus) -> Self {
        Entry::with_cars(menus, None)
    }
    /// Entered with the list of cars the carousel shows.
    pub fn with_cars(menus: &'a Menus, cars: Option<CarList>) -> Self {
        Entry {
            menus,
            cars,
            scene: crate::scene::owner::scene(),
            front_end: crate::scene::polygon_work::front_end(),
            system: system(),
            race: race(),
            sound: crate::sound::owner::sound(),
            input: crate::input::owner::input(),
            audio: crate::audio::owner::audio(),
        }
    }
}

/// Whether the game is quitting.
pub fn quitting() -> bool {
    system().session.flags.quitting.get() == 1
}

/// `scene::prepare_menu_scene` (0x46F7FD). (It leaves esi the view's turn matrix: a screen
/// whose carousel is set up after it hands that no list of cars.)
pub fn prepare_scene(scene: &Scene) {
    crate::scene::prepare_menu_scene::prepare_menu_scene(scene);
}

/// `scene::set_up_carousel` (0x4904DD), with the list of cars menus 0xD and 0xE show.
/// (`_scene`: the loops pass it; the carousel's set-up writes the front end's slot.)
pub fn carousel(cars: Option<CarList>) -> Result<(), Refused> {
    crate::scene::set_up_carousel::set_up_carousel(cars)
}

/// A line for the game's log at level 1 with its line break (`logfile::log_write_l1_nl`,
/// changes no register).
pub fn log_line(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 1);
}

/// A game function a screen step runs (by its Rust version), with what it is handed. The
/// comment on each variant is its address in the original.
#[derive(Clone, Copy, Debug)]
pub enum Callee {
    /// 0x407F95
    TakeKey,
    /// 0x471391 and 0x4713DD (nullsub_83, nullsub_84: a lone `retn`)
    NoOp,
    /// 0x47EFFF, with the picture's path.
    LoadBackground(Text),
    /// 0x46F7FD
    PrepareScene,
    /// 0x46B47C
    ApplyController,
    /// 0x46B4B4
    ReleaseController,
    /// 0x4683CF
    DrawStageMap,
    /// 0x40F757
    StopStream,
    /// 0x40F792
    PlayTune4,
    /// 0x40F7A4
    PlayTune9,
    /// 0x40F826
    StopMusic,
    /// 0x46F7AD
    StepCarStream,
    /// 0x469AA1
    PlayCarStream,
    /// 0x40939F, with the axis.
    AxisMoved(u32),
    /// 0x48340B
    RankStageTimes,
    /// 0x48356F
    RankTotals,
    /// 0x485CBB
    ScoreFourPlayers,
    /// 0x485B0E
    ScoreTwoPlayers,
    /// 0x470F9F
    StyleStageItem,
    /// 0x470FC3
    StyleChosenStageItem,
    /// 0x480032
    CountStagePages,
    /// 0x47F437
    PlayStageSpeech,
    /// 0x47F40F
    PlayRallySpeech,
    /// 0x4684BD
    LoadWelcomePicture,
    /// 0x470F55
    LoadServiceBackground,
    /// 0x471C82
    GoBack,
    /// 0x486C21
    MarkStageChoice,
    /// 0x467C63
    EditPlayerName,
    /// 0x46F1A9
    EditSetupName,
    /// 0x46D685
    EditSavegameName,
    /// 0x46F4E7
    EditReplayName,
    /// 0x480CBF
    CheckCheatName,
    /// 0x4684F5
    LoadSlideshowPicture,
    /// 0x468482
    LoadCarBackground,
    /// 0x411863, with the picture's path.
    LoadPicture(Text),
    /// 0x406126
    ConvertPicture,
    /// 0x47C63F
    SetUpFrontEnd,
    /// 0x442AF6
    SetUpEngine,
    /// 0x474E04
    SetUpFrontEndScene,
}

impl Callee {
    /// Runs the function: whether it failed (the original's carry flag; false for those that
    /// do not set it).
    pub fn run(self, entry: &Entry) -> Result<bool, Refused> {
        let menus = entry.menus;
        let (audio, input, sound) = (entry.audio, entry.input, entry.sound);
        let play = &menus.play;
        let screens = &menus.screens;
        use crate::menu::{controller_items as ci, results_setups as rs, stage_view_steps as v};
        use crate::text::draw_text::GameCanvas;
        match self {
            Self::TakeKey => {
                take_key();
            }
            Self::NoOp => {}
            // Its carry is never set.
            Self::LoadBackground(path) => {
                crate::menu::load_background::load_background(path(menus), &mut GameCanvas)?;
            }
            Self::PrepareScene => prepare_scene(entry.scene),
            Self::ApplyController => ci::apply_controller(menus)?,
            Self::ReleaseController => ci::release_controller(input),
            Self::DrawStageMap => crate::menu::stage_choice::draw_stage_map(menus)?,
            Self::StopStream => crate::sound::stop_stream::stop_stream(),
            Self::PlayTune4 => crate::sound::play_tune_4::play_tune_4(),
            Self::PlayTune9 => crate::sound::play_tune_9::play_tune_9(),
            Self::StopMusic => crate::sound::stop_music::stop_music(),
            Self::StepCarStream => {
                crate::sound::step_car_stream::step_car_stream_in_game(sound, audio)?;
            }
            Self::PlayCarStream => crate::sound::play_car_stream::play_car_stream(
                &mut crate::sound::start_stream::start_stream,
                sound,
            )?,
            Self::AxisMoved(axis) => {
                crate::input::axes::axis_moved(axis, input)?;
            }
            Self::RankStageTimes => rs::rank_stage_times(menus)?,
            Self::RankTotals => rs::rank_totals(menus)?,
            Self::ScoreFourPlayers => rs::score_four_players(menus)?,
            Self::ScoreTwoPlayers => rs::score_two_players(menus)?,
            Self::StyleStageItem => v::style_stage_item(screens, false)?,
            Self::StyleChosenStageItem => v::style_stage_item(screens, true)?,
            Self::CountStagePages => v::count_stage_pages(screens)?,
            Self::PlayStageSpeech => v::play_stage_speech(menus, sound)?,
            Self::PlayRallySpeech => v::play_rally_speech(menus)?,
            Self::LoadWelcomePicture => v::load_welcome_picture(menus)?,
            Self::LoadServiceBackground => crate::service::load_background::load_background()?,
            Self::GoBack => crate::menu::go_back::go_back(),
            Self::MarkStageChoice => crate::menu::mark_stage_choice::mark_stage_choice(play),
            Self::EditPlayerName => {
                crate::menu::edit_player_name::edit_player_name(menus, screens, &mut GameCanvas)?;
            }
            Self::EditSetupName => {
                crate::service::edit_setup_name::edit_setup_name(menus, &mut GameCanvas)?;
            }
            Self::EditSavegameName => crate::menu::edit_savegame_name::edit_savegame_name(
                menus,
                &mut GameCanvas,
                &mut crate::menu::edit_savegame_name::save_game,
            )?,
            Self::EditReplayName => {
                return crate::menu::edit_replay_name::edit_replay_name(menus, &mut GameCanvas);
            }
            Self::CheckCheatName => crate::menu::check_cheat_name::check_cheat_name(menus)?,
            // The way to look is the original's eax there, which the slideshow screen's set-up
            // leaves 0 or (after an action) a small number: always on.
            Self::LoadSlideshowPicture => {
                crate::menu::load_slideshow_picture::load_slideshow_picture_in_game(menus, 0)?;
            }
            // Its carry is never set.
            Self::LoadCarBackground => {
                crate::menu::load_car_background::load_car_background_in_game(menus)?;
            }
            Self::LoadPicture(path) => {
                return Ok(!crate::gfx::load_picture::load_picture(path(menus))?);
            }
            Self::ConvertPicture => return Ok(crate::gfx::convert_picture::convert_picture_now()?),
            Self::SetUpFrontEnd => crate::menu::set_up_front_end::set_up_front_end(menus, sound)?,
            Self::SetUpEngine => crate::scene::set_up_engine::set_up_engine()?,
            Self::SetUpFrontEndScene => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
            }
        }
        Ok(false)
    }
}

/// Where the loop goes next.
#[derive(Clone, Copy)]
enum Next {
    SetUp,
    FrameTop,
    Frame,
    ActionCheck,
    Leave,
}

/// Clears the Escape flags if they are set. The original clears them only when
/// `g_clearsEscapeFlag` (0x7558D8) is 1, which only four screens set (start rally, options,
/// player 1 details, information) and nothing clears: before one of them has been seen, a
/// "no" given with Escape to a leave question left the flag set, and the screen below left
/// at its next frame (a championship loaded at start-up ended at its review screen; seen
/// 2026-10-24). Fixed: cleared as once any of those screens has been seen
/// (`wiki/Fixed-bugs.md`).
/// Returns whether it was set (and so `leave_screen` was cleared with it).
fn clear_escape(screens: &Screens) -> bool {
    let pressed = screens.draw_work.escape_pressed.get() == 1;
    if pressed {
        screens.draw_work.escape_pressed.set(0u32);
        screens.loop_flags.leave_screen.set(0u32);
    }
    pressed
}

/// A call whose failure is the fatal error `code`.
pub fn call_or_fatal(entry: &Entry, function: Callee, code: u32) -> Result<(), Refused> {
    if { function.run(entry) }? {
        log(&format!(
            "a screen loop's Rust version raises fatal error {code:#x}"
        ));
        return Err(FatalError { code, message: "" }.into());
    }
    Ok(())
}

/// Runs the steps of one place. Anything but `Flow::On` comes from a `DoFlow` step, after
/// which the rest of the place is not run.
fn run(steps: &[Step], entry: &Entry) -> Result<Flow, Refused> {
    let menus = entry.menus;
    for step in steps {
        match *step {
            Step::DoFlow(what) => match what(entry)? {
                Flow::On => {}
                elsewhere => return Ok(elsewhere),
            },
            Step::Call(function) => {
                function.run(entry)?;
            }
            Step::Write(variable, value) => variable(menus).set(value),
            Step::Sound(sound) => menus.screens.option_work.set_sound(sound),
            Step::WriteScene(variable, value) => variable(entry.scene).set(value),
            Step::WriteObject(variable, value) => {
                variable(&crate::race::view_scratch::front_end().object).set(value);
            }
            Step::WriteSceneF32(variable, value) => variable(entry.scene).set(value),
            Step::WriteLight(variable, value) => {
                variable(&crate::race::view_scratch::front_end().light).set(value);
            }
            Step::CallOrFatal(function, code) => call_or_fatal(entry, function, code)?,
            Step::Do(what) => what(entry)?,
        }
    }
    Ok(Flow::On)
}

/// Puts the menu and the lists in place, as the original does before a frame (the esi it
/// leaves at the last list's table is read by no one: `cars` is set before its reader).
fn set_lists(menus: &Menus, screens: &Screens, screen: &Screen, lists: &[List]) {
    screens.loop_flags.menu.set((screen.menu)(menus));
    for list in lists.iter().filter(|list| !list.run) {
        (list.pointer)(menus).set((list.table)(menus));
    }
}

/// `screens::run_frame` (0x405A71) with the screen's frame callback: the messages, then
/// (unless quitting) the frame, with the menu and the carried palette and x. Neither
/// changes a carried value (the callbacks keep every register; the carry they leave is
/// read by nothing).
fn run_frame(menus: &Menus, callback: FrameRoutine, menu: MenuId) -> Result<(), Refused> {
    let frame = || match callback {
        FrameRoutine::Menu => crate::menu::frame_callback::frame_callback(menus, menu),
        FrameRoutine::Inspection => crate::menu::inspection_frame::inspection_frame(menus, menu),
    };
    match crate::platform::run_frame_now(frame) {
        Some(made) => made.map(|_| ()),
        None => Ok(()),
    }
}

/// `entry` is what the screen is entered with.
/// Returns why the screen was left, for the log.
pub fn screen_loop(screens: &Screens, screen: &Screen, entry: &Entry) -> Result<Exit, Refused> {
    let menus = entry.menus;
    let menu = (screen.menu)(menus);
    let keys_menu = screen.keys.menu.map_or(menu, |other| other(menus));
    run(screen.before, entry)?;
    let mut state = Loop {
        screen,
        entry,
        menu,
        callback: screen.frames.callback,
        later_callback: screen
            .frames
            .later_callback
            .unwrap_or(screen.frames.callback),
        keys_menu,
        action_menu: screen.keys.action_menu.map_or(menu, |other| other(menus)),
        next_frame: if screen.frames.next_frame_repeats_first {
            Next::FrameTop
        } else {
            Next::Frame
        },
        frames: 0,
        actions: 0,
        why: ScreenExit::Escape,
    };
    let mut next = Next::SetUp;
    loop {
        next = match next {
            Next::SetUp => state.set_up()?,
            Next::FrameTop => state.after_first_frame()?,
            Next::Frame => state.frame()?,
            Next::ActionCheck if screens.option_work.sound() != MenuSound::Enter => state.keys()?,
            Next::ActionCheck => state.action()?,
            Next::Leave => match state.leave()? {
                Left::Done(exit) => return Ok(exit),
                Left::SetUp => Next::SetUp,
            },
        };
    }
}

/// Why an action's result sent the loop out of the screen (read from the flags the action and
/// the leaving functions left: `ask_leave`, `event_over`, `leave_screen`, `event_left`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionExit {
    /// `ask_leave` was 1 (and `event_over`, as `event_over`, on a standard check).
    Asked { ask_leave: u32, event_over: u32 },
    /// `ask_leave` was 1 on a restarting check.
    AskedRestart,
    /// `leave_screen` was 1 (cleared by a standard check, kept by a restarting one).
    LeaveScreen { cleared: bool },
    /// `event_left` was 1 after an action that stays by default (`leave_screen` cleared).
    EventLeft,
}

/// Why a screen was left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenExit {
    /// Escape was taken from the keys.
    Escape,
    /// `escape_pressed` was 1 at the top of a frame (`escape_leaves`).
    EscapeFlag,
    /// A step of the screen's own sent the loop out.
    OwnStep,
    /// After the action of the highlighted item.
    Action { item: u32, how: ActionExit },
}

impl std::fmt::Display for ScreenExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ScreenExit::Escape => write!(f, "Escape"),
            ScreenExit::EscapeFlag => write!(f, "the Escape flag was set"),
            ScreenExit::OwnStep => write!(f, "a step of the screen's own"),
            ScreenExit::Action { item, how } => {
                write!(f, "after the action of item {item}: ")?;
                match how {
                    ActionExit::Asked {
                        ask_leave,
                        event_over,
                    } => write!(f, "ask_leave={ask_leave} event_over={event_over}"),
                    ActionExit::AskedRestart => write!(f, "ask_leave was 1"),
                    ActionExit::LeaveScreen { cleared: true } => write!(f, "leave_screen was 1"),
                    ActionExit::LeaveScreen { cleared: false } => write!(f, "leave_screen is 1"),
                    ActionExit::EventLeft => write!(f, "event_left is 1"),
                }
            }
        }
    }
}

/// How a screen loop ended: why, and how long it ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit {
    pub reason: ScreenExit,
    pub frames: u32,
    pub actions: u32,
}

impl std::fmt::Display for Exit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; {} frames, {} actions",
            self.reason, self.frames, self.actions
        )
    }
}

/// What the loop of one screen carries from step to step.
struct Loop<'a> {
    screen: &'a Screen,
    entry: &'a Entry<'a>,
    menu: MenuId,
    callback: FrameRoutine,
    later_callback: FrameRoutine,
    keys_menu: MenuId,
    action_menu: MenuId,
    /// Where the loop goes after a frame without a key: the top (`next_frame_repeats_first`)
    /// or the next frame.
    next_frame: Next,
    frames: u32,
    actions: u32,
    /// Why the screen is being left, for the log.
    why: ScreenExit,
}

/// How leaving ended.
enum Left {
    /// The screen is left, for the reason given.
    Done(Exit),
    /// The question was answered "no": the screen is set up again.
    SetUp,
}

impl Loop<'_> {
    /// Set-up, again after every action that does not leave the screen: the lists put in place
    /// and the first frame drawn.
    fn set_up(&mut self) -> Result<Next, Refused> {
        let system = self.entry.system;
        let menus = self.entry.menus;
        let screens = &menus.screens;
        let screen = self.screen;
        let entry = self.entry;
        let menu = self.menu;
        run(screen.set_up_first, entry)?;
        if !screen.late_highlight {
            menus.tables.captions.left_by_key.set(0);
            menus
                .screens
                .loop_flags
                .item
                .set((screen.saved_item)(menus).get());
            clear_escape(screens);
        }
        run(screen.prepare, entry)?;

        menus.screens.loop_flags.menu.set(menu);
        for list in screen.lists {
            let table = (list.table)(menus);
            if list.run {
                run_list(menus, table)?;
            }
            (list.pointer)(menus).set(table);
        }
        run(screen.after_lists, entry)?;
        if !screen.late_highlight {
            menus.screens.option_work.set_sound(MenuSound::None);
            menus.screens.loop_flags.leave_screen.set(0u32);
        }
        run(screen.background, entry)?;
        crate::menu::draw_frame::draw_frame(menus, menu)?;
        update_blink(menus);
        system
            .session
            .clocks
            .setup_time
            .set(system.session.clocks.frame.get());
        self.entry.race.shared.physics_tables.scene_clock.set(0u32);
        run(screen.frames.before_first_frame, entry)?;
        set_lists(
            menus,
            screens,
            screen,
            screen.frames.first_frame_lists.unwrap_or(screen.lists),
        );
        run_frame(menus, self.callback, menu)?;
        if screen.late_highlight {
            menus.tables.captions.left_by_key.set(0);
            menus
                .screens
                .loop_flags
                .item
                .set((screen.saved_item)(menus).get());
            clear_escape(screens);
        }
        Ok(Next::FrameTop)
    }

    /// The `after_first_frame` steps, once the first frame is drawn.
    fn after_first_frame(&mut self) -> Result<Next, Refused> {
        let screen = self.screen;
        let entry = self.entry;
        Ok({
            match run(screen.after_first_frame, entry)? {
                Flow::On | Flow::NextFrame => Next::Frame,
                Flow::SetUp => Next::SetUp,
                Flow::Leave => {
                    self.why = ScreenExit::OwnStep;
                    Next::Leave
                }
            }
        })
    }

    /// One frame: blink, quit and Escape checks, the frame self.callback and the per-frame steps.
    fn frame(&mut self) -> Result<Next, Refused> {
        let system = self.entry.system;
        let menus = self.entry.menus;
        let screens = &menus.screens;
        let screen = self.screen;
        let entry = self.entry;
        let menu = self.menu;
        Ok({
            update_blink(menus);
            self.frames += 1;
            let flow = run(screen.frames.after_blink, entry)?;
            if flow == Flow::SetUp {
                Next::SetUp
            } else if flow == Flow::Leave {
                self.why = ScreenExit::OwnStep;
                Next::Leave
            } else if screen.keys.escape_leaves && screens.draw_work.escape_pressed.get() == 1 {
                self.why = ScreenExit::EscapeFlag;
                Next::Leave
            } else if !screen.frames.no_quit_check && quitting() {
                // At once: a screen left as if chosen lets the flow behind it go on with
                // what the screen never set (a championship's next stage: a fatal error).
                crate::standalone::leave_on_close();
            } else if crate::debug::screens::serve()? {
                Next::SetUp
            } else {
                if screen.frames.no_quit_check {
                    screens.option_work.set_sound(MenuSound::None);
                }
                run(screen.frames.before_frame, entry)?;
                set_lists(menus, screens, screen, screen.lists);
                run_frame(menus, self.later_callback, menu)?;
                self.entry.race.shared.physics_tables.scene_clock.set(
                    system
                        .session
                        .clocks
                        .frame
                        .get()
                        .wrapping_sub(system.session.clocks.setup_time.get()),
                );
                match run(screen.frames.each_frame, entry)? {
                    Flow::On => Next::ActionCheck,
                    Flow::NextFrame => Next::Frame,
                    Flow::SetUp => Next::SetUp,
                    Flow::Leave => {
                        self.why = ScreenExit::OwnStep;
                        Next::Leave
                    }
                }
            }
        })
    }

    /// The keys of a frame (`menu::handle_keys`) and where they lead.
    fn keys(&mut self) -> Result<Next, Refused> {
        let menus = self.entry.menus;
        Ok({
            menus.tables.texts.keys_menu.set(self.keys_menu.code());
            // The original hands it the time since set-up in eax (the frame's last
            // write there that a screen's steps do not change before it reads it).
            match handle_keys(
                menus,
                self.entry.race.shared.physics_tables.scene_clock.get(),
            )? {
                Pressed::Nothing => self.next_frame,
                Pressed::Enter => Next::ActionCheck,
                Pressed::Escape => {
                    self.why = ScreenExit::Escape;
                    Next::Leave
                }
            }
        })
    }

    /// The highlighted item's action, and what it leaves: leave, next frame or set up again.
    fn action(&mut self) -> Result<Next, Refused> {
        let menus = self.entry.menus;
        let screens = &menus.screens;
        let screen = self.screen;
        Ok({
            self.actions += 1;
            let item = menus.screens.loop_flags.item.get();
            let kept = screen.keys.action_saved_item.unwrap_or(screen.saved_item);
            kept(menus).set(item);
            clear_escape(screens);
            let mut result = run_item_action(menus, self.action_menu)?;
            menus.tables.captions.left_by_key.set(0);
            menus.screens.loop_flags.item.set(kept(menus).get());
            if clear_escape(screens) {
                result.leave_screen = false;
            }
            match screen.keys.action_check {
                ActionCheck::Restart => self.action_restart(item, result)?,
                check => self.action_checked(item, check == ActionCheck::Short, result),
            }
        })
    }

    /// The action check of `ActionCheck::Restart`: the flags the action left, in the
    /// original's order, say where the loop goes.
    fn action_restart(&mut self, item: u32, result: ActionResult) -> Result<Next, Refused> {
        let menus = self.entry.menus;
        let leave = |how| ScreenExit::Action { item, how };
        Ok(if result.ask_leave == 1 {
            self.why = leave(ActionExit::AskedRestart);
            Next::Leave
        } else if result.event_over == 1 {
            menus.tables.pages.yes_no_answer.set(0u32);
            Callee::TakeKey.run(self.entry)?;
            Next::SetUp
        } else if result.leave_screen {
            self.why = leave(ActionExit::LeaveScreen { cleared: false });
            Next::Leave
        } else {
            menus.screens.option_work.set_sound(MenuSound::None);
            Next::SetUp
        })
    }

    /// The action check of `ActionCheck::Standard` (`short`: of `ActionCheck::Short`).
    fn action_checked(&mut self, item: u32, short: bool, result: ActionResult) -> Next {
        let menus = self.entry.menus;
        let flags = &menus.screens.loop_flags;
        let ask_leave = result.ask_leave;
        let event_over = result.event_over;
        let leave = |how| ScreenExit::Action { item, how };
        if ask_leave == 1 || (!short && event_over == 1) {
            self.why = leave(ActionExit::Asked {
                ask_leave,
                event_over,
            });
            Next::Leave
        } else if result.leave_screen {
            flags.leave_screen.set(0u32);
            self.why = leave(ActionExit::LeaveScreen { cleared: true });
            Next::Leave
        } else if short {
            Next::SetUp
        } else {
            menus.screens.option_work.set_sound(MenuSound::None);
            if result.action_mode == 2 {
                self.next_frame
            } else if result.event_left {
                flags.leave_screen.set(0u32);
                self.why = leave(ActionExit::EventLeft);
                Next::Leave
            } else {
                Next::SetUp
            }
        }
    }

    /// Leaving: the question when the screen asks, then the `after` steps.
    fn leave(&mut self) -> Result<Left, Refused> {
        let menus = self.entry.menus;
        let screens = &menus.screens;
        let screen = self.screen;
        let entry = self.entry;
        let done = || Exit {
            reason: self.why,
            frames: self.frames,
            actions: self.actions,
        };
        if screen
            .leave_at_once
            .is_some_and(|flag| flag(menus).get() == 1)
        {
            return Ok(Left::Done(done()));
        }
        if let Some(asking) = &screen.asking {
            let asked = asking.always
                || (menus.tables.captions.left_by_key.get() == 1
                    && !asking.unless.is_some_and(|flag| flag(menus) == 1))
                || asking.also_if.is_some_and(|flag| flag(menus).get() == 1);
            if asked {
                run(asking.before, entry)?;
                confirm_leaving()?;
                if menus.tables.pages.yes_no_answer.get() != 1 {
                    run(asking.on_refusal, entry)?;
                    return Ok(Left::SetUp);
                }
                run(asking.confirmed, entry)?;
                if asking.always {
                    return Ok(Left::Done(done()));
                }
            } else if asking.unasked_skips_save {
                run(screen.after, entry)?;
                run(screen.after_restored, entry)?;
                return Ok(Left::Done(done()));
            }
        } else {
            run(screen.leaving, entry)?;
        }
        screen.leave_saved_item.unwrap_or(screen.saved_item)(menus)
            .set(menus.screens.loop_flags.item.get());
        clear_escape(screens);
        run(screen.after, entry)?;
        run(screen.after_restored, entry)?;
        Ok(Left::Done(done()))
    }
}

/// `menu::confirm_leaving` (0x46FA90): the question's screen (`menu::leave_question_loop`),
/// which reads nothing it is entered with; every register comes back as it was.
pub fn confirm_leaving() -> Result<u32, Refused> {
    let mut asked = Ok(());
    let answer = {
        crate::menu::confirm_leaving::confirm_leaving(&mut || {
            if quitting() {
                // The window was closed: leaving is the answer. (The original asked; the
                // question left at once on the switch with no answer, and the screen asked
                // again forever, 0x474B43 / 0x47FB9A. Fixed bug.)
                menus().tables.pages.yes_no_answer.set(1u32);
                return;
            }
            asked = run_screen(
                &crate::menu::leave_question_loop::SCREEN,
                &Entry::new(menus()),
            )
            .map(|_| ());
        })
    };
    asked.map(|()| answer)
}

// ---- entries ----

/// Runs a screen from the menus' tables, starting with nothing carried in. Rust callers use
/// `run_screen`.
pub fn screen_entry(name: &str, screen: &Screen) -> Result<(), Refused> {
    let screens = &menus().screens;
    log(&format!("{name}: the Rust version runs"));
    let exit = screen_loop(screens, screen, &Entry::new(menus()))?;
    log(&format!(
        "{name}: the Rust version has left the screen ({exit})"
    ));
    Ok(())
}

/// A screen loop run from Rust (the event flows, `confirm_leaving`), entered with what the
/// screen reads from its caller (`Entry`: the list of cars of `select_car_loop` and
/// `car_specs_display_loop`). Returns the screen's result where it has one (the original's
/// eax).
pub fn run_screen(screen: &Screen, entry: &Entry) -> Result<Option<u32>, Refused> {
    let menus = entry.menus;
    let screens = &menus.screens;
    { screen_loop(screens, screen, entry) }?;
    Ok(screen.result.map(|result| result(menus)))
}
