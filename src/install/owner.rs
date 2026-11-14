//! The installer's owner: its state in one struct, handed by reference to the code that runs it.
//! `install()` is the boundary: a caller outside the module fetches the owner there.

use crate::install::installer::InstallTexts;
use crate::install::name_tables::NameTable;
use crate::install::read_sku::Sku;
use crate::install::run_flags::RunFlags;
use crate::install::store::{DriveRows, InstallVars, LobbyKeys};
use crate::install::vars::{
    InstallerNames, InstallerPages, InstallerPalette, InstallerPaths, InstallerRun,
};
use crate::memory::owned::Owned;
use crate::session::GameThread;

/// The installation's state.
pub struct Install {
    pub texts: InstallTexts,
    pub sku: Sku,
    pub vars: InstallVars,
    pub drive_rows: DriveRows,
    pub run: InstallerRun,
    pub flags: RunFlags,
    pub names: InstallerNames,
    pub pages: InstallerPages,
    pub paths: InstallerPaths,
    pub palette: InstallerPalette,
    /// The lobby's registry key and value names (read-only texts).
    pub lobby_keys: LobbyKeys,
    /// The folders' names (0x10100 bytes) the installer's scans and script collect.
    pub folders: Owned<NameTable>,
    /// The files' names (0x40100 bytes) the same collect.
    pub files: Owned<NameTable>,
}

/// The one instance, reached through `install()`.
static INSTALL: GameThread<Install> = GameThread::new(Install {
    texts: InstallTexts::new(),
    sku: Sku::new(),
    vars: InstallVars::new(),
    drive_rows: DriveRows::new(),
    run: InstallerRun::new(),
    flags: RunFlags::new(),
    names: InstallerNames::new(),
    pages: InstallerPages::new(),
    paths: InstallerPaths::new(),
    palette: InstallerPalette::new(),
    lobby_keys: LobbyKeys::new(),
    folders: Owned::new("no installer folders table"),
    files: Owned::new("no installer files table"),
});

/// The installation's state; pass it down where a function reaches it more than once.
pub fn install() -> &'static Install {
    INSTALL.get()
}
