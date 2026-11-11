//! Module `install`'s owner (step 11, owned state): the installer's state in one struct,
//! handed by reference to the code that runs it in place of global stores.
//!
//! `install()` is the boundary: a caller outside the module fetches the owner there. The
//! `name_tables` `Owned` tables stay where they are.

use crate::install::installer::InstallTexts;
use crate::install::read_sku::Sku;
use crate::install::store::{DriveRows, InstallVars, LobbyKeys};
use crate::install::vars::{
    InstallerNames, InstallerPages, InstallerPalette, InstallerPaths, InstallerRun,
};
use crate::session::GameThread;

/// The installation's state.
pub struct Install {
    pub texts: InstallTexts,
    pub sku: Sku,
    pub vars: InstallVars,
    pub drive_rows: DriveRows,
    pub run: InstallerRun,
    pub names: InstallerNames,
    pub pages: InstallerPages,
    pub paths: InstallerPaths,
    pub palette: InstallerPalette,
    /// The lobby's registry key and value names (read-only texts).
    pub lobby_keys: LobbyKeys,
}

/// The one instance. Reached through `install()`; `moved` registers its stores' places.
pub static INSTALL: GameThread<Install> = GameThread::new(Install {
    texts: InstallTexts::new(),
    sku: Sku::new(),
    vars: InstallVars::new(),
    drive_rows: DriveRows::new(),
    run: InstallerRun::new(),
    names: InstallerNames::new(),
    pages: InstallerPages::new(),
    paths: InstallerPaths::new(),
    palette: InstallerPalette::new(),
    lobby_keys: LobbyKeys::new(),
});

/// The installation's state; pass it down where a function reaches it more than once.
pub fn install() -> &'static Install {
    INSTALL.get()
}
