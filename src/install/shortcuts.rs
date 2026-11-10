//! The standalone's Start menu items and the program's copy (platform code, owner's decision
//! 2026-10-19): what `install::add_start_menu_items` (0x413E42) and
//! `install::remove_start_menu_items` (0x4140C3) ask the Program Manager by DDE is done with
//! shell shortcuts (`IShellLinkW` saved by `IPersistFile`) in the user's Start menu
//! Programs folder, group "Rally Championship" (Wine makes them menu entries):
//! - "Rally": `<installed folder>rc2k.exe` (the original: `ral.exe`);
//! - "Uninstall Rally": `<installed folder>rc2k.exe` with " -u" (the original names the
//!   program it ran from, the CD's, which finds the installation in the registry; the
//!   standalone's record is in the installed folder, so only that copy can remove it;
//!   2026-10-20);
//! - "View Readme": `<installed folder>readme.txt` (named in the case the folder holds it);
//! - the removal deletes the three and the group's folder (the original's `DeleteGroup`).
//!
//! On Unix the items are desktop entries in the user's applications folder
//! (`~/.local/share/applications`, as Wine makes them from Start menu items).
//!
//! And the program installed: `rc2k.exe` and its DLLs copied from the running program's
//! folder into the current one (the original copies the program named on its command line
//! to `.\ral.exe` when run from the CD).

use std::path::{Path, PathBuf};

/// The group's folder name (the original's `[CreateGroup(Rally Championship)]`).
pub const GROUP: &str = "Rally Championship";
/// The program's file, in place of the original's `ral.exe`.
#[cfg(windows)]
pub const PROGRAM: &str = "rc2k.exe";
#[cfg(not(windows))]
pub const PROGRAM: &str = "rc2k";
/// The libraries the program needs next to it (as `deploy_standalone.sh` copies them;
/// on Unix the system's).
#[cfg(windows)]
pub const DLLS: &[&str] = &["SDL3.dll", "avcodec-61.dll", "avutil-59.dll"];
#[cfg(not(windows))]
pub const DLLS: &[&str] = &[];

/// The three items: (name, target in the installed folder, arguments).
fn items(installed: &Path) -> [(&'static str, PathBuf, &'static str); 3] {
    [
        ("Rally", installed.join(PROGRAM), ""),
        ("Uninstall Rally", installed.join(PROGRAM), "-u"),
        ("View Readme", installed.join(readme_name(installed)), ""),
    ]
}

/// The readme's name as the installed folder holds it: the copy keeps the source's case
/// (`README.TXT`), which a case-sensitive system needs named exactly; else `readme.txt`.
fn readme_name(installed: &Path) -> std::ffi::OsString {
    std::fs::read_dir(installed)
        .ok()
        .and_then(|entries| {
            entries.flatten().map(|e| e.file_name()).find(|n| {
                n.to_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case("readme.txt"))
            })
        })
        .unwrap_or_else(|| "readme.txt".into())
}

/// The three items, all in `installed`, the installed folder.
pub fn add_start_menu_items(installed: &Path) {
    start_menu::add(installed);
}

/// The group and its items removed.
pub fn remove_start_menu_items() {
    start_menu::remove();
}

#[cfg(windows)]
mod start_menu {
    use super::{GROUP, items};
    use crate::ffi::win32::{programs_folder, with_com, write_link};
    use std::path::{Path, PathBuf};

    fn group_folder() -> Option<PathBuf> {
        programs_folder().map(|p| p.join(GROUP))
    }

    /// The three items, all in `installed`, the installed folder.
    pub fn add(installed: &Path) {
        let Some(group) = group_folder() else {
            crate::framework::log("install: no Start menu Programs folder; no items added");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&group) {
            crate::framework::log(&format!("install: {}: {e}", group.display()));
            return;
        }
        let items = items(installed);
        with_com(|| {
            for (name, target, arguments) in items {
                let at = group.join(format!("{name}.lnk"));
                let done = write_link(&at, &target, arguments, name);
                crate::framework::log(&format!(
                    "install: Start menu item {} -> {} {arguments}: {}",
                    at.display(),
                    target.display(),
                    if done { "written" } else { "failed" }
                ));
            }
        });
    }

    /// The group and its items removed.
    pub fn remove() {
        let Some(group) = group_folder() else {
            return;
        };
        let gone = std::fs::remove_dir_all(&group);
        crate::framework::log(&format!(
            "install: Start menu group {} removed: {gone:?}",
            group.display()
        ));
    }
}

#[cfg(unix)]
mod start_menu {
    use super::{GROUP, items};
    use std::path::{Path, PathBuf};

    /// The user's applications folder.
    fn applications() -> Option<PathBuf> {
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
        Some(data.join("applications"))
    }

    /// An item's desktop entry's file.
    fn entry_file(folder: &Path, name: &str) -> PathBuf {
        let id = format!("{GROUP} {name}").to_lowercase().replace(' ', "-");
        folder.join(format!("{id}.desktop"))
    }

    pub fn add(installed: &Path) {
        let Some(folder) = applications() else {
            crate::framework::log("install: no applications folder; no items added");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&folder) {
            crate::framework::log(&format!("install: {}: {e}", folder.display()));
            return;
        }
        for (name, target, arguments) in items(installed) {
            let at = entry_file(&folder, name);
            let quoted = |p: &Path| format!("\"{}\"", p.display());
            let exec = if target
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
            {
                format!("xdg-open {}", quoted(&target))
            } else {
                format!("{} {arguments}", quoted(&target))
                    .trim_end()
                    .to_string()
            };
            let text = format!(
                "[Desktop Entry]\nType=Application\nName={name}\nExec={exec}\nPath={}\nCategories=Game;\n",
                installed.display()
            );
            let done = std::fs::write(&at, text);
            crate::framework::log(&format!(
                "install: menu item {} -> {exec}: {}",
                at.display(),
                if done.is_ok() { "written" } else { "failed" }
            ));
        }
    }

    pub fn remove() {
        let Some(folder) = applications() else {
            return;
        };
        for (name, ..) in items(Path::new("")) {
            let at = entry_file(&folder, name);
            let gone = std::fs::remove_file(&at);
            crate::framework::log(&format!(
                "install: menu item {} removed: {gone:?}",
                at.display()
            ));
        }
    }
}

/// The program and its DLLs copied from the running program's folder into the current one,
/// unless it is the same folder. False if a copy failed.
pub fn copy_program() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let (Some(from), Ok(to)) = (exe.parent(), std::env::current_dir()) else {
        return false;
    };
    let same = match (from.canonicalize(), to.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    if same {
        return true;
    }
    let mut files = vec![(exe.clone(), to.join(PROGRAM))];
    files.extend(DLLS.iter().map(|d| (from.join(d), to.join(d))));
    let mut all = true;
    for (a, b) in files {
        let done = std::fs::copy(&a, &b);
        crate::framework::log(&format!(
            "install: {} -> {}: {}",
            a.display(),
            b.display(),
            match &done {
                Ok(_) => "copied".to_string(),
                Err(e) => e.to_string(),
            }
        ));
        all &= done.is_ok();
    }
    all
}

/// A program started, not waited for (the original's `system::run_program` with ebx 1).
pub fn run_program(command: &Path) -> bool {
    let started = std::process::Command::new(command).spawn();
    crate::framework::log(&format!(
        "install: run {}: {}",
        command.display(),
        if started.is_ok() { "started" } else { "failed" }
    ));
    started.is_ok()
}
