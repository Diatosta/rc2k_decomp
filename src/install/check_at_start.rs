//! Finds the game's data when the game is started from somewhere that is not an installation.
//! Nothing when the installer is running or `var\ral.zog` is there; otherwise the sound is
//! switched off, and nothing more if `files\d\gfx\jade.bun` is there. Else, if the registry
//! records an installation whose `ral.exe` exists, the installer's script is set up for an
//! update; either way the first CD-ROM drive holding the game's files becomes the current
//! directory ("Update cannot find RC CD-ROM" and a fatal error if there is none). Not the copy
//! protection's CD check. Called once, by `start`. Refused: a scratch path that is not a text
//! when the source cannot be put there.

use crate::errors::{BadInput, FatalError, Refused};
use crate::install::outside::Outside;
use crate::install::runs_from_source::{scratch_bytes, scratch_put, scratch_text};
use crate::session::Dword;
use crate::system::owner::System;

const PATH_ROOM: usize = 0x100;

/// What both checks write into the script's entry, but for the string's id.
pub fn set_script_entry(install: &crate::install::owner::Install, text_id: u16) {
    // The entry of the installer's script: two bytes, a string's id, and
    // 0xBD (the second item's text) = the installer runs.
    install.pages.main_menu.items[0].first.set(5);
    install.pages.main_menu.items[0].count.set(0x0C);
    install.pages.main_menu.items[0].text.set(text_id);
    install.pages.main_menu.items[1].text.set(0xBD);
}

/// `no_audio` is the sound's switch.
pub fn check_at_start(
    install: &crate::install::owner::Install,
    system: &System,
    race: &crate::race::owner::Race,
    outside: &mut dyn Outside,
    no_audio: &Dword,
) -> Result<(), Refused> {
    let record = install.texts.record_path.text();
    if install.flags.installing.get() || outside.file_exists(&record)? {
        return Ok(());
    }
    no_audio.set(1);
    let jade = &crate::gfx::texts::JADE[..crate::gfx::texts::JADE.len() - 1];
    if outside.file_exists(jade)? {
        return Ok(());
    }
    // `text` and its NUL into the path buffer at `at`.
    let put = |at: usize, text: &[u8]| -> Result<(), BadInput> {
        scratch_put(at, &[text, &[0]].concat())
    };
    let program = install.names.program.text();

    outside.read_registry();
    if install.names.status.get() == 1 && install.names.source.get(0) != 0 {
        // Whether the source could be put there is not asked: the program's name goes
        // after whatever text the place holds.
        if let Some(normalised) = outside.normalise_path(&install.names.source.text())? {
            scratch_put(0, &normalised)?;
        }
        let held = scratch_bytes()?;
        let Some(len) = held
            .iter()
            .position(|&c| c == 0)
            .filter(|len| len + 8 < PATH_ROOM)
        else {
            return Err(BadInput("the scratch path holds no text to go on from").into());
        };
        put(len, &program)?;
        if outside.file_exists(&scratch_text()?)? {
            // "patch6: ", then the version from the blank before its `(`.
            let version = system.start().texts.version.text();
            let from = (1..version.len())
                .find(|&n| version[n] == b'(')
                .ok_or(BadInput("a version text without a ("))?
                - 1;
            let text = [b"patch6: ", &version[from..], b"\n\0"].concat();
            race.text_buffers()
                .number_text
                .write_at(0, &text, "the number text")?;
            set_script_entry(install, 0x1006);
        }
    }

    for letter in b'a'..=b'z' {
        if outside.drive_type(letter) != 5 {
            continue;
        }
        put(0, &[letter, b':', b'\\'])?;
        put(3, &program)?;
        if !outside.file_exists(&scratch_text()?)? {
            continue;
        }
        put(3, jade)?;
        if !outside.file_exists(&scratch_text()?)? {
            continue;
        }
        put(3, b"")?;
        if !outside.set_directory(&scratch_text()?) {
            continue;
        }
        install.vars.running_from_cd.set(1);
        return Ok(());
    }
    outside.log(b"Update cannot find RC CD-ROM");
    Err(FatalError {
        code: 0x9000_2103,
        message: "Cannot find the RC CD-ROM\nPlease insert the RC CD-ROM into your CD-ROM drive and try again",
    }
    .into())
}
