//! The loop every menu screen runs, from entering the screen to leaving it. The game has 83
//! copies of it, one per screen, that differ in what `Screen` describes; each is hooked from
//! a file of its own that holds its `Screen`, and this is what they share.
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
//! The original's registers: most of what it holds in them is scratch, but some callees
//! read what an earlier step left, and the loop keeps those values in `Carried` (named for
//! what reads them; see its fields), exactly as the original's registers would hold them at
//! each call (2026-10-24: the register set the loop used to thread went). The screens' own
//! steps (`Step::Do`) work on it the same way.
//!
//! What the loop calls is called directly: its own functions (`call` below for the steps
//! that name a function by its address), the frame callbacks, `handle_keys`, the item's
//! action (`menu::run_item_action`, with the carried values in the registers the action
//! gets, as the original hands them). A refusal goes back to the loop's caller; a set
//! carry after a `Step::CallOrFatal` is the fatal error, raised by the caller too.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, log, rva};
use crate::input::take_key::take_key;
use crate::menu::handle_keys::handle_keys;
use crate::menu::handle_yes_no::LEAVING_RVA;
use crate::menu::item_at::CURRENT_MENU_RVA;
use crate::menu::run_item_action::run_item_action;
use crate::menu::run_list::run_list;
use crate::menu::update_blink::update_blink;
use crate::service::state::MENU_ITEM_RVA;
use crate::session::CLOCKS;
use crate::session::FLAGS;

// ---- game code the loop calls itself ----

pub const TAKE_KEY: usize = rva(0x407F95); // input::take_key
pub const UPDATE_BLINK: usize = rva(0x47E826); // menu::update_blink
pub const FRAME_CALLBACK: usize = rva(0x474D4B); // menu::frame_callback
const INSPECTION_FRAME: usize = rva(0x46D019); // menu::inspection_frame
const NULLSUB_83: usize = rva(0x471391); // nullsub_83: a lone `retn`
const NULLSUB_84: usize = rva(0x4713DD); // nullsub_84: a lone `retn`

// ---- game data ----

const ESCAPED_RVA: usize = rva(0x755A3C); // dword_755A3C
const BACK_RVA: usize = rva(0x755561); // dword_755561
const WENT_BACK_RVA: usize = rva(0x75555D); // dword_75555D
const SOUND_RVA: usize = rva(0x7556A4); // dword_7556A4: 0x14D = Enter was pressed
const DONE_A_RVA: usize = rva(0x755569); // dword_755569
const DONE_B_RVA: usize = rva(0x755980); // dword_755980
const ACTION_VALUE_RVA: usize = rva(0x755A38); // dword_755A38: 2 = stay on the screen as it is
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const KEYS_MENU_RVA: usize = rva(0x731BBB); // dword_731BBB
const ELAPSED_RVA: usize = rva(0x613830); // dword_613830

const NO_SOUND: u32 = 0x6F;
const ENTER_SOUND: u32 = 0x14D;

// ---- what differs from screen to screen ----

/// Something a screen does at one of the loop's fixed places.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    /// Calls the game function at this RVA (one of `call`'s).
    Call(usize),
    /// Writes a game variable.
    Write(usize, u32),
    WriteF32(usize, f32),
    /// Sets what the next call is handed (the original's eax).
    Arg(u32),
    /// The same with the address of something in the game (a path, say).
    ArgAddr(usize),
    /// Calls the game function at this RVA, and if it comes back with the carry flag set,
    /// that is the fatal error with this code (and no message).
    CallOrFatal(usize, u32),
    /// Something only this screen does, written out in its file.
    Do(unsafe fn(&mut Carried) -> Result<(), Refused>),
    /// The same for a step that can also send the loop somewhere: a name editor called once
    /// a frame that keeps the keys from the menu while a name is typed. Only looked at in
    /// `each_frame`.
    DoFlow(unsafe fn(&mut Carried) -> Result<Flow, Refused>),
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
    pub pointer: usize,
    pub table: usize,
    /// Run with `menu::run_list` at set-up, before its pointer is stored. Such a list is not
    /// put in place again before each frame; the others are.
    pub run: bool,
}

pub struct Screen {
    /// off_...: the screen's menu.
    pub menu: usize,
    /// The variable the highlight is kept in between visits.
    pub saved_item: usize,
    pub lists: &'static [List],
    /// The lists put in place before the first frame after set-up, where they are not the
    /// same as before every other frame: 35 loops put their 3D list in `g_unreadList` at
    /// set-up and before each frame, and a list of their own in `g_sceneList` only here.
    pub first_frame_lists: Option<&'static [List]>,
    /// `FRAME_CALLBACK` or `menu::inspection_frame` (0x46D019).
    pub frame_callback: usize,
    /// The frame callback of every frame but the first after set-up, where it is another
    /// one: the inspection area's loop is the only one.
    pub later_frame_callback: Option<usize>,
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
    pub ask_also_if: Option<usize>,
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
    pub leave_at_once: Option<usize>,
    /// "Next frame" (after `handle_keys` returns 1, and after an action that leaves 2 in
    /// `dword_755A38`) goes to the `after_first_frame` steps, not past them. A `Flow` those
    /// steps return is honoured on every screen.
    pub next_frame_repeats_first: bool,
    /// The variable the highlight is kept in around an item's action, where it is not
    /// `saved_item`.
    pub action_saved_item: Option<usize>,
    /// The menu whose item's action is run, where it is not `menu`: three loops name a
    /// neighbouring screen's menu there (the original's slip, reproduced).
    pub action_menu: Option<usize>,
    /// The quit switch is not looked at; `dword_7556A4` = 0x6F is stored once a frame in
    /// that place.
    pub no_quit_check: bool,
    /// On a screen that asks: what runs before the question is put.
    pub before_asking: &'static [Step],
    /// A variable that, when it is 1, keeps the question from being put although
    /// `dword_758806` is 1.
    pub ask_unless: Option<usize>,
    /// On a screen that does not ask: what runs first on leaving, before the highlight is
    /// saved (the mode menu calls `menu::go_back` there when `dword_758806` is 1).
    pub leaving: &'static [Step],
    /// The variable the highlight is saved in on leaving, where it is not `saved_item`.
    pub leave_saved_item: Option<usize>,
    /// The menu stored in `dword_731BBB` before `menu::handle_keys`, where it is not `menu`.
    pub keys_menu: Option<usize>,
    /// What runs last of all, with the carried values back as they were at entry.
    pub after_restored: &'static [Step],
    /// What the loop returns (in eax), where it returns something: read from the game's
    /// variables as the loop leaves them.
    pub result: Option<unsafe fn() -> u32>,
    /// `dword_758806` is cleared and the highlight put in place (with the Escape check) after
    /// the first frame, not at the start of set-up, and set-up stores neither `dword_7556A4`
    /// nor `dword_755561` after the lists: the main menu.
    pub late_highlight: bool,
    /// Steps before the first frame after set-up, with the clock in `Carried::arg`.
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

/// The values of the original's registers that something the loop calls reads, as the
/// original would hold them at each call. The loop sets them where the original sets those
/// registers, and every callee that changes one changes it here (`call`, the screens' own
/// steps). Its other registers are scratch that nothing reads (edi is the menu wherever a
/// callee reads it; ebp nothing reads).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Carried {
    /// eax: what the next call is handed (a path, a direction, a mode); after a frame the
    /// milliseconds since set-up, whose bits 8 .. 15 `handle_keys` reads on Enter.
    pub arg: u32,
    /// ebx: the palette `load_background` draws its frame sprite with and `draw_menu` a
    /// highlighted first item's sprite with (the saved highlight, mostly).
    pub palette: u32,
    /// ecx: the x `draw_menu` measures a first item that is a text button from, and
    /// `menu::set_up_front_end` the players' names.
    pub measure_x: u32,
    /// edx: the frame clock `handle_keys` leaves, or what else last wrote edx; read by
    /// `scene::set_up_front_end_scene`, `menu::play_films` and the items' actions.
    pub clock: u32,
    /// esi: the list of cars `scene::set_up_carousel` reads for menus 0xD and 0xE (from the
    /// caller, else what the loop last left in esi); read by
    /// `scene::set_up_front_end_scene`, `menu::play_films` and the items' actions.
    pub cars: u32,
    /// The carry flag, which `Step::CallOrFatal` reads after its call (a call that does not
    /// write it leaves the one before).
    pub carry: bool,
}

/// Whether the game is quitting (`dword_51837C`).
pub fn quitting() -> bool {
    FLAGS.quitting.get() == 1
}

/// `scene::prepare_menu_scene` (0x46F7FD), with what it leaves (`set_view_matrices`').
pub unsafe fn prepare_scene(c: &mut Carried) {
    let left = unsafe { crate::scene::prepare_menu_scene::prepare_menu_scene() };
    let mut r = Regs {
        eax: c.arg,
        ebx: c.palette,
        ecx: c.measure_x,
        edx: c.clock,
        esi: c.cars,
        ..Regs::default()
    };
    left.apply(&mut r);
    (c.arg, c.palette, c.measure_x, c.clock, c.cars) = (r.eax, r.ebx, r.ecx, r.edx, r.esi);
}

/// `scene::set_up_carousel` (0x4904DD): reads the list of cars (esi) for menus 0xD and 0xE
/// and leaves eax, ebx, ecx and esi by the menu. It reads no edi.
pub unsafe fn carousel(c: &mut Carried) -> Result<(), Refused> {
    let left = unsafe { crate::scene::set_up_carousel::set_up_carousel(c.cars) }?;
    // edi is not carried.
    let mut edi = 0;
    left.over([
        &mut c.arg,
        &mut c.palette,
        &mut c.measure_x,
        &mut c.cars,
        &mut edi,
    ]);
    Ok(())
}

pub unsafe fn read_at(address: u32) -> u32 {
    unsafe { (address as *const u32).read_unaligned() }
}

/// A line for the game's log at level 1 with its line break (`logfile::log_write_l1_nl`,
/// called between `pusha` and `popa`: no register changes).
pub unsafe fn log_line(text: usize) {
    unsafe { crate::logfile::log_write::log_write(game_addr(text) as *const u8, true, 1) };
}

/// The function at `function` (an RVA) a step names, run by its Rust version, with what it
/// reads and leaves of the carried values.
pub unsafe fn call(function: usize, c: &mut Carried) -> Result<(), Refused> {
    use crate::menu::{controller_items as ci, results_setups as rs, stage_view_steps as v};
    use crate::text::draw_text::GameCanvas;
    unsafe {
        match function {
            TAKE_KEY => c.arg = c.arg & !0xFF | take_key() as u32,
            UPDATE_BLINK => update_blink(),
            NULLSUB_83 | NULLSUB_84 => {}
            f if f == rva(0x47EFFF) => {
                crate::menu::load_background::load_background(
                    c.arg as *mut u8,
                    c.palette,
                    &mut GameCanvas,
                )?;
                c.carry = false;
            }
            f if f == rva(0x46F7FD) => prepare_scene(c),
            f if f == rva(0x4904DD) => carousel(c)?,
            f if f == rva(0x46B47C) => ci::apply_controller()?,
            f if f == rva(0x46B4B4) => ci::release_controller(),
            f if f == rva(0x4683CF) => crate::menu::stage_choice::draw_stage_map()?,
            f if f == rva(0x40F757) => crate::sound::stop_stream::stop_stream(),
            f if f == rva(0x40F792) => crate::sound::play_tune_4::play_tune_4(),
            f if f == rva(0x40F7A4) => crate::sound::play_tune_9::play_tune_9(),
            f if f == rva(0x40F826) => crate::sound::stop_music::stop_music(),
            f if f == rva(0x46F7AD) => crate::sound::step_car_stream::step_car_stream_in_game(),
            f if f == rva(0x469AA1) => crate::sound::play_car_stream::play_car_stream(
                &mut crate::sound::start_stream::start_stream,
            )?,
            f if f == rva(0x40939F) => {
                let moved = crate::input::axes::axis_moved(c.arg);
                (c.arg, c.carry) = (moved.eax, moved.carry);
            }
            f if f == rva(0x48340B) => rs::rank_stage_times()?,
            f if f == rva(0x48356F) => rs::rank_totals()?,
            f if f == rva(0x485CBB) => rs::score_four_players()?,
            f if f == rva(0x485B0E) => rs::score_two_players()?,
            f if f == rva(0x470F9F) => v::style_stage_item(false)?,
            f if f == rva(0x470FC3) => v::style_stage_item(true)?,
            f if f == rva(0x480032) => v::count_stage_pages()?,
            f if f == rva(0x47F437) => v::play_stage_speech()?,
            f if f == rva(0x47F40F) => v::play_rally_speech()?,
            f if f == rva(0x4684BD) => v::load_welcome_picture()?,
            f if f == rva(0x470F55) => crate::service::load_background::load_background()?,
            f if f == rva(0x471C82) => crate::menu::go_back::go_back(),
            f if f == rva(0x4708FD) => crate::menu::car_choice::find_player_car()?,
            f if f == rva(0x486C21) => crate::menu::mark_stage_choice::mark_stage_choice(),
            f if f == rva(0x467C63) => {
                crate::menu::edit_player_name::edit_player_name(&mut GameCanvas)?
            }
            f if f == rva(0x46F1A9) => {
                crate::service::edit_setup_name::edit_setup_name(&mut GameCanvas)?
            }
            f if f == rva(0x46D685) => crate::menu::edit_savegame_name::edit_savegame_name(
                &mut GameCanvas,
                &mut crate::menu::edit_savegame_name::save_game,
            )?,
            f if f == rva(0x46F4E7) => {
                c.carry = crate::menu::edit_replay_name::edit_replay_name(&mut GameCanvas)?
            }
            f if f == rva(0x480CBF) => crate::menu::check_cheat_name::check_cheat_name()?,
            f if f == rva(0x4684F5) => {
                crate::menu::load_slideshow_picture::load_slideshow_picture_in_game(c.arg)?
            }
            f if f == rva(0x468482) => {
                crate::menu::load_car_background::load_car_background_in_game()?;
                c.carry = false;
            }
            f if f == rva(0x411863) => {
                c.carry = !crate::gfx::load_picture::load_picture(c.arg as *mut u8)?
            }
            // The carry its last instruction leaves.
            f if f == rva(0x406126) => {
                c.carry = crate::gfx::convert_picture::convert_picture_now()?
            }
            // Reads ebp (only where it is not looked at, see there) and ecx; keeps every
            // register.
            f if f == rva(0x47C63F) => {
                crate::menu::set_up_front_end::set_up_front_end(0, c.measure_x)?
            }
            // Read no edi (`lq.py`: `set_up_engine` reads nothing, the scene edx and esi).
            f if f == rva(0x442AF6) => {
                crate::scene::set_up_engine::set_up_engine()?;
                // What it leaves in eax, ebx, ecx and edx (esi and the carry as they were).
                [c.arg, c.palette, c.measure_x, c.clock] = crate::physics::test_drive::LEFT;
            }
            f if f == rva(0x474E04) => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene()?
            }
            _ => return Err(BadInput("a screen's step calls a function it does not know").into()),
        }
    }
    Ok(())
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
unsafe fn clear_escape() {
    unsafe {
        if game_read::<u32>(ESCAPED_RVA) == 1 {
            game_write(ESCAPED_RVA, 0u32);
            game_write(BACK_RVA, 0u32);
        }
    }
}

/// A call after which a set carry flag is the fatal error `code`.
pub unsafe fn call_or_fatal(function: usize, code: u32, c: &mut Carried) -> Result<(), Refused> {
    unsafe { call(function, c) }?;
    if c.carry {
        log(&format!(
            "a screen loop's Rust version raises fatal error {code:#x}"
        ));
        return Err(FatalError { code, message: "" }.into());
    }
    Ok(())
}

/// Runs the steps of one place. Anything but `Flow::On` comes from a `DoFlow` step, after
/// which the rest of the place is not run.
unsafe fn run(steps: &[Step], c: &mut Carried) -> Result<Flow, Refused> {
    for step in steps {
        unsafe {
            match *step {
                Step::DoFlow(what) => match what(c)? {
                    Flow::On => {}
                    elsewhere => return Ok(elsewhere),
                },
                Step::Call(function) => call(function, c)?,
                Step::Write(variable, value) => game_write(variable, value),
                Step::WriteF32(variable, value) => game_write(variable, value),
                Step::Arg(value) => c.arg = value,
                Step::ArgAddr(what) => c.arg = game_addr(what) as u32,
                Step::CallOrFatal(function, code) => call_or_fatal(function, code, c)?,
                Step::Do(what) => what(c)?,
            }
        }
    }
    Ok(Flow::On)
}

/// Puts the menu and the lists in place, as the original does before a frame (esi left
/// at the last list's table).
unsafe fn set_lists(screen: &Screen, lists: &[List], c: &mut Carried) {
    unsafe {
        game_write(CURRENT_MENU_RVA, game_addr(screen.menu) as u32);
        for list in lists.iter().filter(|list| !list.run) {
            c.cars = game_addr(list.table) as u32;
            game_write(list.pointer, c.cars);
        }
    }
}

/// `screens::run_frame` (0x405A71) with the screen's frame callback: the messages, then
/// (unless quitting) the frame, with the menu and the carried palette and x. Neither
/// changes a carried value (the callbacks keep every register; the carry they leave is
/// read by nothing).
unsafe fn run_frame(callback: usize, menu: u32, c: &Carried) -> Result<(), Refused> {
    let frame = || unsafe {
        match callback {
            FRAME_CALLBACK => {
                crate::menu::frame_callback::frame_callback(menu, c.palette, c.measure_x)
            }
            INSPECTION_FRAME => {
                crate::menu::inspection_frame::inspection_frame(menu, c.palette, c.measure_x)
            }
            _ => Err(BadInput("a screen with a frame callback of no menu's").into()),
        }
    };
    match crate::platform::run_frame_now(frame) {
        Some(made) => made.map(|_| ()),
        None => Ok(()),
    }
}

/// `entry` are the carried values at entry.
/// Returns why the screen was left, for the log.
pub unsafe fn screen_loop(screen: &Screen, entry: &Carried) -> Result<String, Refused> {
    let menu = game_addr(screen.menu) as u32;
    let callback = screen.frame_callback;
    let later_callback = screen.later_frame_callback.unwrap_or(screen.frame_callback);
    let keys_menu = screen
        .keys_menu
        .map_or(menu, |other| game_addr(other) as u32);
    let mut c = *entry;
    let (mut frames, mut actions, mut why) = (0u32, 0u32, String::new());
    unsafe {
        run(screen.before, &mut c)?;

        let action_menu = screen
            .action_menu
            .map_or(menu, |other| game_addr(other) as u32);
        let next_frame = if screen.next_frame_repeats_first {
            Next::FrameTop
        } else {
            Next::Frame
        };
        let mut next = Next::SetUp;
        loop {
            next = match next {
                Next::SetUp => {
                    run(screen.set_up_first, &mut c)?;
                    if !screen.late_highlight {
                        game_write(LEAVING_RVA, 0u32);
                        c.palette = game_read(screen.saved_item);
                        game_write(MENU_ITEM_RVA, c.palette);
                        clear_escape();
                    }
                    run(screen.prepare, &mut c)?;

                    game_write(CURRENT_MENU_RVA, menu);
                    for list in screen.lists {
                        c.cars = game_addr(list.table) as u32;
                        if list.run {
                            run_list(c.cars)?;
                        }
                        game_write(list.pointer, c.cars);
                    }
                    run(screen.after_lists, &mut c)?;
                    if !screen.late_highlight {
                        game_write(SOUND_RVA, NO_SOUND);
                        game_write(BACK_RVA, 0u32);
                    }
                    run(screen.background, &mut c)?;
                    crate::menu::draw_frame::draw_frame(menu, c.palette, c.measure_x)?;
                    update_blink();
                    c.arg = CLOCKS.frame.get();
                    CLOCKS.setup_time.set(c.arg);
                    game_write(ELAPSED_RVA, 0u32);
                    run(screen.before_first_frame, &mut c)?;
                    c.arg = game_addr(callback) as u32;
                    set_lists(
                        screen,
                        screen.first_frame_lists.unwrap_or(screen.lists),
                        &mut c,
                    );
                    run_frame(callback, menu, &c)?;
                    if screen.late_highlight {
                        game_write(LEAVING_RVA, 0u32);
                        c.palette = game_read(screen.saved_item);
                        game_write(MENU_ITEM_RVA, c.palette);
                        clear_escape();
                    }
                    Next::FrameTop
                }
                Next::FrameTop => match run(screen.after_first_frame, &mut c)? {
                    Flow::On | Flow::NextFrame => Next::Frame,
                    Flow::SetUp => Next::SetUp,
                    Flow::Leave => {
                        why = "a step of the screen's own".into();
                        Next::Leave
                    }
                },
                Next::Frame => {
                    update_blink();
                    frames += 1;
                    let flow = run(screen.after_blink, &mut c)?;
                    if flow == Flow::SetUp {
                        Next::SetUp
                    } else if flow == Flow::Leave {
                        why = "a step of the screen's own".into();
                        Next::Leave
                    } else if screen.escape_leaves && game_read::<u32>(ESCAPED_RVA) == 1 {
                        why = "the Escape flag was set".into();
                        Next::Leave
                    } else if !screen.no_quit_check && quitting() {
                        why = "the game is quitting".into();
                        Next::Leave
                    } else {
                        if screen.no_quit_check {
                            game_write(SOUND_RVA, NO_SOUND);
                        }
                        run(screen.before_frame, &mut c)?;
                        c.arg = game_addr(later_callback) as u32;
                        set_lists(screen, screen.lists, &mut c);
                        run_frame(later_callback, menu, &c)?;
                        c.arg = CLOCKS.frame.get().wrapping_sub(CLOCKS.setup_time.get());
                        game_write(ELAPSED_RVA, c.arg);
                        match run(screen.each_frame, &mut c)? {
                            Flow::On => Next::ActionCheck,
                            Flow::NextFrame => Next::Frame,
                            Flow::SetUp => Next::SetUp,
                            Flow::Leave => {
                                why = "a step of the screen's own".into();
                                Next::Leave
                            }
                        }
                    }
                }
                Next::ActionCheck if game_read::<u32>(SOUND_RVA) != ENTER_SOUND => {
                    game_write(KEYS_MENU_RVA, keys_menu);
                    let keys = handle_keys(c.arg)?;
                    c.arg = keys.result;
                    if let Some(clock) = keys.clock {
                        c.clock = clock;
                    }
                    match keys.result {
                        1 => next_frame,
                        3 => Next::ActionCheck,
                        other => {
                            why = format!("handle_keys returned {other}");
                            Next::Leave
                        }
                    }
                }
                Next::ActionCheck => {
                    let elapsed = c.arg;
                    actions += 1;
                    c.measure_x = game_read(MENU_ITEM_RVA);
                    let item = c.measure_x;
                    let kept = screen.action_saved_item.unwrap_or(screen.saved_item);
                    game_write(kept, c.measure_x);
                    clear_escape();
                    run_item_action(action_menu, &c)?;
                    game_write(LEAVING_RVA, 0u32);
                    c.palette = game_read(kept);
                    game_write(MENU_ITEM_RVA, c.palette);
                    clear_escape();
                    c.arg = elapsed;
                    if screen.restart_action_check {
                        if game_read::<u32>(DONE_A_RVA) == 1 {
                            why = format!("after the action of item {item}: dword_755569 was 1");
                            Next::Leave
                        } else if game_read::<u32>(DONE_B_RVA) == 1 {
                            game_write(ANSWER_RVA, 0u32);
                            call(TAKE_KEY, &mut c)?;
                            Next::SetUp
                        } else if game_read::<u32>(BACK_RVA) == 1 {
                            why = format!("after the action of item {item}: dword_755561 is 1");
                            Next::Leave
                        } else {
                            game_write(SOUND_RVA, NO_SOUND);
                            Next::SetUp
                        }
                    } else if game_read::<u32>(DONE_A_RVA) == 1
                        || (!screen.short_action_check && game_read::<u32>(DONE_B_RVA) == 1)
                    {
                        why = format!(
                            "after the action of item {item}: dword_755569={} dword_755980={}",
                            game_read::<u32>(DONE_A_RVA),
                            game_read::<u32>(DONE_B_RVA)
                        );
                        Next::Leave
                    } else if game_read::<u32>(BACK_RVA) == 1 {
                        game_write(BACK_RVA, 0u32);
                        why = format!("after the action of item {item}: dword_755561 was 1");
                        Next::Leave
                    } else if screen.short_action_check {
                        Next::SetUp
                    } else {
                        game_write(SOUND_RVA, NO_SOUND);
                        if game_read::<u32>(ACTION_VALUE_RVA) == 2 {
                            next_frame
                        } else if game_read::<u32>(WENT_BACK_RVA) == 1 {
                            game_write(BACK_RVA, 0u32);
                            why = format!("after the action of item {item}: dword_75555D is 1");
                            Next::Leave
                        } else {
                            Next::SetUp
                        }
                    }
                }
                Next::Leave => {
                    let done = || format!("{why}; {frames} frames, {actions} actions");
                    if screen
                        .leave_at_once
                        .is_some_and(|flag| game_read::<u32>(flag) == 1)
                    {
                        return Ok(done());
                    }
                    if let Some(on_refusal) = screen.confirm {
                        let asked = screen.always_asks
                            || (game_read::<u32>(LEAVING_RVA) == 1
                                && !screen
                                    .ask_unless
                                    .is_some_and(|flag| game_read::<u32>(flag) == 1))
                            || screen
                                .ask_also_if
                                .is_some_and(|flag| game_read::<u32>(flag) == 1);
                        if asked {
                            run(screen.before_asking, &mut c)?;
                            // The flags of `cmp g_yesNoAnswer, 1`.
                            c.carry = confirm_leaving(&c)? < 1;
                            if game_read::<u32>(ANSWER_RVA) != 1 {
                                run(on_refusal, &mut c)?;
                                next = Next::SetUp;
                                continue;
                            }
                            run(screen.confirmed, &mut c)?;
                            if screen.always_asks {
                                return Ok(done());
                            }
                        } else if screen.unasked_skips_save {
                            run(screen.after, &mut c)?;
                            c = *entry;
                            run(screen.after_restored, &mut c)?;
                            return Ok(done());
                        }
                    } else {
                        run(screen.leaving, &mut c)?;
                    }
                    c.measure_x = game_read(MENU_ITEM_RVA);
                    game_write(
                        screen.leave_saved_item.unwrap_or(screen.saved_item),
                        c.measure_x,
                    );
                    clear_escape();
                    run(screen.after, &mut c)?;
                    c = *entry;
                    run(screen.after_restored, &mut c)?;
                    return Ok(done());
                }
            };
        }
    }
}

/// `menu::confirm_leaving` (0x46FA90): the question's screen (`menu::leave_question_loop`)
/// gets the carried values as they are; every register comes back as it was.
pub unsafe fn confirm_leaving(c: &Carried) -> Result<u32, Refused> {
    let mut asked = Ok(());
    let answer = unsafe {
        crate::menu::confirm_leaving::confirm_leaving(&mut || {
            asked = run_screen(&crate::menu::leave_question_loop::SCREEN, c).map(|_| ());
        })
    };
    asked.map(|()| answer)
}

// ---- entries ----

/// Runs a screen from the menus' tables. The loop starts from nothing carried: its body
/// never reads the registers it was entered with (only `pusha`'d for the `popa` at the
/// end), and what it passes down unchanged reaches only plain Rust versions (2026-10-23,
/// `attrib.py` over the 88 item routes; their esi is the outer loop's list table, never a
/// list of cars). Its result (eax) is unused by the tables' callers. Rust callers use
/// `run_screen`.
pub fn screen_entry(name: &str, screen: &Screen) -> Result<(), Refused> {
    log(&format!("{name}: the Rust version runs"));
    let why = unsafe { screen_loop(screen, &Carried::default()) }?;
    log(&format!(
        "{name}: the Rust version has left the screen ({why})"
    ));
    Ok(())
}

/// A screen loop run from Rust (the event flows, `confirm_leaving`): it starts from the
/// caller's carried values, which can hold what the screen reads (`select_car_loop`'s and
/// `car_specs_47af4c_loop`'s list of cars, read by `scene::set_up_carousel`: the time
/// trial's carousel refused with "no list of cars" when they were dropped, 2026-10-23).
/// Returns the screen's result where it has one (the original's eax; every other register
/// kept).
pub unsafe fn run_screen(screen: &Screen, entry: &Carried) -> Result<Option<u32>, Refused> {
    unsafe { screen_loop(screen, entry) }?;
    Ok(screen.result.map(|result| unsafe { result() }))
}

/// `run_screen` for a flow that holds the carried values: the screen's result, where it
/// has one, becomes `arg` (the original's eax); every other value is kept.
pub unsafe fn run_screen_carried(screen: &Screen, c: &mut Carried) -> Result<(), Refused> {
    if let Some(result) = unsafe { run_screen(screen, c) }? {
        c.arg = result;
    }
    Ok(())
}

/// The carried values in registers `r` (for callers still holding the original's).
pub fn carried(r: &Regs) -> Carried {
    Carried {
        arg: r.eax,
        palette: r.ebx,
        measure_x: r.ecx,
        clock: r.edx,
        cars: r.esi,
        carry: r.carry(),
    }
}

/// `run_screen` for callers still holding the original's registers: the carried values
/// from them, and eax the result where the screen has one.
pub unsafe fn run_screen_regs(screen: &Screen, r: &mut Regs) -> Result<(), Refused> {
    if let Some(result) = unsafe { run_screen(screen, &carried(r)) }? {
        r.eax = result;
    }
    Ok(())
}
