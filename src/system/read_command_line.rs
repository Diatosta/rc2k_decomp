//! `system::read_command_line` (0x410684): takes the switches off the command line.
//!
//! In:  nothing
//! Out: every register preserved
//! The first word is the program (its length to `COMMAND_LINE`; the original also kept its
//! place, `dword_518BA8`, which nothing reads).
//! Of the words after it only those of two characters or more count, by their first two:
//! - `-v`: a message box with the version ("Rally 5.30 (4.8.2000)", a tab, a line break and
//!   the second version text `byte_4E0077`, built in the fatal report's `Text`; the title
//!   is the report's "Rally: Guru Meditation" cut after "Rally"), and the game leaves
//!   through `start`'s exit path (0x401116) without coming back here. The words after
//!   it are not read;
//! - `-u`: `dword_51B9CC` = 1; `-p`: `g_installing` (0x51B9E0) = 1; `-a`: `g_noAudio` = 1;
//! - `-i`: `dword_51B9D0` and `dword_51BA04` = 1, the next two words are copied (without a
//!   NUL) to `byte_95E6F8` and `byte_95E7F8`, and the first characters of the two after
//!   them, less `0`, go to `dword_51BA08` and `dword_51BA0C` (0 for a word that is not
//!   there);
//! - `-c`: the next word's first character: `1` sets `dword_5AB954` (`ignoresecondary`),
//!   `2` sets `dword_5AB950` (`ignoreprimary`).
//!
//! The line ends at a word of no length that starts where the reading stands.
//! One caller, `start`. Refuses a word for `-i` of 0x100 bytes or more.
//! The `-v` part cannot be run in a test: it was looked at by the owner.
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::audio::store::AUDIO;
use crate::d3d::cards_store::CARDS_3D;
use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::session::Dword;
use crate::system::start_texts::START_TEXTS;

// ---- game data ----

const EXIT_PATH: usize = 0x401116;

// ---- Rust version ----

/// What the line asks for besides its switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Nothing,
    /// `-v`: the version is to be shown and the game left.
    Version,
}

/// The `-v` part: the box, and out through `start`'s exit path. Does not return.
pub fn show_version_and_leave() -> ! {
    let leave = |e: BadInput| -> ! { crate::fatal::raise::end("system::read_command_line", e) };
    let caption = GamePtr::to(&crate::fatal::state::CAPTION);
    // byte_4F6702: its `:` (a byte of the caption text)
    caption
        .offset(5)
        .write(0u8, "the caption")
        .unwrap_or_else(|e| leave(e));
    let c_text = |at: GamePtr, what| crate::files::cstring(at, what).unwrap_or_else(|e| leave(e));
    // The text from its seventh byte up to its NUL.
    let mut text = c_text(GamePtr::to(&START_TEXTS.version).offset(6), "the version");
    text.extend(*b"\t\r\n");
    // The second text is empty: its NUL is its one byte.
    text.extend(c_text(
        GamePtr::to(&START_TEXTS.second_text),
        "the second text",
    ));
    let mut kept = text.clone();
    kept.push(0);
    GamePtr::to(&crate::fatal::state::TEXT) // Text
        .write_bytes(&kept, "the version's text")
        .unwrap_or_else(|e| leave(e));
    // MB_ICONINFORMATION | MB_TASKMODAL.
    let caption = crate::files::cstring(caption, "the caption").unwrap_or_else(|e| leave(e));
    crate::os::message_box(crate::os::Icon::Information, &caption, &text);
    // The exit path (0x401116) ends in `ExitProcess`; so does the standalone, here.
    crate::standalone::finish(&format!("the game's way out {EXIT_PATH:#x} was reached"), 0)
}

/// The Rust version on a command line (its text with the NUL; kept for the program's
/// life, as the game keeps a pointer into it).
pub fn read_command_line(line: &[u8]) -> Result<Asked, BadInput> {
    // Converted: `rc2k_core::system::command_line::read`, the whole line read before
    // anything is written (a refused `-i` word now leaves nothing written).
    let sw = rc2k_core::system::command_line::read(line)?;
    COMMAND_LINE.program_len.set(sw.program.1 as u32);
    for (on, cell) in [
        (sw.u, &crate::install::owner::install().run.updating),
        (
            sw.installing,
            &crate::install::owner::install().run.installing,
        ),
        (sw.no_audio, &AUDIO.no_audio),
    ] {
        if on {
            cell.set(1u32);
        }
    }
    for i in &sw.install {
        crate::install::owner::install().run.from_command.set(1u32);
        crate::install::owner::install().run.ignore_room.set(1u32);
        for (place, (start, len)) in [
            &crate::install::owner::install().paths.game_dir,
            &crate::install::owner::install().paths.installed,
        ]
        .into_iter()
        .zip(i.words)
        {
            let word = line
                .get(start..start + len)
                .ok_or(BadInput("a word past the command line"))?;
            place.write_at(0, word, "an installer path")?;
        }
        crate::install::owner::install()
            .run
            .given_mode
            .set(i.digits[0]);
        crate::install::owner::install()
            .run
            .given_language
            .set(i.digits[1]);
    }
    if sw.ignore_secondary {
        CARDS_3D.ignore_secondary.set(1u32);
    }
    if sw.ignore_primary {
        CARDS_3D.not_primary.set(1u32);
    }
    Ok(if sw.version {
        Asked::Version
    } else {
        Asked::Nothing
    })
}

/// The Rust version on the program's own command line (`os::command_line`, for
/// `GetCommandLineA`; kept as a region); `-v` shows the version and leaves through
/// `start`'s exit path.
pub fn read_own_command_line() -> Result<(), BadInput> {
    let mut line = crate::os::command_line();
    line.push(0);
    if read_command_line(&line)? == Asked::Version {
        show_version_and_leave();
    }
    Ok(())
}

/// The program's name on the command line, as `read_command_line` found it (the image had
/// its place and length at `dword_518BA8` / `dword_518BAC`; nothing reads the place).
pub struct CommandLine {
    /// `dword_518BAC`: its length.
    pub program_len: Dword,
}

pub static COMMAND_LINE: CommandLine = CommandLine {
    program_len: Dword::new(0x0),
};
