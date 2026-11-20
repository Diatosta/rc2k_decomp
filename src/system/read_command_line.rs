//! Takes the switches off the command line: the first word is the program (its length goes to
//! `COMMAND_LINE`); of the words after it only those of two characters or more count, by their
//! first two. `-v` shows a message box with the version and leaves; `-u` and `-p` are returned in
//! the `Command`, `-a` sets `no_audio`; `-i` takes the next two words as the game and installed
//! folder paths and the two after as the display mode and language; `-c` takes its word and
//! does nothing. Refuses a word for `-i` of
//! 0x100 bytes or more. The `-v` part cannot be run in a test.

use crate::errors::BadInput;
use crate::install::run_flags::{Command, GivenInstall};
use crate::memory::session::PlainDword;
use crate::system::owner::Start;

/// The `-v` part: the box, and the program ended. Does not return.
fn show_version_and_leave(start: &Start) -> ! {
    let leave = |e: BadInput| -> ! { crate::fatal::raise::end("system::read_command_line", e) };
    let caption = &crate::fatal::state::CAPTION;
    // The `:` of the caption text is cut (the title is "Rally" alone).
    caption.set(5, 0);
    // The text from its seventh byte up to its NUL.
    let version = start.texts.version.read();
    let Some(end) = version[6..].iter().position(|&b| b == 0) else {
        leave(BadInput("the version without its end in its block"))
    };
    let mut text = version[6..6 + end].to_vec();
    text.extend(*b"\t\r\n");
    let mut kept = text.clone();
    kept.push(0);
    crate::fatal::state::TEXT
        .write_at(0, &kept, "the version's text")
        .unwrap_or_else(|e| leave(e));
    let caption = crate::files::cstring(caption, "the caption").unwrap_or_else(|e| leave(e));
    crate::platform::os::message_box(crate::platform::os::Icon::Information, &caption, &text);
    crate::standalone::finish("the version was shown", 0)
}

/// Reads a command line (its text with the NUL), keeps the program word's length, sets the
/// audio switch and copies the `-i` paths into the installer's folders;
/// returns what else it asked for.
pub fn read_command_line(
    start: &Start,
    install: &crate::install::owner::Install,
    audio: &crate::audio::owner::Audio,
    line: &[u8],
) -> Result<Command, BadInput> {
    let sw = rc2k_core::system::command_line::read(line)?;
    start.command_line.program_len.set(sw.program.1 as u32);
    if sw.no_audio {
        audio.state.no_audio.set(1);
    }
    let mut given = None;
    for i in &sw.install {
        for (place, (start, len)) in [&install.paths.game_dir, &install.paths.installed]
            .into_iter()
            .zip(i.words)
        {
            let word = line
                .get(start..start + len)
                .ok_or(BadInput("a word past the command line"))?;
            place.write_at(0, word, "an installer path")?;
        }
        given = Some(GivenInstall {
            mode: i.digits[0] as i32,
            language: i.digits[1] as i32,
        });
    }
    Ok(Command {
        show_version: sw.version,
        updating: sw.u,
        installing: sw.installing,
        given,
    })
}

/// Reads the program's own command line (`os::command_line`); `-v` shows the version and ends
/// the program.
pub fn read_own_command_line(o: crate::owners::Owners<'static>) -> Result<(), BadInput> {
    let mut line = crate::platform::os::command_line();
    line.push(0);
    let start = o.system.start();
    let command = read_command_line(start, o.install, o.audio, &line)?;
    o.install.flags.begin(&command);
    if command.show_version {
        show_version_and_leave(start);
    }
    Ok(())
}

/// The program's name on the command line, as `read_command_line` found it.
pub struct CommandLine {
    /// The program word's length.
    pub program_len: PlainDword,
}

impl CommandLine {
    pub const fn new() -> Self {
        CommandLine {
            program_len: PlainDword::new(0x0),
        }
    }
}
