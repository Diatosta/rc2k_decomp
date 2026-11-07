//! The loop every menu screen runs, from entering the screen to leaving it. The 83 screens
//! differ only in what their `Screen` describes; each has a file of its own that holds it,
//! and this is what they share.
//!
//! Once, on entry: the screen's `before` steps.
//!
//! Set-up (again after every action that does not leave the screen): the `set_up_first`
//! steps; `dword_758806` = 0; the highlight comes back from the screen's own variable; the
//! `prepare` steps; the menu and the function lists are put in place (a list marked `run`
//! is run once, here); `dword_7556A4` = 0x6F and `dword_755561` = 0; the `background` steps;
//! one frame is drawn (`menu::draw_frame`, `menu::update_blink`, then `screens::run_frame`
//! with the screen's frame callback, before which the lists are put in place again: the
//! `first_frame_lists` if the screen has any), with the time noted in `dword_5189A0`; the
//! `after_first_frame` steps.
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`;
//! `dword_613830` = milliseconds since set-up; then `menu::handle_keys`:
//!   1  next frame
//!   2  (Escape) leave
//!   3  (Enter) the highlighted item's action runs (`menu::run_item_action`, with the
//!      highlight saved to the screen's variable before and taken back from it after).
//!      Then: leave if `dword_755569` or `dword_755980` is 1, or if `dword_755561` is 1
//!      (which is cleared); else next frame if `dword_755A38` is 2; else leave if
//!      `dword_75555D` is 1 (clearing `dword_755561`); else set up again.
//!   else leave
//! The same action check is made at the top of a frame whose `dword_7556A4` is 0x14D.
//! Three things a screen may have besides (`escape_leaves`, `each_frame`,
//! `short_action_check`) are described at those fields of `Screen`.
//!
//! Leaving: nothing more happens if the screen's `leave_at_once` variable is 1. On a screen
//! that asks (`confirm`), if `dword_758806` is 1,
//! `menu::confirm_leaving` puts the question; unless the answer (`g_yesNoAnswer`) is 1, the
//! `on_refusal` steps run and the screen is set up again; after a "yes" the `confirmed`
//! steps run. Otherwise the highlight is saved and the `after` steps run. Two variants: the
//! question is put as well when the `ask_also_if` variable is 1; and with
//! `unasked_skips_save`, a screen left without the question only runs its `after` steps.
//! More of the same kind are at their fields of `Screen`: `before_asking`, `ask_unless`,
//! `leaving`, `leave_saved_item`, `keys_menu`, `after_restored` and `result` (two loops
//! return a value in eax).
//!
//! Wherever the highlight is saved or taken back, `dword_755A3C` and `dword_755561` are
//! cleared if `dword_7558D8` and `dword_755A3C` are both 1.
//!
//! What the loop calls is called directly: its own functions (`Callee` below, what the
//! steps name), the frame callbacks, `handle_keys` and the item's action
//! (`menu::run_item_action`). A refusal goes back to the loop's caller; a set carry after a
//! `Step::CallOrFatal` is the fatal error, raised by the caller too.

use crate::errors::{FatalError, Refused};
use crate::framework::log;
use crate::input::take_key::take_key;
use crate::menu::handle_keys::handle_keys;
use crate::menu::lists_store::CarList;
use crate::menu::menus_store::MenuId;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::owner::menus;
use crate::menu::run_item_action::run_item_action;
use crate::menu::run_list::run_list;
use crate::menu::update_blink::update_blink;
use crate::race::owner::race;
use crate::session::{Dword, Float};
use crate::system::owner::system;
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

const NO_SOUND: u32 = 0x6F;
const ENTER_SOUND: u32 = 0x14D;

// ---- what differs from screen to screen ----

/// A variable of the game that a screen's description names, found when asked (a description
/// is a constant, and a constant cannot hold a reference to a static).
pub type Cell = fn(&'static Menus) -> &'static Dword;

/// A text of the game that a screen's description names (a picture's path), found when
/// asked.
pub type Text = fn(&'static Menus) -> &'static dyn crate::session::TextPlace;

/// Something a screen does at one of the loop's fixed places.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    /// Calls the game function at this RVA (one of `call`'s).
    Call(Callee),
    /// Writes a game variable.
    Write(Cell, u32),
    WriteF32(fn(&'static Menus) -> &'static Float, f32),
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
    pub pointer: fn(&'static Menus) -> &'static crate::menu::fn_list::ListCell,
    pub table: fn(&'static Menus) -> crate::menu::fn_list::FnList,
    /// Run with `menu::run_list` at set-up, before its pointer is stored. Such a list is not
    /// put in place again before each frame; the others are.
    pub run: bool,
}

pub struct Screen {
    /// off_...: the screen's menu.
    pub menu: fn(&'static Menus) -> MenuId,
    /// The variable the highlight is kept in between visits.
    pub saved_item: Cell,
    pub lists: &'static [List],
    /// The lists put in place before the first frame after set-up, where they are not the
    /// same as before every other frame: 35 loops put their 3D list in `g_unreadList` at
    /// set-up and before each frame, and a list of their own in `g_sceneList` only here.
    pub first_frame_lists: Option<&'static [List]>,
    pub frame_callback: FrameRoutine,
    /// The frame callback of every frame but the first after set-up, where it is another
    /// one: the inspection area's loop is the only one.
    pub later_frame_callback: Option<FrameRoutine>,
    pub before: &'static [Step],
    pub set_up_first: &'static [Step],
    pub prepare: &'static [Step],
    /// Steps between putting the lists in place and the background.
    pub after_lists: &'static [Step],
    pub background: &'static [Step],
    pub after_first_frame: &'static [Step],
    /// Some if leaving is asked about: what runs after a "no".
    pub confirm: Option<&'static [Step]>,
    /// A variable that, when it is 1, has the question put as well (`dword_755569` in the
    /// menus after the results screens).
    pub ask_also_if: Option<Cell>,
    /// What runs after a "yes".
    pub confirmed: &'static [Step],
    /// When the question is not put, the highlight is not saved and the Escape flag is left
    /// alone: only the `after` steps run.
    pub unasked_skips_save: bool,
    pub after: &'static [Step],
    /// The Escape flag `dword_755A3C` is looked at at the top of every frame, before the quit
    /// switch, and the screen is left when it is 1.
    pub escape_leaves: bool,
    /// Steps of the screen's own once a frame, after the frame and before the keys.
    pub each_frame: &'static [Step],
    /// After an action only `dword_755569` and `dword_755561` are looked at; unless the
    /// screen is left it is set up again, whatever `dword_755A38` and `dword_75555D` hold.
    pub short_action_check: bool,
    /// A variable that, when it is 1 at the moment of leaving, makes the loop return at once:
    /// no question, the highlight not saved, no `after` steps.
    pub leave_at_once: Option<Cell>,
    /// "Next frame" (after `handle_keys` returns 1, and after an action that leaves 2 in
    /// `dword_755A38`) goes to the `after_first_frame` steps, not past them. A `Flow` those
    /// steps return is honoured on every screen.
    pub next_frame_repeats_first: bool,
    /// The variable the highlight is kept in around an item's action, where it is not
    /// `saved_item`.
    pub action_saved_item: Option<Cell>,
    /// The menu whose item's action is run, where it is not `menu`: three loops name a
    /// neighbouring screen's menu there (the original's slip, reproduced).
    pub action_menu: Option<fn(&'static Menus) -> MenuId>,
    /// The quit switch is not looked at; `dword_7556A4` = 0x6F is stored once a frame in
    /// that place.
    pub no_quit_check: bool,
    /// On a screen that asks: what runs before the question is put.
    pub before_asking: &'static [Step],
    /// A variable that, when it is 1, keeps the question from being put although
    /// `dword_758806` is 1.
    pub ask_unless: Option<fn(&'static Menus) -> u32>,
    /// On a screen that does not ask: what runs first on leaving, before the highlight is
    /// saved (the mode menu calls `menu::go_back` there when `dword_758806` is 1).
    pub leaving: &'static [Step],
    /// The variable the highlight is saved in on leaving, where it is not `saved_item`.
    pub leave_saved_item: Option<Cell>,
    /// The menu stored in `dword_731BBB` before `menu::handle_keys`, where it is not `menu`.
    pub keys_menu: Option<fn(&'static Menus) -> MenuId>,
    /// What runs last of all, with the carried values back as they were at entry.
    pub after_restored: &'static [Step],
    /// What the loop returns (in eax), where it returns something: read from the game's
    /// variables as the loop leaves them.
    pub result: Option<fn(&'static Menus) -> u32>,
    /// `dword_758806` is cleared and the highlight put in place (with the Escape check) after
    /// the first frame, not at the start of set-up, and set-up stores neither `dword_7556A4`
    /// nor `dword_755561` after the lists: the main menu.
    pub late_highlight: bool,
    /// Steps before the first frame after set-up.
    pub before_first_frame: &'static [Step],
    /// Steps before every other frame.
    pub before_frame: &'static [Step],
    /// Steps right after `update_blink` at the top of a frame; a `Flow` is honoured.
    pub after_blink: &'static [Step],
    /// After an action: left when `dword_755569` is 1; when `dword_755980` is 1 the answer
    /// `g_yesNoAnswer` is cleared, a key taken and the screen set up again; left when
    /// `dword_755561` is 1, which stays 1; otherwise `dword_7556A4` = 0x6F and set up again.
    pub restart_action_check: bool,
    /// The question is put whenever the screen is left, and after a "yes" the loop returns
    /// at once: the highlight is not saved.
    pub always_asks: bool,
}

/// What a screen is entered with that its steps read: the list of cars its carousel shows
/// (`scene::set_up_carousel` reads it for menus 0xD and 0xE).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub cars: Option<CarList>,
}

/// Whether the game is quitting (`dword_51837C`).
pub fn quitting() -> bool {
    system().session.flags.quitting.get() == 1
}

/// `scene::prepare_menu_scene` (0x46F7FD). (It leaves esi the view's turn matrix: a screen
/// whose carousel is set up after it hands that no list of cars.)
pub fn prepare_scene() {
    crate::scene::prepare_menu_scene::prepare_menu_scene();
}

/// `scene::set_up_carousel` (0x4904DD), with the list of cars menus 0xD and 0xE show.
pub fn carousel(cars: Option<CarList>) -> Result<(), Refused> {
    crate::scene::set_up_carousel::set_up_carousel(cars)
}

/// A line for the game's log at level 1 with its line break (`logfile::log_write_l1_nl`,
/// called between `pusha` and `popa`: no register changes).
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
    /// 0x47EFFF, with the picture's path (the original's eax).
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
    /// 0x40939F, with the axis (the original's eax).
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
    /// 0x411863, with the picture's path (the original's eax).
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
    pub fn run(self) -> Result<bool, Refused> {
        let menus = menus();
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
            Self::PrepareScene => prepare_scene(),
            Self::ApplyController => ci::apply_controller()?,
            Self::ReleaseController => ci::release_controller(),
            Self::DrawStageMap => crate::menu::stage_choice::draw_stage_map(menus)?,
            Self::StopStream => crate::sound::stop_stream::stop_stream(),
            Self::PlayTune4 => crate::sound::play_tune_4::play_tune_4(),
            Self::PlayTune9 => crate::sound::play_tune_9::play_tune_9(),
            Self::StopMusic => crate::sound::stop_music::stop_music(),
            Self::StepCarStream => crate::sound::step_car_stream::step_car_stream_in_game()?,
            Self::PlayCarStream => crate::sound::play_car_stream::play_car_stream(
                &mut crate::sound::start_stream::start_stream,
            )?,
            // What it leaves (eax, the carry) is read by none of the screens.
            Self::AxisMoved(axis) => {
                crate::input::axes::axis_moved(axis)?;
            }
            Self::RankStageTimes => rs::rank_stage_times(menus)?,
            Self::RankTotals => rs::rank_totals(menus)?,
            Self::ScoreFourPlayers => rs::score_four_players(menus)?,
            Self::ScoreTwoPlayers => rs::score_two_players(menus)?,
            Self::StyleStageItem => v::style_stage_item(screens, false)?,
            Self::StyleChosenStageItem => v::style_stage_item(screens, true)?,
            Self::CountStagePages => v::count_stage_pages(screens)?,
            Self::PlayStageSpeech => v::play_stage_speech(menus)?,
            Self::PlayRallySpeech => v::play_rally_speech(menus)?,
            Self::LoadWelcomePicture => v::load_welcome_picture(menus)?,
            Self::LoadServiceBackground => crate::service::load_background::load_background()?,
            Self::GoBack => crate::menu::go_back::go_back(),
            Self::MarkStageChoice => crate::menu::mark_stage_choice::mark_stage_choice(play),
            Self::EditPlayerName => {
                crate::menu::edit_player_name::edit_player_name(screens, &mut GameCanvas)?;
            }
            Self::EditSetupName => {
                crate::service::edit_setup_name::edit_setup_name(&mut GameCanvas)?;
            }
            Self::EditSavegameName => crate::menu::edit_savegame_name::edit_savegame_name(
                menus,
                &mut GameCanvas,
                &mut crate::menu::edit_savegame_name::save_game,
            )?,
            Self::EditReplayName => {
                return crate::menu::edit_replay_name::edit_replay_name(menus, &mut GameCanvas);
            }
            Self::CheckCheatName => crate::menu::check_cheat_name::check_cheat_name(play)?,
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
            Self::SetUpFrontEnd => crate::menu::set_up_front_end::set_up_front_end(menus)?,
            // What it leaves in eax, ebx, ecx and edx is read by none of the screens.
            Self::SetUpEngine => crate::scene::set_up_engine::set_up_engine()?,
            Self::SetUpFrontEndScene => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
            }
        }
        Ok(false)
    }
}

// ---- Rust version ----

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
fn clear_escape(screens: &Screens) {
    if screens.draw_work.escaped.get() == 1 {
        screens.draw_work.escaped.set(0u32);
        screens.loop_flags.back.set(0u32);
    }
}

/// A call whose failure is the fatal error `code`.
pub fn call_or_fatal(function: Callee, code: u32) -> Result<(), Refused> {
    if { function.run() }? {
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
    let menus = menus();
    for step in steps {
        match *step {
            Step::DoFlow(what) => match what(entry)? {
                Flow::On => {}
                elsewhere => return Ok(elsewhere),
            },
            Step::Call(function) => {
                function.run()?;
            }
            Step::Write(variable, value) => variable(menus).set(value),
            Step::WriteF32(variable, value) => variable(menus).set(value),
            Step::CallOrFatal(function, code) => call_or_fatal(function, code)?,
            Step::Do(what) => what(entry)?,
        }
    }
    Ok(Flow::On)
}

/// Puts the menu and the lists in place, as the original does before a frame (the esi it
/// leaves at the last list's table is read by no one: `cars` is set before its reader).
fn set_lists(screens: &Screens, screen: &Screen, lists: &[List]) {
    let menus = menus();
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
pub fn screen_loop(screens: &Screens, screen: &Screen, entry: &Entry) -> Result<String, Refused> {
    let menus = menus();
    let menu = (screen.menu)(menus);
    let keys_menu = screen.keys_menu.map_or(menu, |other| other(menus));
    run(screen.before, entry)?;
    let mut state = Loop {
        screen,
        entry,
        menu,
        callback: screen.frame_callback,
        later_callback: screen.later_frame_callback.unwrap_or(screen.frame_callback),
        keys_menu,
        action_menu: screen.action_menu.map_or(menu, |other| other(menus)),
        next_frame: if screen.next_frame_repeats_first {
            Next::FrameTop
        } else {
            Next::Frame
        },
        frames: 0,
        actions: 0,
        why: String::new(),
    };
    let mut next = Next::SetUp;
    loop {
        next = match next {
            Next::SetUp => state.set_up()?,
            Next::FrameTop => state.after_first_frame()?,
            Next::Frame => state.frame()?,
            Next::ActionCheck if screens.option_work.menu_sound.get() != ENTER_SOUND => {
                state.keys()?
            }
            Next::ActionCheck => state.action()?,
            Next::Leave => match state.leave()? {
                Left::Done(why) => return Ok(why),
                Left::SetUp => Next::SetUp,
            },
        };
    }
}

/// What the loop of one screen carries from step to step.
struct Loop<'a> {
    screen: &'a Screen,
    entry: &'a Entry,
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
    why: String,
}

/// How leaving ended.
enum Left {
    /// The screen is left, for the reason given.
    Done(String),
    /// The question was answered "no": the screen is set up again.
    SetUp,
}

impl Loop<'_> {
    /// Set-up, again after every action that does not leave the screen: the lists put in place
    /// and the first frame drawn.
    fn set_up(&mut self) -> Result<Next, Refused> {
        let system = system();
        let menus = menus();
        let screens = &menus.screens;
        let screen = self.screen;
        let entry = self.entry;
        let menu = self.menu;
        run(screen.set_up_first, entry)?;
        if !screen.late_highlight {
            menus.tables.captions.screen_left.set(0);
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
            menus.screens.option_work.menu_sound.set(NO_SOUND);
            menus.screens.loop_flags.back.set(0u32);
        }
        run(screen.background, entry)?;
        crate::menu::draw_frame::draw_frame(menus, menu)?;
        update_blink(menus);
        system
            .session
            .clocks
            .setup_time
            .set(system.session.clocks.frame.get());
        race().shared.physics_tables.scene_clock.set(0u32);
        run(screen.before_first_frame, entry)?;
        set_lists(
            screens,
            screen,
            screen.first_frame_lists.unwrap_or(screen.lists),
        );
        run_frame(menus, self.callback, menu)?;
        if screen.late_highlight {
            menus.tables.captions.screen_left.set(0);
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
                    self.why = "a step of the screen's own".into();
                    Next::Leave
                }
            }
        })
    }

    /// One frame: blink, quit and Escape checks, the frame self.callback and the per-frame steps.
    fn frame(&mut self) -> Result<Next, Refused> {
        let system = system();
        let menus = menus();
        let screens = &menus.screens;
        let screen = self.screen;
        let entry = self.entry;
        let menu = self.menu;
        Ok({
            update_blink(menus);
            self.frames += 1;
            let flow = run(screen.after_blink, entry)?;
            if flow == Flow::SetUp {
                Next::SetUp
            } else if flow == Flow::Leave {
                self.why = "a step of the screen's own".into();
                Next::Leave
            } else if screen.escape_leaves && screens.draw_work.escaped.get() == 1 {
                self.why = "the Escape flag was set".into();
                Next::Leave
            } else if !screen.no_quit_check && quitting() {
                // At once: a screen left as if chosen lets the flow behind it go on with
                // what the screen never set (a championship's next stage: a fatal error).
                crate::standalone::leave_on_close();
            } else if crate::debug::screens::serve()? {
                Next::SetUp
            } else {
                if screen.no_quit_check {
                    screens.option_work.menu_sound.set(NO_SOUND);
                }
                run(screen.before_frame, entry)?;
                set_lists(screens, screen, screen.lists);
                run_frame(menus, self.later_callback, menu)?;
                race().shared.physics_tables.scene_clock.set(
                    system
                        .session
                        .clocks
                        .frame
                        .get()
                        .wrapping_sub(system.session.clocks.setup_time.get()),
                );
                match run(screen.each_frame, entry)? {
                    Flow::On => Next::ActionCheck,
                    Flow::NextFrame => Next::Frame,
                    Flow::SetUp => Next::SetUp,
                    Flow::Leave => {
                        self.why = "a step of the screen's own".into();
                        Next::Leave
                    }
                }
            }
        })
    }

    /// The keys of a frame (`menu::handle_keys`) and where they lead.
    fn keys(&mut self) -> Result<Next, Refused> {
        let menus = menus();
        Ok({
            menus.tables.texts.keys_menu.set(self.keys_menu.code());
            // The original hands it the time since set-up in eax (the frame's last
            // write there that a screen's steps do not change before it reads it).
            match handle_keys(menus, race().shared.physics_tables.scene_clock.get())? {
                Pressed::Nothing => self.next_frame,
                Pressed::Enter => Next::ActionCheck,
                Pressed::Escape => {
                    self.why = "Escape".to_string();
                    Next::Leave
                }
            }
        })
    }

    /// The highlighted item's action, and what it leaves: leave, next frame or set up again.
    fn action(&mut self) -> Result<Next, Refused> {
        let menus = menus();
        let screens = &menus.screens;
        let screen = self.screen;
        Ok({
            self.actions += 1;
            let item = menus.screens.loop_flags.item.get();
            let kept = screen.action_saved_item.unwrap_or(screen.saved_item);
            kept(menus).set(item);
            clear_escape(screens);
            run_item_action(menus, self.action_menu)?;
            menus.tables.captions.screen_left.set(0);
            menus.screens.loop_flags.item.set(kept(menus).get());
            clear_escape(screens);
            if screen.restart_action_check {
                if menus.screens.loop_flags.ended_a.get() == 1 {
                    self.why = format!("after the action of item {item}: dword_755569 was 1");
                    Next::Leave
                } else if menus.screens.event.event_done.get() == 1 {
                    menus.tables.pages.yes_no_answer.set(0u32);
                    Callee::TakeKey.run()?;
                    Next::SetUp
                } else if menus.screens.loop_flags.back.get() == 1 {
                    self.why = format!("after the action of item {item}: dword_755561 is 1");
                    Next::Leave
                } else {
                    menus.screens.option_work.menu_sound.set(NO_SOUND);
                    Next::SetUp
                }
            } else if menus.screens.loop_flags.ended_a.get() == 1
                || (!screen.short_action_check && menus.screens.event.event_done.get() == 1)
            {
                self.why = format!(
                    "after the action of item {item}: dword_755569={} dword_755980={}",
                    menus.screens.loop_flags.ended_a.get(),
                    menus.screens.event.event_done.get()
                );
                Next::Leave
            } else if menus.screens.loop_flags.back.get() == 1 {
                menus.screens.loop_flags.back.set(0u32);
                self.why = format!("after the action of item {item}: dword_755561 was 1");
                Next::Leave
            } else if screen.short_action_check {
                Next::SetUp
            } else {
                menus.screens.option_work.menu_sound.set(NO_SOUND);
                if menus.screens.draw_work.action_value.get() == 2 {
                    self.next_frame
                } else if menus.screens.loop_flags.went_back.get() == 1 {
                    menus.screens.loop_flags.back.set(0u32);
                    self.why = format!("after the action of item {item}: dword_75555D is 1");
                    Next::Leave
                } else {
                    Next::SetUp
                }
            }
        })
    }

    /// Leaving: the question when the screen asks, then the `after` steps.
    fn leave(&mut self) -> Result<Left, Refused> {
        let menus = menus();
        let screens = &menus.screens;
        let screen = self.screen;
        let entry = self.entry;
        let done = || {
            format!(
                "{}; {} frames, {} actions",
                self.why, self.frames, self.actions
            )
        };
        if screen
            .leave_at_once
            .is_some_and(|flag| flag(menus).get() == 1)
        {
            return Ok(Left::Done(done()));
        }
        if let Some(on_refusal) = screen.confirm {
            let asked = screen.always_asks
                || (menus.tables.captions.screen_left.get() == 1
                    && !screen.ask_unless.is_some_and(|flag| flag(menus) == 1))
                || screen
                    .ask_also_if
                    .is_some_and(|flag| flag(menus).get() == 1);
            if asked {
                run(screen.before_asking, entry)?;
                confirm_leaving()?;
                if menus.tables.pages.yes_no_answer.get() != 1 {
                    run(on_refusal, entry)?;
                    return Ok(Left::SetUp);
                }
                run(screen.confirmed, entry)?;
                if screen.always_asks {
                    return Ok(Left::Done(done()));
                }
            } else if screen.unasked_skips_save {
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
            asked = run_screen(&crate::menu::leave_question_loop::SCREEN, &Entry::default())
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
    let why = { screen_loop(screens, screen, &Entry::default()) }?;
    log(&format!(
        "{name}: the Rust version has left the screen ({why})"
    ));
    Ok(())
}

/// A screen loop run from Rust (the event flows, `confirm_leaving`), entered with what the
/// screen reads from its caller (`Entry`: the list of cars of `select_car_loop` and
/// `car_specs_47af4c_loop`). Returns the screen's result where it has one (the original's
/// eax).
pub fn run_screen(screen: &Screen, entry: &Entry) -> Result<Option<u32>, Refused> {
    let menus = menus();
    let screens = &menus.screens;
    { screen_loop(screens, screen, entry) }?;
    Ok(screen.result.map(|result| result(menus)))
}
