//! The installation's record, `var\install.cfg`: the program is portable, a folder that needs
//! nothing outside it, so what an installation is lives in one small text file beside the game.
//! It holds the status (1 installed); the folder installed from is not kept: read, it is always
//! the folder the program runs in, so that a copied or moved folder is still its own
//! installation. Started from somewhere that is no installation (a CD), there is no record to
//! find: the installer offers a new installation.

use std::io;
use std::path::Path;

/// The record, relative to an installation's folder.
pub const FILE: &str = "var\\install.cfg";

/// The most bytes of the installed folder's text the installer's source place holds (its NUL
/// included).
const SOURCE_ROOM: u32 = 0xFF;

/// Whether the program runs in the game's download folder (the CD's files, as it is
/// bundled), not an installation: the game's data (`files\d\gfx\jade.bun`) but neither
/// `var\ral.zog` nor `var\install.cfg`. The main loop then enters the installer (owner:
/// the folder in place of a CD). Never on a phone or tablet.
pub fn is_source_folder() -> bool {
    // A phone has no installer: the folder's files are played where they are.
    if cfg!(phone) {
        return false;
    }
    let here = |p: &str| crate::files::host_path(p.as_bytes()).exists();
    here("files\\d\\gfx\\jade.bun") && !here("var\\ral.zog") && !here(FILE)
}

/// What the record holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallRecord {
    /// 1 installed; 0x80 an installation or an uninstall under way; 0 none.
    pub status: u8,
}

impl InstallRecord {
    /// The record in a text (`status=N`, any other line ignored), if it has a status.
    pub fn parse(text: &str) -> Option<InstallRecord> {
        text.lines()
            .find_map(|line| {
                let value = line
                    .trim()
                    .strip_prefix("status")?
                    .trim_start()
                    .strip_prefix('=')?;
                value.trim().parse::<u8>().ok()
            })
            .map(|status| InstallRecord { status })
    }

    /// The text the record is kept as.
    pub fn text(&self) -> String {
        format!(
            "; Rally Championship\r\n; INSTALLATION STATUS (the standalone's, in place of the registry)\r\n\r\nstatus={}\r\n",
            self.status
        )
    }

    /// The record in the file at `path`; None without the file or a status in it.
    pub fn load(path: &Path) -> Option<InstallRecord> {
        InstallRecord::parse(&std::fs::read_to_string(path).unwrap_or_default())
    }

    /// The record written to the file at `path`.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.text())
    }
}

/// The current directory with a `\` after it.
fn here() -> Option<Vec<u8>> {
    let mut d = crate::install::normalise_path::current_directory().ok()??;
    if !matches!(d.last(), Some(b'\\' | b'/')) {
        d.push(b'\\');
    }
    Some(d)
}

/// What the installation's record says, put in the installer's names: the status and the
/// source, which is the folder the program runs in (with its NUL). Nothing changes without a
/// record, or without a status in it.
pub(super) fn read_installed(install: &crate::install::owner::Install) {
    let names = &install.names;
    if !crate::files::host_path(FILE.as_bytes()).exists() {
        return;
    }
    let Some(record) = InstallRecord::load(&crate::files::host_path(FILE.as_bytes())) else {
        return;
    };
    names.status.set(record.status);
    let Some(mut folder) = here() else {
        return;
    };
    folder.push(0);
    if folder.len() as u32 <= SOURCE_ROOM {
        // Fits: its length is under the place's.
        let _ = names.source.set(&folder);
    }
}

/// The installer's status written as the record: in the folder installed from, else in the
/// current one, if that folder has a `var\` (an installation's). The result is logged.
pub(super) fn write_installed(install: &crate::install::owner::Install) {
    let names = &install.names;
    let status = names.status.get();
    let folder = names.source.text();
    let mut var = folder.clone();
    if !var.is_empty() && !var.ends_with(b"\\") {
        var.push(b'\\');
    }
    var.extend(b"var");
    let var = crate::files::host_path(&var);
    let folder = crate::files::ansi_text(&folder);
    let what = if var.is_dir() {
        match (InstallRecord { status }).save(&var.join("install.cfg")) {
            Ok(()) => "written".to_string(),
            Err(e) => format!("not written: {e}"),
        }
    } else {
        "not written: no var folder there".to_string()
    };
    crate::program_log::log(&format!(
        "install: the installation's record (status {status}) in \"{folder}\": {what}"
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes of a record as the installer wrote them before the record was a type.
    const WRITTEN_EARLIER: &str = "; Rally Championship\r\n; INSTALLATION STATUS (the standalone's, in place of the registry)\r\n\r\nstatus=1\r\n";

    #[test]
    fn a_record_written_earlier_reads_the_same_and_writes_the_same_bytes() {
        let read = InstallRecord::parse(WRITTEN_EARLIER).unwrap();
        assert_eq!(read, InstallRecord { status: 1 });
        assert_eq!(read.text(), WRITTEN_EARLIER);
    }

    #[test]
    fn a_record_goes_through_a_file_and_back() {
        let dir = std::env::temp_dir().join(format!("rc2k_record_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("install.cfg");
        assert_eq!(InstallRecord::load(&path), None);
        for status in [0, 1, 0x80, 255] {
            let saved = InstallRecord { status };
            saved.save(&path).unwrap();
            assert_eq!(InstallRecord::load(&path), Some(saved));
            assert_eq!(std::fs::read_to_string(&path).unwrap(), saved.text());
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_text_without_a_status_is_no_record() {
        assert_eq!(InstallRecord::parse(""), None);
        assert_eq!(InstallRecord::parse("; status=1\r\nstate=1\r\n"), None);
        assert_eq!(InstallRecord::parse("status=many\r\n"), None);
        assert_eq!(InstallRecord::parse("status=300\r\n"), None);
        // The line may hold blanks around the sign, and the record other lines.
        assert_eq!(
            InstallRecord::parse("a=1\r\n  status = 7 \r\n"),
            Some(InstallRecord { status: 7 })
        );
    }
}
