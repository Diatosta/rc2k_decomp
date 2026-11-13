//! `system::read_command_line` (0x410684): takes the switches off the command line.
//!
//! The first word is the program (its length goes to `COMMAND_LINE`; the original also
//! kept its place, which nothing reads).
//! Of the words after it only those of two characters or more count, by their first two:
//! - `-v`: a message box with the version ("Rally 5.30 (4.8.2000)", a tab, a line break and
//!   the second version text, built in the fatal report's `Text`; the title is the
//!   report's "Rally: Guru Meditation" cut after "Rally"), and the game leaves through
//!   `start`'s exit path without coming back here. The words after it are not read;
//! - `-u`: `run.updating` = 1; `-p`: `run.installing` = 1; `-a`: `no_audio` = 1;
//! - `-i`: `run.from_command` and `run.ignore_room` = 1, the next two words are copied
//!   (without a NUL) to the game and installed folder paths, and the first characters of
//!   the two after them, less `0`, become the given display mode and language (0 for a
//!   word that is not there);
//! - `-c`: the next word's first character: `1` sets `cards.ignore_secondary`
//!   (`ignoresecondary`), `2` sets `cards.not_primary` (`ignoreprimary`).
//!
//! The line ends at a word of no length that starts where the reading stands. One caller, `start`.
//! Refuses a word for `-i` of 0x100 bytes or more. The `-v` part cannot be run in a test: it was
//! looked at by the owner. `start` calls it directly.

use crate::errors::BadInput;
use crate::session::PlainDword;
use crate::system::owner::Start;

// ---- game data ----

const EXIT_PATH: usize = 0x401116;

/// What the line asks for besides its switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Nothing,
    /// `-v`: the version is to be shown and the game left.
    Version,
}

/// The `-v` part: the box, and out through `start`'s exit path. Does not return.
pub fn show_version_and_leave(start: &Start) -> ! {
    let leave = |e: BadInput| -> ! { crate::fatal::raise::end("system::read_command_line", e) };
    let caption = &crate::fatal::state::CAPTION;
    // The `:` of the caption text is cut (the title is "Rally" alone).
    caption.set(5, 0);
    let c_text = |at: &dyn crate::cells::Store, what| {
        crate::files::cstring(at, what).unwrap_or_else(|e| leave(e))
    };
    // The text from its seventh byte up to its NUL.
    let version = start.texts.version.read();
    let Some(end) = version[6..].iter().position(|&b| b == 0) else {
        leave(BadInput("the version without its end in its block"))
    };
    let mut text = version[6..6 + end].to_vec();
    text.extend(*b"\t\r\n");
    // The second text is empty: its NUL is its one byte.
    text.extend(c_text(&start.texts.second_text, "the second text"));
    let mut kept = text.clone();
    kept.push(0);
    crate::fatal::state::TEXT // Text
        .write_at(0, &kept, "the version's text")
        .unwrap_or_else(|e| leave(e));
    // MB_ICONINFORMATION | MB_TASKMODAL.
    let caption = c_text(caption, "the caption");
    crate::os::message_box(crate::os::Icon::Information, &caption, &text);
    // The exit path ends in `ExitProcess`; so does the standalone, here.
    crate::standalone::finish(&format!("the game's way out {EXIT_PATH:#x} was reached"), 0)
}

/// Reads a command line (its text with the NUL; kept for the program's
/// life, as the game keeps a pointer into it).
pub fn read_command_line(
    start: &Start,
    install: &crate::install::owner::Install,
    d3d: &crate::d3d::owner::D3d,
    audio: &crate::audio::owner::Audio,
    line: &[u8],
) -> Result<Asked, BadInput> {
    // Converted: `rc2k_core::system::command_line::read`, the whole line read before
    // anything is written (a refused `-i` word now leaves nothing written).
    let sw = rc2k_core::system::command_line::read(line)?;
    start.command_line.program_len.set(sw.program.1 as u32);
    if sw.u {
        install.run.updating.set(1);
    }
    if sw.installing {
        install.run.installing.set(1);
    }
    if sw.no_audio {
        audio.state.no_audio.set(1);
    }
    for i in &sw.install {
        install.run.from_command.set(1u32);
        install.run.ignore_room.set(1u32);
        for (place, (start, len)) in [&install.paths.game_dir, &install.paths.installed]
            .into_iter()
            .zip(i.words)
        {
            let word = line
                .get(start..start + len)
                .ok_or(BadInput("a word past the command line"))?;
            place.write_at(0, word, "an installer path")?;
        }
        install.run.given_mode.set(i.digits[0]);
        install.run.given_language.set(i.digits[1]);
    }
    if sw.ignore_secondary {
        d3d.cards.get().ignore_secondary.set(1u32);
    }
    if sw.ignore_primary {
        d3d.cards.get().not_primary.set(1u32);
    }
    Ok(if sw.version {
        Asked::Version
    } else {
        Asked::Nothing
    })
}

/// Reads the program's own command line (`os::command_line`, for
/// `GetCommandLineA`; kept as a region); `-v` shows the version and leaves through
/// `start`'s exit path.
pub fn read_own_command_line(o: crate::owners::Owners<'static>) -> Result<(), BadInput> {
    let mut line = crate::os::command_line();
    line.push(0);
    let start = o.system.start();
    if read_command_line(start, o.install, o.d3d, o.audio, &line)? == Asked::Version {
        show_version_and_leave(start);
    }
    Ok(())
}

/// The program's name on the command line, as `read_command_line` found it (the image had
/// its place and length; nothing reads the place).
pub struct CommandLine {
    /// Its length (`dword_518BAC`).
    pub program_len: PlainDword,
}

impl CommandLine {
    pub const fn new() -> Self {
        CommandLine {
            program_len: PlainDword::new(0x0),
        }
    }
}
