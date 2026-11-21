//! The texts of the pages this game adds to the base game's (the controller axes page, the
//! controller buttons page's new rows and the H shifter's switch). The base game's language file
//! holds none of them, so they are kept in one file per language, `assets/locale/<name>.rc2k.toml`
//! (`<name>` the language's `.LOC` file name), and every file is built into the program: the
//! game's folder never supplies them, so they always match the program. A language with no file,
//! or a file that lacks a text, shows it in English.
//!
//! A file is TOML, UTF-8, one `key = "text"` for each text, the key's dots standing for tables
//! (`axis_page.dead_zone` is `dead_zone` in `[axis_page]`). `KEYS` names every key and the id a
//! menu item gives for it. The texts are converted to Windows-1252 as the game's own are.
//! A translator adds a language by writing a file, no code.
//!
//! Their string ids are past the language file's (`FIRST ..= LAST`); a menu item names one as it
//! names a text of the file.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The first id of an own text.
pub const FIRST: u32 = 0x2000;
/// The last id of an own text.
pub const LAST: u32 = 0x21FF;

/// Whether `id` is an own text's.
pub(crate) fn is_own(id: u32) -> bool {
    (FIRST..=LAST).contains(&id)
}

/// A page's text ids.
pub mod id {
    use super::FIRST;

    /// An empty text.
    pub const BLANK: u32 = FIRST;
    /// The controller page's item that opens the axes page.
    pub const AXES: u32 = FIRST + 1;
    /// The buttons page's item for its second page, and for its first.
    pub const MORE_CONTROLS: u32 = FIRST + 2;
    pub const FIRST_CONTROLS: u32 = FIRST + 3;
    /// A joystick's axis `n` (0 ..= 5), a gamepad's axis `n`.
    pub const JOYSTICK_AXIS: u32 = FIRST + 0x10;
    pub const PAD_AXIS: u32 = FIRST + 0x18;
    /// The axes page's wheel row and its full rotation row.
    pub const WHEEL: u32 = FIRST + 0x20;
    pub const FULL_ROTATION: u32 = FIRST + 0x21;
    /// "yes", "no", "on", "off".
    pub const YES: u32 = FIRST + 0x28;
    pub const NO: u32 = FIRST + 0x29;
    pub const ON: u32 = FIRST + 0x2A;
    pub const OFF: u32 = FIRST + 0x2B;
    /// The row `n` of `Row::LISTED` on the buttons page.
    pub const CONTROL_ROWS: u32 = FIRST + 0x30;
    /// The prompts while a control is learnt: steering axis, pedal or axis, button.
    pub const PROMPT_STEERING: u32 = FIRST + 0x40;
    pub const PROMPT_AXIS: u32 = FIRST + 0x41;
    pub const PROMPT_BUTTON: u32 = FIRST + 0x42;
    /// The axis page's rows: the axis, dead zone, sensitivity, range, invert and the live
    /// position.
    pub const SETTINGS: u32 = FIRST + 0x50;
    /// What a sensitivity does: nothing (linear), finer near the rest, quicker near the rest.
    pub const LINEAR: u32 = FIRST + 0x58;
    pub const GENTLE: u32 = FIRST + 0x59;
    pub const QUICK: u32 = FIRST + 0x5A;
    /// The wheel's unit.
    pub const DEGREES: u32 = FIRST + 0x46;
    /// The buttons page's rows for the H shifter's gears 1 .. 7 and reverse (8 ids).
    pub const GEAR_ROWS: u32 = FIRST + 0x60;
    /// The controller page's row for the button pictures, and its five values (automatic, Xbox,
    /// PlayStation, Steam Deck, numbers).
    pub const BUTTON_ICONS: u32 = FIRST + 0x88;
    pub const ICON_STYLES: u32 = FIRST + 0x90;
    /// The controller page's switch for the engine's part of the force feedback.
    pub const ENGINE_VIBRATION: u32 = FIRST + 0x89;
    /// The hints beside the pictures of a question or the on-screen keyboard: yes, no, type,
    /// delete, cancel, done.
    pub const HINTS: u32 = FIRST + 0x98;
    /// The buttons page's rows for the menu controls (8 ids).
    pub const MENU_ROWS: u32 = FIRST + 0x70;
    /// The sound page's switch for sound while the window is not in front, the volume page's
    /// films slider and the network settings' port.
    pub const BACKGROUND_SOUND: u32 = FIRST + 0xC0;
    pub const FILM_VOLUME: u32 = FIRST + 0xC1;
    pub const NETWORK_PORT: u32 = FIRST + 0xC2;
    /// The network settings' password row, the answer to a refused join (wrong or missing
    /// password), the mark before a locked session's name in the list.
    pub const NETWORK_PASSWORD: u32 = FIRST + 0xD0;
    pub const PASSWORD_REFUSED: u32 = FIRST + 0xD1;
    pub const LOCK_MARK: u32 = FIRST + 0xD2;
    /// What a joiner is asked at a locked session, and when its answer was wrong.
    pub const PASSWORD_PROMPT: u32 = FIRST + 0xD3;
    pub const PASSWORD_WRONG: u32 = FIRST + 0xD4;
    /// The answer to hosting on a port another program holds ("{}" is the port's number).
    pub const PORT_IN_USE: u32 = FIRST + 0xD5;
    /// The lobby's answers to the host's kick command: not the host, no such player, put out,
    /// could not be put out (4 ids).
    pub const KICK_ANSWERS: u32 = FIRST + 0xA0;
    /// The controller page's switch for the H shifter.
    pub const H_SHIFTER: u32 = FIRST + 0x68;
    /// The graphics page's frame limit row, its "no limit", "display" and "frames a second".
    pub const FRAME_LIMIT: u32 = FIRST + 0xC8;
    pub const NO_LIMIT: u32 = FIRST + 0xC9;
    pub const DISPLAY: u32 = FIRST + 0xCA;
    pub const FRAMES_A_SECOND: u32 = FIRST + 0xCB;
    /// The graphics page's rows for the HUD's box and the window's mode, the window modes' names
    /// and the word for a size set by hand.
    pub const HUD_BOX: u32 = FIRST + 0xE0;
    pub const WINDOW_MODE: u32 = FIRST + 0xE1;
    pub const WINDOW: u32 = FIRST + 0xE2;
    pub const FULL_SCREEN: u32 = FIRST + 0xE3;
    pub const BORDERLESS: u32 = FIRST + 0xE4;
    pub const CUSTOM: u32 = FIRST + 0xE5;
    /// The advanced page's row for frames drawn between the ticks.
    pub const SMOOTH_FRAMES: u32 = FIRST + 0xB9;
    /// The advanced page's row for the controls a loaded saved game brings.
    pub const SAVE_CONTROLS: u32 = FIRST + 0xBB;
    /// The advanced page's row for saving after every stage.
    pub const SAVE_EVERY_STAGE: u32 = FIRST + 0x1B9;
    /// The advanced page's button to its second part, and the second part's to the first.
    pub const ADVANCED_MORE: u32 = FIRST + 0xBC;
    pub const ADVANCED_FIRST: u32 = FIRST + 0xBD;
    /// The single rally's players menu: the captions of its three- and four-player split screens.
    pub const SPLIT_THREE: u32 = FIRST + 0xF2;
    pub const SPLIT_FOUR: u32 = FIRST + 0xF3;
    /// The graphics page's 3D resolution row.
    pub const RENDER_SCALE: u32 = FIRST + 0xE8;
    /// The advanced page: its name (also the options menu's item), then the rows' names in
    /// the order of `options::advanced::ROWS` (the eighth text, rain and snow, names the graphics
    /// page's item), then the renderer names and the "next start" note.
    pub const ADVANCED: u32 = FIRST + 0xA8;
    pub const ADVANCED_ROWS: u32 = FIRST + 0xA9;
    pub const RENDERER_GPU: u32 = FIRST + 0xB4;
    pub const RENDERER_SOFTWARE: u32 = FIRST + 0xB5;
    pub const RENDERER_GLES: u32 = FIRST + 0xB6;
    pub const RENDERER_GL: u32 = FIRST + 0xBA;
    pub const NEXT_START: u32 = FIRST + 0xB7;
    /// The note beside the software renderer when it runs because no GPU device could be made.
    pub const NO_GPU: u32 = FIRST + 0x1B8;
    /// The advanced page's item that makes its changes real.
    pub const APPLY: u32 = FIRST + 0xB8;
    /// The network lobby's button that saves the last race's replay.
    pub const SAVE_REPLAY: u32 = FIRST + 0xF8;
    /// The network lobby's button that opens the in-race options.
    pub const LOBBY_SETTINGS: u32 = FIRST + 0xD8;
    /// The network lobby's computer drivers button, with the number of bots: off, then 1 .. 7
    /// (8 ids).
    pub const BOTS: u32 = FIRST + 0x78;
    /// The network lobby's button for the way the bots start: on the grid, then like the
    /// arcade's opponents (2 ids).
    pub const BOT_START: u32 = FIRST + 0xF6;
    /// The message shown when a replay could not be saved.
    pub const REPLAY_NOT_SAVED: u32 = FIRST + 0xD9;
    /// The tag the replays list puts after the name of a slot that holds only the base game's
    pub const REPLAY_OLD: u32 = FIRST + 0xF4;
    /// The network lobby's ready button while the player is ready: it takes the ready back.
    pub const NOT_READY: u32 = FIRST + 0xDA;
    /// The error dialog's texts: the title of an error and of a notice, the ways on (quit,
    /// continue), the line that points to the log, the line that says what the keys do.
    pub const DIALOG_ERROR: u32 = FIRST + 0xE9;
    pub const DIALOG_NOTICE: u32 = FIRST + 0xEA;
    pub const DIALOG_QUIT: u32 = FIRST + 0xEB;
    pub const DIALOG_CONTINUE: u32 = FIRST + 0xEC;
    pub const DIALOG_LOG: u32 = FIRST + 0xED;
    pub const DIALOG_KEYS: u32 = FIRST + 0xEE;
    /// The notices of a damaged saved game ("{}" the slot's number), a damaged replay (the
    /// same), and of settings reset to their defaults ("{}" their names).
    pub const DAMAGED_SAVEGAME: u32 = FIRST + 0xE6;
    pub const DAMAGED_REPLAY: u32 = FIRST + 0xE7;
    pub const SETTINGS_RESET: u32 = FIRST + 0xEF;
    /// The dialog's way back to the main menu, and the notices of a picture, a sound, a film
    /// and a replay that could not be used ("{}" what it was), and of a player dropped from a
    /// network session ("{}" the player).
    pub const DIALOG_MENU: u32 = FIRST + 0xF9;
    pub const BAD_PICTURE: u32 = FIRST + 0xFA;
    pub const BAD_SOUND: u32 = FIRST + 0xFB;
    pub const BAD_PEER: u32 = FIRST + 0xFD;
    /// The race's message that a player left it ("{}" the player), and that the host's leaving
    /// ended the session ("{}" the host).
    pub const LEFT_RACE: u32 = FIRST + 0xFC;
    pub const HOST_LEFT: u32 = FIRST + 0xF5;
    pub const BAD_REPLAY_PLAY: u32 = FIRST + 0xFE;
    /// The replay viewer's dialog before a render to video: its title, the rows' labels
    /// (size, frame rate, part, quality, sound, file, folder), the two buttons, the screen's
    /// own size ("{}" its width and height), the three parts, the four qualities (4 ids), the
    /// note when no encoder was found, and the line that says what the keys do.
    pub const RENDER_TITLE: u32 = FIRST + 0x47;
    pub const RENDER_SIZE: u32 = FIRST + 0x48;
    pub const RENDER_FPS: u32 = FIRST + 0x49;
    pub const RENDER_PART: u32 = FIRST + 0x4A;
    pub const RENDER_QUALITY: u32 = FIRST + 0x4B;
    pub const RENDER_SOUND: u32 = FIRST + 0x4C;
    pub const RENDER_FILE: u32 = FIRST + 0x4D;
    pub const RENDER_FOLDER: u32 = FIRST + 0x4E;
    pub const RENDER_GO: u32 = FIRST + 0x4F;
    pub const RENDER_CANCEL: u32 = FIRST + 0x5B;
    pub const RENDER_SCREEN: u32 = FIRST + 0x5C;
    pub const RENDER_WHOLE: u32 = FIRST + 0x5D;
    pub const RENDER_FROM_HERE: u32 = FIRST + 0x5E;
    pub const RENDER_MARKED: u32 = FIRST + 0x5F;
    pub const RENDER_QUALITIES: u32 = FIRST + 0x69;
    pub const RENDER_NO_ENCODER: u32 = FIRST + 0x6D;
    pub const RENDER_KEYS: u32 = FIRST + 0x6E;
    /// The panel shown while a render runs in a program of its own: its title, how far it has come
    /// ("{}" the percent), the frames written ("{}" their number), the line before it has begun,
    /// and the notices when the video is ready ("{}" its file) and when it could not be made
    /// ("{}" why).
    pub const RENDER_PROGRESS_TITLE: u32 = FIRST + 0x100;
    pub const RENDER_PERCENT: u32 = FIRST + 0x101;
    pub const RENDER_FRAMES: u32 = FIRST + 0x102;
    pub const RENDER_STARTING: u32 = FIRST + 0x103;
    pub const RENDER_DONE: u32 = FIRST + 0x104;
    pub const RENDER_FAILED: u32 = FIRST + 0x105;
    /// The speed of the render ("{}" the frames a second) and the time it has left ("{}" as
    /// minutes and seconds).
    pub const RENDER_SPEED: u32 = FIRST + 0x170;
    pub const RENDER_LEFT: u32 = FIRST + 0x171;
    /// The dialog's encoder row and its two values (automatic, software: 0x173 and 0x174).
    pub const RENDER_ENCODER: u32 = FIRST + 0x172;
    pub const RENDER_ENCODERS: u32 = FIRST + 0x173;
    /// Notices of what went wrong with a video that was made or tried: no `ffmpeg` ("{}" the
    /// folder the numbered pictures went to), the encoder stopping ("{}" its first line), the
    /// card's encoder making a damaged picture (the render was done again in software), and
    /// the sound that could not be joined.
    pub const RENDER_NO_FFMPEG: u32 = FIRST + 0x178;
    pub const RENDER_ENCODER_FAILED: u32 = FIRST + 0x179;
    pub const RENDER_ENCODER_REDONE: u32 = FIRST + 0x17A;
    pub const RENDER_SOUND_FAILED: u32 = FIRST + 0x17B;
    /// The lead-in of the dialog that goes back to the main menu ("{}" the trouble).
    pub const BACK_TO_MENU: u32 = FIRST + 0xFF;
    /// The save code page (`save_code`): its texts follow each other from the options menu's
    /// item.
    pub const SAVE_CODE_ITEM: u32 = FIRST + 0x106;
    pub const SAVE_CODE_TITLE: u32 = FIRST + 0x107;
    pub const SAVE_CODE_COPY: u32 = FIRST + 0x108;
    pub const SAVE_CODE_PASTE: u32 = FIRST + 0x109;
    pub const SAVE_CODE_CAMERA: u32 = FIRST + 0x10A;
    pub const SAVE_CODE_BACK: u32 = FIRST + 0x10B;
    pub const SAVE_CODE_KEYS: u32 = FIRST + 0x10C;
    pub const SAVE_CODE_QR_WHOLE: u32 = FIRST + 0x10D;
    pub const SAVE_CODE_LENGTH: u32 = FIRST + 0x112;
    pub const SAVE_CODE_COPIED: u32 = FIRST + 0x113;
    pub const SAVE_CODE_CLIPBOARD_EMPTY: u32 = FIRST + 0x114;
    pub const SAVE_CODE_BAD_CODE: u32 = FIRST + 0x115;
    pub const SAVE_CODE_NO_CAMERA: u32 = FIRST + 0x116;
    pub const SAVE_CODE_LOOKING: u32 = FIRST + 0x117;
    pub const SAVE_CODE_CAMERA_DENIED: u32 = FIRST + 0x118;
    pub const SAVE_CODE_CONFIRM_TITLE: u32 = FIRST + 0x119;
    pub const SAVE_CODE_IN_CODE: u32 = FIRST + 0x11A;
    pub const SAVE_CODE_NOW: u32 = FIRST + 0x11B;
    pub const SAVE_CODE_ROW_MADE: u32 = FIRST + 0x11C;
    pub const SAVE_CODE_ROW_PROGRAM: u32 = FIRST + 0x11D;
    pub const SAVE_CODE_ROW_BEST: u32 = FIRST + 0x120;
    pub const SAVE_CODE_ROW_SETUPS: u32 = FIRST + 0x121;
    pub const SAVE_CODE_ROW_GAMES: u32 = FIRST + 0x122;
    pub const SAVE_CODE_YES: u32 = FIRST + 0x123;
    pub const SAVE_CODE_NO: u32 = FIRST + 0x124;
    pub const SAVE_CODE_DONE: u32 = FIRST + 0x125;
    pub const SAVE_CODE_FAILED: u32 = FIRST + 0x126;
    pub const SAVE_CODE_NOT_HELD: u32 = FIRST + 0x127;
    pub const SAVE_CODE_CONFIRM_KEYS: u32 = FIRST + 0x128;
    pub const SAVE_CODE_NOTE: u32 = FIRST + 0x129;
    /// The page's notes for a code of several QR codes ("{}" the count), a part's place
    /// ("{}" its number, "{}" the count), the camera's progress ("{}" read, "{}" of how
    /// many), the numbers read ("{}"), a part of another code, and a state too long for parts.
    pub const SAVE_CODE_PARTS_NOTE: u32 = FIRST + 0x160;
    pub const SAVE_CODE_PART_OF: u32 = FIRST + 0x161;
    pub const SAVE_CODE_PARTS_READ: u32 = FIRST + 0x162;
    pub const SAVE_CODE_PARTS_HELD: u32 = FIRST + 0x163;
    pub const SAVE_CODE_PART_OTHER: u32 = FIRST + 0x164;
    pub const SAVE_CODE_TOO_LONG: u32 = FIRST + 0x165;
    /// The save code page's density choice: the button's label while the large codes are shown
    /// (it switches to the small ones) and while the small ones are (it switches back), and the
    /// notes under the text for each.
    pub const SAVE_CODE_SMALL_BUTTON: u32 = FIRST + 0x190;
    pub const SAVE_CODE_LARGE_BUTTON: u32 = FIRST + 0x191;
    pub const SAVE_CODE_LARGE_NOTE: u32 = FIRST + 0x192;
    pub const SAVE_CODE_SMALL_NOTE: u32 = FIRST + 0x193;
    /// The scan screen's line naming the camera in use ("{}" its number, "{}" how many, "{}" its
    /// name) and the keys that choose another.
    pub const SAVE_CODE_CAMERA_NAME: u32 = FIRST + 0x180;
    pub const SAVE_CODE_CAMERA_SWITCH: u32 = FIRST + 0x181;
    /// The same line on a touch screen, where a slide sideways chooses another camera.
    pub const SAVE_CODE_CAMERA_SWITCH_TOUCH: u32 = FIRST + 0x182;
    /// The advanced page's row that deletes everything saved, its question, the answers' labels, and what the player is told after
    /// the deletion (all went, or some could not be removed).
    pub const DELETE_DATA: u32 = FIRST + 0x1A0;
    pub const DELETE_ASK: u32 = FIRST + 0x1A1;
    /// The deletion question's title (the row's name is lower case like every row's).
    pub const DELETE_TITLE: u32 = FIRST + 0x1A2;
    pub const DIALOG_YES: u32 = FIRST + 0x1A3;
    pub const DIALOG_NO: u32 = FIRST + 0x1A4;
    pub const DELETE_DONE: u32 = FIRST + 0x1A5;
    pub const DELETE_FAILED: u32 = FIRST + 0x1A6;
    /// The advanced page's row (phones) that opens the share sheet with the program's log.
    pub const SHARE_LOG: u32 = FIRST + 0x1A7;
    /// The error dialog's last line on a touch screen, where a tap on a way chooses it.
    pub const DIALOG_KEYS_TOUCH: u32 = FIRST + 0x1B0;
    /// The replay viewer's names of its buttons, shown when the highlight moves to one: the 26
    /// buttons of the two rows in order, the render button, the name while a camera is edited
    /// ("{}" the camera's number), a car's HUD shown or hidden ("{}" the car's number), and the
    /// three replay speeds.
    pub const VIEWER_TIPS: u32 = FIRST + 0x12A;
    pub const VIEWER_TIP_RENDER: u32 = FIRST + 0x144;
    pub const VIEWER_TIP_EDITING: u32 = FIRST + 0x145;
    pub const VIEWER_TIP_HUD_SHOW: u32 = FIRST + 0x146;
    pub const VIEWER_TIP_HUD_HIDE: u32 = FIRST + 0x147;
    pub const VIEWER_TIP_SPEEDS: u32 = FIRST + 0x148;
}

/// Each text's file key and id.
pub const KEYS: &[(&str, u32)] = &[
    ("blank", id::BLANK),
    ("axes", id::AXES),
    ("more_controls", id::MORE_CONTROLS),
    ("first_controls", id::FIRST_CONTROLS),
    ("wheel", id::WHEEL),
    ("full_rotation", id::FULL_ROTATION),
    ("degrees", id::DEGREES),
    ("yes", id::YES),
    ("no", id::NO),
    ("on", id::ON),
    ("off", id::OFF),
    ("h_shifter", id::H_SHIFTER),
    ("options.background_sound", id::BACKGROUND_SOUND),
    ("options.film_volume", id::FILM_VOLUME),
    ("options.network_port", id::NETWORK_PORT),
    ("network.password", id::NETWORK_PASSWORD),
    ("network.password_refused", id::PASSWORD_REFUSED),
    ("network.lock_mark", id::LOCK_MARK),
    ("network.password_prompt", id::PASSWORD_PROMPT),
    ("network.password_wrong", id::PASSWORD_WRONG),
    ("network.port_in_use", id::PORT_IN_USE),
    ("network.left_race", id::LEFT_RACE),
    ("network.host_left", id::HOST_LEFT),
    ("button_icons", id::BUTTON_ICONS),
    ("engine_vibration", id::ENGINE_VIBRATION),
    ("graphics.frame_limit", id::FRAME_LIMIT),
    ("graphics.no_limit", id::NO_LIMIT),
    ("graphics.display", id::DISPLAY),
    ("graphics.frames_a_second", id::FRAMES_A_SECOND),
    ("advanced.smooth_frames", id::SMOOTH_FRAMES),
    ("advanced.saved_game_controls", id::SAVE_CONTROLS),
    ("advanced.save_every_stage", id::SAVE_EVERY_STAGE),
    ("advanced.more", id::ADVANCED_MORE),
    ("advanced.first", id::ADVANCED_FIRST),
    ("split_screen.three", id::SPLIT_THREE),
    ("split_screen.four", id::SPLIT_FOUR),
    ("graphics.render_scale", id::RENDER_SCALE),
    ("graphics.hud_box", id::HUD_BOX),
    ("graphics.window_mode", id::WINDOW_MODE),
    ("graphics.window", id::WINDOW),
    ("graphics.full_screen", id::FULL_SCREEN),
    ("graphics.borderless", id::BORDERLESS),
    ("graphics.custom", id::CUSTOM),
    ("lobby.save_replay", id::SAVE_REPLAY),
    ("lobby.settings", id::LOBBY_SETTINGS),
    ("lobby.replay_not_saved", id::REPLAY_NOT_SAVED),
    ("replays.old", id::REPLAY_OLD),
    ("lobby.bots_0", id::BOTS),
    ("lobby.bots_1", id::BOTS + 1),
    ("lobby.bots_2", id::BOTS + 2),
    ("lobby.bots_3", id::BOTS + 3),
    ("lobby.bots_4", id::BOTS + 4),
    ("lobby.bots_5", id::BOTS + 5),
    ("lobby.bots_6", id::BOTS + 6),
    ("lobby.bots_7", id::BOTS + 7),
    ("lobby.bot_start_grid", id::BOT_START),
    ("lobby.bot_start_arcade", id::BOT_START + 1),
    ("lobby.not_ready", id::NOT_READY),
    ("viewer.render_title", id::RENDER_TITLE),
    ("viewer.render_size", id::RENDER_SIZE),
    ("viewer.render_fps", id::RENDER_FPS),
    ("viewer.render_part", id::RENDER_PART),
    ("viewer.render_quality", id::RENDER_QUALITY),
    ("viewer.render_sound", id::RENDER_SOUND),
    ("viewer.render_file", id::RENDER_FILE),
    ("viewer.render_folder", id::RENDER_FOLDER),
    ("viewer.render_go", id::RENDER_GO),
    ("viewer.render_cancel", id::RENDER_CANCEL),
    ("viewer.render_screen", id::RENDER_SCREEN),
    ("viewer.render_whole", id::RENDER_WHOLE),
    ("viewer.render_from_here", id::RENDER_FROM_HERE),
    ("viewer.render_marked", id::RENDER_MARKED),
    ("viewer.render_quality_low", id::RENDER_QUALITIES),
    ("viewer.render_quality_medium", id::RENDER_QUALITIES + 1),
    ("viewer.render_quality_high", id::RENDER_QUALITIES + 2),
    ("viewer.render_quality_best", id::RENDER_QUALITIES + 3),
    ("viewer.render_no_encoder", id::RENDER_NO_ENCODER),
    ("viewer.render_keys", id::RENDER_KEYS),
    ("viewer.render_progress_title", id::RENDER_PROGRESS_TITLE),
    ("viewer.render_percent", id::RENDER_PERCENT),
    ("viewer.render_frames", id::RENDER_FRAMES),
    ("viewer.render_starting", id::RENDER_STARTING),
    ("viewer.render_done", id::RENDER_DONE),
    ("viewer.render_failed", id::RENDER_FAILED),
    ("viewer.render_speed", id::RENDER_SPEED),
    ("viewer.render_left", id::RENDER_LEFT),
    ("viewer.render_encoder", id::RENDER_ENCODER),
    ("viewer.render_encoder_automatic", id::RENDER_ENCODERS),
    ("viewer.render_encoder_software", id::RENDER_ENCODERS + 1),
    ("viewer.render_no_ffmpeg", id::RENDER_NO_FFMPEG),
    ("viewer.render_encoder_failed", id::RENDER_ENCODER_FAILED),
    ("viewer.render_encoder_redone", id::RENDER_ENCODER_REDONE),
    ("viewer.render_sound_failed", id::RENDER_SOUND_FAILED),
    ("dialog.error", id::DIALOG_ERROR),
    ("dialog.notice", id::DIALOG_NOTICE),
    ("dialog.quit", id::DIALOG_QUIT),
    ("dialog.continue", id::DIALOG_CONTINUE),
    ("dialog.log", id::DIALOG_LOG),
    ("dialog.keys", id::DIALOG_KEYS),
    ("dialog.damaged_savegame", id::DAMAGED_SAVEGAME),
    ("dialog.damaged_replay", id::DAMAGED_REPLAY),
    ("dialog.settings_reset", id::SETTINGS_RESET),
    ("dialog.main_menu", id::DIALOG_MENU),
    ("dialog.bad_picture", id::BAD_PICTURE),
    ("dialog.bad_sound", id::BAD_SOUND),
    ("dialog.bad_peer", id::BAD_PEER),
    ("dialog.bad_replay_play", id::BAD_REPLAY_PLAY),
    ("dialog.back_to_menu", id::BACK_TO_MENU),
    ("advanced.title", id::ADVANCED),
    ("advanced.threads", id::ADVANCED_ROWS),
    ("advanced.renderer", id::ADVANCED_ROWS + 1),
    ("advanced.frame_counter", id::ADVANCED_ROWS + 2),
    ("advanced.films", id::ADVANCED_ROWS + 3),
    ("advanced.screenshots", id::ADVANCED_ROWS + 4),
    ("advanced.nonlinear_steering", id::ADVANCED_ROWS + 5),
    ("advanced.automatic_neutral", id::ADVANCED_ROWS + 6),
    ("advanced.rain_and_snow", id::ADVANCED_ROWS + 7),
    ("advanced.quit_pictures", id::ADVANCED_ROWS + 10),
    ("advanced.gpu", id::RENDERER_GPU),
    ("advanced.software", id::RENDERER_SOFTWARE),
    ("advanced.gles", id::RENDERER_GLES),
    ("advanced.gl", id::RENDERER_GL),
    ("advanced.next_start", id::NEXT_START),
    ("advanced.no_gpu", id::NO_GPU),
    ("advanced.apply", id::APPLY),
    ("advanced.delete_data", id::DELETE_DATA),
    ("advanced.share_log", id::SHARE_LOG),
    ("dialog.delete", id::DELETE_ASK),
    ("dialog.delete_title", id::DELETE_TITLE),
    ("dialog.yes", id::DIALOG_YES),
    ("dialog.no", id::DIALOG_NO),
    ("dialog.delete_done", id::DELETE_DONE),
    ("dialog.delete_failed", id::DELETE_FAILED),
    ("dialog.keys_touch", id::DIALOG_KEYS_TOUCH),
    ("icon_styles.automatic", id::ICON_STYLES),
    ("icon_styles.xbox", id::ICON_STYLES + 1),
    ("icon_styles.playstation", id::ICON_STYLES + 2),
    ("icon_styles.steam_deck", id::ICON_STYLES + 3),
    ("icon_styles.numbers", id::ICON_STYLES + 4),
    ("kick.not_host", id::KICK_ANSWERS),
    ("kick.no_player", id::KICK_ANSWERS + 1),
    ("kick.done", id::KICK_ANSWERS + 2),
    ("kick.failed", id::KICK_ANSWERS + 3),
    ("hints.yes", id::HINTS),
    ("hints.no", id::HINTS + 1),
    ("hints.type", id::HINTS + 2),
    ("hints.delete", id::HINTS + 3),
    ("hints.cancel", id::HINTS + 4),
    ("hints.done", id::HINTS + 5),
    ("joystick_axis.x", id::JOYSTICK_AXIS),
    ("joystick_axis.y", id::JOYSTICK_AXIS + 1),
    ("joystick_axis.z", id::JOYSTICK_AXIS + 2),
    ("joystick_axis.rx", id::JOYSTICK_AXIS + 3),
    ("joystick_axis.ry", id::JOYSTICK_AXIS + 4),
    ("joystick_axis.rz", id::JOYSTICK_AXIS + 5),
    ("pad_axis.left_stick_x", id::PAD_AXIS),
    ("pad_axis.left_stick_y", id::PAD_AXIS + 1),
    ("pad_axis.right_stick_x", id::PAD_AXIS + 2),
    ("pad_axis.right_stick_y", id::PAD_AXIS + 3),
    ("pad_axis.left_trigger", id::PAD_AXIS + 4),
    ("pad_axis.right_trigger", id::PAD_AXIS + 5),
    ("axis_page.axis", id::SETTINGS),
    ("axis_page.dead_zone", id::SETTINGS + 1),
    ("axis_page.sensitivity", id::SETTINGS + 2),
    ("axis_page.range", id::SETTINGS + 3),
    ("axis_page.invert", id::SETTINGS + 4),
    ("axis_page.position", id::SETTINGS + 5),
    ("sensitivity.linear", id::LINEAR),
    ("sensitivity.gentle", id::GENTLE),
    ("sensitivity.quick", id::QUICK),
    ("control_rows.steering_axis", id::CONTROL_ROWS),
    ("control_rows.accelerate_axis", id::CONTROL_ROWS + 1),
    ("control_rows.brake_axis", id::CONTROL_ROWS + 2),
    ("control_rows.steer_left_button", id::CONTROL_ROWS + 3),
    ("control_rows.steer_right_button", id::CONTROL_ROWS + 4),
    ("control_rows.accelerate_button", id::CONTROL_ROWS + 5),
    ("control_rows.brake_button", id::CONTROL_ROWS + 6),
    ("control_rows.gear_up", id::CONTROL_ROWS + 7),
    ("control_rows.gear_down", id::CONTROL_ROWS + 8),
    ("control_rows.handbrake", id::CONTROL_ROWS + 9),
    ("control_rows.camera", id::CONTROL_ROWS + 10),
    ("control_rows.pause", id::CONTROL_ROWS + 11),
    ("control_rows.lights", id::CONTROL_ROWS + 12),
    ("control_rows.reset_car", id::CONTROL_ROWS + 13),
    ("gear_rows.gear_1", id::GEAR_ROWS),
    ("gear_rows.gear_2", id::GEAR_ROWS + 1),
    ("gear_rows.gear_3", id::GEAR_ROWS + 2),
    ("gear_rows.gear_4", id::GEAR_ROWS + 3),
    ("gear_rows.gear_5", id::GEAR_ROWS + 4),
    ("gear_rows.gear_6", id::GEAR_ROWS + 5),
    ("gear_rows.gear_7", id::GEAR_ROWS + 6),
    ("gear_rows.reverse", id::GEAR_ROWS + 7),
    ("menu_rows.enter", id::MENU_ROWS),
    ("menu_rows.back", id::MENU_ROWS + 1),
    ("menu_rows.up", id::MENU_ROWS + 2),
    ("menu_rows.down", id::MENU_ROWS + 3),
    ("menu_rows.left", id::MENU_ROWS + 4),
    ("menu_rows.right", id::MENU_ROWS + 5),
    ("menu_rows.third_answer", id::MENU_ROWS + 6),
    ("menu_rows.tab", id::MENU_ROWS + 7),
    ("prompt.steering", id::PROMPT_STEERING),
    ("prompt.axis", id::PROMPT_AXIS),
    ("prompt.button", id::PROMPT_BUTTON),
    ("save_code.item", id::SAVE_CODE_ITEM),
    ("save_code.title", id::SAVE_CODE_TITLE),
    ("save_code.copy", id::SAVE_CODE_COPY),
    ("save_code.paste", id::SAVE_CODE_PASTE),
    ("save_code.camera", id::SAVE_CODE_CAMERA),
    ("save_code.back", id::SAVE_CODE_BACK),
    ("save_code.keys", id::SAVE_CODE_KEYS),
    ("save_code.qr_whole", id::SAVE_CODE_QR_WHOLE),
    ("save_code.length", id::SAVE_CODE_LENGTH),
    ("save_code.copied", id::SAVE_CODE_COPIED),
    ("save_code.clipboard_empty", id::SAVE_CODE_CLIPBOARD_EMPTY),
    ("save_code.bad_code", id::SAVE_CODE_BAD_CODE),
    ("save_code.no_camera", id::SAVE_CODE_NO_CAMERA),
    ("save_code.looking", id::SAVE_CODE_LOOKING),
    ("save_code.camera_denied", id::SAVE_CODE_CAMERA_DENIED),
    ("save_code.confirm_title", id::SAVE_CODE_CONFIRM_TITLE),
    ("save_code.in_code", id::SAVE_CODE_IN_CODE),
    ("save_code.now", id::SAVE_CODE_NOW),
    ("save_code.row_made", id::SAVE_CODE_ROW_MADE),
    ("save_code.row_program", id::SAVE_CODE_ROW_PROGRAM),
    ("save_code.row_best", id::SAVE_CODE_ROW_BEST),
    ("save_code.row_setups", id::SAVE_CODE_ROW_SETUPS),
    ("save_code.row_games", id::SAVE_CODE_ROW_GAMES),
    ("save_code.yes", id::SAVE_CODE_YES),
    ("save_code.no", id::SAVE_CODE_NO),
    ("save_code.done", id::SAVE_CODE_DONE),
    ("save_code.failed", id::SAVE_CODE_FAILED),
    ("save_code.not_held", id::SAVE_CODE_NOT_HELD),
    ("save_code.confirm_keys", id::SAVE_CODE_CONFIRM_KEYS),
    ("save_code.note", id::SAVE_CODE_NOTE),
    ("save_code.parts_note", id::SAVE_CODE_PARTS_NOTE),
    ("save_code.part_of", id::SAVE_CODE_PART_OF),
    ("save_code.parts_read", id::SAVE_CODE_PARTS_READ),
    ("save_code.parts_held", id::SAVE_CODE_PARTS_HELD),
    ("save_code.part_other", id::SAVE_CODE_PART_OTHER),
    ("save_code.too_long", id::SAVE_CODE_TOO_LONG),
    ("save_code.small_button", id::SAVE_CODE_SMALL_BUTTON),
    ("save_code.large_button", id::SAVE_CODE_LARGE_BUTTON),
    ("save_code.large_note", id::SAVE_CODE_LARGE_NOTE),
    ("save_code.small_note", id::SAVE_CODE_SMALL_NOTE),
    ("save_code.camera_name", id::SAVE_CODE_CAMERA_NAME),
    ("save_code.camera_switch", id::SAVE_CODE_CAMERA_SWITCH),
    (
        "save_code.camera_switch_touch",
        id::SAVE_CODE_CAMERA_SWITCH_TOUCH,
    ),
    ("viewer.tip_render", id::VIEWER_TIP_RENDER),
    ("viewer.tip_editing", id::VIEWER_TIP_EDITING),
    ("viewer.tip_hud_show", id::VIEWER_TIP_HUD_SHOW),
    ("viewer.tip_hud_hide", id::VIEWER_TIP_HUD_HIDE),
    ("viewer.tip_speed_normal", id::VIEWER_TIP_SPEEDS),
    ("viewer.tip_speed_half", id::VIEWER_TIP_SPEEDS + 1),
    ("viewer.tip_speed_quarter", id::VIEWER_TIP_SPEEDS + 2),
    ("viewer.tip_0", id::VIEWER_TIPS),
    ("viewer.tip_1", id::VIEWER_TIPS + 1),
    ("viewer.tip_2", id::VIEWER_TIPS + 2),
    ("viewer.tip_3", id::VIEWER_TIPS + 3),
    ("viewer.tip_4", id::VIEWER_TIPS + 4),
    ("viewer.tip_5", id::VIEWER_TIPS + 5),
    ("viewer.tip_6", id::VIEWER_TIPS + 6),
    ("viewer.tip_7", id::VIEWER_TIPS + 7),
    ("viewer.tip_8", id::VIEWER_TIPS + 8),
    ("viewer.tip_9", id::VIEWER_TIPS + 9),
    ("viewer.tip_10", id::VIEWER_TIPS + 10),
    ("viewer.tip_11", id::VIEWER_TIPS + 11),
    ("viewer.tip_12", id::VIEWER_TIPS + 12),
    ("viewer.tip_13", id::VIEWER_TIPS + 13),
    ("viewer.tip_14", id::VIEWER_TIPS + 14),
    ("viewer.tip_15", id::VIEWER_TIPS + 15),
    ("viewer.tip_16", id::VIEWER_TIPS + 16),
    ("viewer.tip_17", id::VIEWER_TIPS + 17),
    ("viewer.tip_18", id::VIEWER_TIPS + 18),
    ("viewer.tip_19", id::VIEWER_TIPS + 19),
    ("viewer.tip_20", id::VIEWER_TIPS + 20),
    ("viewer.tip_21", id::VIEWER_TIPS + 21),
    ("viewer.tip_22", id::VIEWER_TIPS + 22),
    ("viewer.tip_23", id::VIEWER_TIPS + 23),
    ("viewer.tip_24", id::VIEWER_TIPS + 24),
    ("viewer.tip_25", id::VIEWER_TIPS + 25),
];

/// The English file, built in.
pub(crate) const ENGLISH: &str = include_str!("../../../assets/locale/english.rc2k.toml");

/// Every language's file, built in: the language's name (the `.LOC` file's, in lower case) and
/// the file's text.
pub(crate) const LANGUAGES: &[(&str, &str)] = &[
    ("english", ENGLISH),
    (
        "portuges",
        include_str!("../../../assets/locale/portuges.rc2k.toml"),
    ),
];

/// The texts of a language file, as Windows-1252 bytes by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct File {
    texts: HashMap<u32, Vec<u8>>,
    /// The keys the file has that `KEYS` does not.
    pub unknown: Vec<String>,
}

/// The keys of `table` and their texts, each key with the tables above it joined by dots.
fn flatten(
    table: &toml::Table,
    prefix: &str,
    into: &mut Vec<(String, String)>,
) -> Result<(), String> {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            toml::Value::String(text) => into.push((key, text.clone())),
            toml::Value::Table(inner) => flatten(inner, &key, into)?,
            _ => return Err(format!("{key} is not a text")),
        }
    }
    Ok(())
}

impl File {
    /// A language file's texts. Refused: text that is not TOML, a value that is not a text.
    pub fn parse(text: &str) -> Result<File, String> {
        let table: toml::Table = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
        let mut pairs = Vec::new();
        flatten(&table, "", &mut pairs)?;
        let mut file = File::default();
        for (key, text) in pairs {
            match KEYS.iter().find(|(known, _)| *known == key) {
                Some(&(_, id)) => {
                    file.texts.insert(id, crate::files::ansi::ansi_bytes(&text));
                }
                None => file.unknown.push(key),
            }
        }
        Ok(file)
    }

    /// The text of own id `id` if the file has it.
    pub fn get(&self, id: u32) -> Option<&[u8]> {
        self.texts.get(&id).map(Vec::as_slice)
    }
}

/// The built-in English texts.
pub fn english() -> &'static File {
    static ENGLISH_TEXTS: OnceLock<File> = OnceLock::new();
    ENGLISH_TEXTS.get_or_init(|| {
        // The file is part of the program: a test keeps it parsing.
        File::parse(ENGLISH).unwrap_or_default()
    })
}

/// The built-in texts of the language named `name` (any case), parsed once; none for a language
/// with no file, whose texts are English.
pub fn for_language(name: &str) -> Option<&'static File> {
    static PARSED: OnceLock<Vec<(&'static str, File)>> = OnceLock::new();
    let parsed = PARSED.get_or_init(|| {
        LANGUAGES
            .iter()
            // A file that does not parse is kept out; a test keeps every one parsing.
            .filter_map(|(name, text)| Some((*name, File::parse(text).ok()?)))
            .collect()
    });
    parsed
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|(_, file)| file)
}

/// The text of own id `id`: the language's file's, else English; none for an id that is none's.
pub fn text(id: u32, language: Option<&File>) -> Option<&[u8]> {
    language
        .and_then(|file| file.get(id))
        .or_else(|| english().get(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_keys_are_unique_and_inside_the_range() {
        for (n, (key, id)) in KEYS.iter().enumerate() {
            assert!(is_own(*id), "{key}");
            assert!(KEYS[n + 1..].iter().all(|(k, i)| k != key && i != id));
        }
    }

    #[test]
    fn the_english_file_has_every_text() {
        let file = File::parse(ENGLISH).unwrap();
        assert!(file.unknown.is_empty(), "{:?}", file.unknown);
        for (key, id) in KEYS {
            assert!(file.get(*id).is_some(), "english lacks {key}");
        }
        assert_eq!(english(), &file);
    }

    #[test]
    fn every_shipped_file_has_known_keys_and_texts() {
        for (name, text) in LANGUAGES {
            let file = File::parse(text).unwrap();
            assert!(file.unknown.is_empty(), "{name}: {:?}", file.unknown);
            for (key, id) in KEYS {
                assert!(file.get(*id).is_some(), "{name} lacks {key}");
            }
        }
    }

    #[test]
    fn texts_are_windows_1252_and_fall_back_to_english() {
        let portuguese =
            File::parse("axes = \"eix\u{f5}s\"\nbogus = \"x\"\n[prompt]\nbutton = \"b\"").unwrap();
        assert_eq!(portuguese.unknown, vec!["bogus".to_string()]);
        assert_eq!(text(id::AXES, Some(&portuguese)), Some(&b"eix\xF5s"[..]));
        assert_eq!(text(id::PROMPT_BUTTON, Some(&portuguese)), Some(&b"b"[..]));
        // Not in the file: English.
        assert_eq!(text(id::WHEEL, Some(&portuguese)), Some(&b"wheel"[..]));
        assert_eq!(text(id::WHEEL, None), Some(&b"wheel"[..]));
        assert_eq!(text(FIRST + 0xF0, None), None);
    }

    #[test]
    fn a_language_is_found_by_its_name_in_any_case() {
        assert_eq!(for_language("PORTUGES").unwrap().unknown.len(), 0);
        assert_eq!(for_language("English"), Some(english()));
        assert!(for_language("klingon").is_none());
        assert_ne!(for_language("portuges"), for_language("english"));
    }

    #[test]
    fn a_value_that_is_not_a_text_is_refused() {
        assert!(File::parse("axes = 3").is_err());
        assert!(File::parse("axes = ").is_err());
    }
}
