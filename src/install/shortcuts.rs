//! The program's Start menu items and the program's copy (platform code): shell shortcuts in
//! the user's Start menu Programs folder, group "Rally Championship": "Rally", "Uninstall Rally"
//! (`rc2k.exe -u`; only that copy can remove the installation) and "View Readme"; the removal
//! deletes the three and the group's folder. On Unix the items are desktop entries in
//! `~/.local/share/applications`; on macOS none (the installed bundle is the application,
//! `install::mac_bundle`). And the program installed: `rc2k.exe` and its DLLs
//! copied from the running program's folder into the current one.

use std::path::Path;
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
use std::path::PathBuf;

/// The group's folder name.
#[cfg(not(any(target_os = "macos", target_os = "ios")))] // reason: macOS has no items
pub const GROUP: &str = "Rally Championship";
/// The program's file.
#[cfg(windows)]
pub const PROGRAM: &str = "rc2k.exe";
#[cfg(not(windows))]
pub const PROGRAM: &str = "rc2k";
/// The libraries the program needs next to it (as `deploy_standalone.sh` copies them;
/// on Unix the system's).
#[cfg(windows)]
pub const DLLS: &[&str] = &["SDL3.dll"];
#[cfg(not(windows))]
pub const DLLS: &[&str] = &[];

/// The three items: (name, target in the installed folder, arguments).
#[cfg(not(any(target_os = "macos", target_os = "ios")))] // reason: macOS has no items
fn items(installed: &Path) -> [(&'static str, PathBuf, &'static str); 3] {
    [
        ("Rally", installed.join(PROGRAM), ""),
        ("Uninstall Rally", installed.join(PROGRAM), "-u"),
        ("View Readme", installed.join(readme_name(installed)), ""),
    ]
}

/// The readme's name as the installed folder holds it: the copy keeps the source's case
/// (`README.TXT`), which a case-sensitive system needs named exactly; else `readme.txt`.
#[cfg(not(any(target_os = "macos", target_os = "ios")))] // reason: macOS has no items
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

pub fn add_start_menu_items(installed: &Path) {
    start_menu::add(installed);
}

pub fn remove_start_menu_items() {
    start_menu::remove();
}

#[cfg(windows)]
mod start_menu {
    use super::{GROUP, items};
    use crate::platform::ffi::win32::{programs_folder, with_com, write_link};
    use std::path::{Path, PathBuf};

    fn group_folder() -> Option<PathBuf> {
        programs_folder().map(|p| p.join(GROUP))
    }

    pub fn add(installed: &Path) {
        let Some(group) = group_folder() else {
            crate::program_log::log("install: no Start menu Programs folder; no items added");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&group) {
            crate::program_log::log(&format!("install: {}: {e}", group.display()));
            return;
        }
        let items = items(installed);
        with_com(|| {
            for (name, target, arguments) in items {
                let at = group.join(format!("{name}.lnk"));
                let done = write_link(&at, &target, arguments, name);
                crate::program_log::log(&format!(
                    "install: Start menu item {} -> {} {arguments}: {}",
                    at.display(),
                    target.display(),
                    if done { "written" } else { "failed" }
                ));
            }
        });
    }

    pub fn remove() {
        let Some(group) = group_folder() else {
            return;
        };
        let gone = std::fs::remove_dir_all(&group);
        crate::program_log::log(&format!(
            "install: Start menu group {} removed: {gone:?}",
            group.display()
        ));
    }
}

/// macOS has no items of its own: the installed bundle is the application (`install::mac_bundle`),
/// listed by Launchpad; removing it is moving it to the Bin, or the uninstaller's removal here.
/// iOS has no items and no installed program: the game runs from its app, and the game's folder
/// is the app's Documents folder, which the Files app shows.
#[cfg(target_os = "ios")]
mod start_menu {
    use std::path::Path;

    pub fn add(_installed: &Path) {}

    pub fn remove() {}
}

#[cfg(target_os = "macos")]
mod start_menu {
    use std::path::Path;

    pub fn add(_installed: &Path) {}

    pub fn remove() {
        crate::install::mac_bundle::remove();
    }
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
mod start_menu {
    use super::{GROUP, items};
    use std::path::{Path, PathBuf};

    fn data_home() -> Option<PathBuf> {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    }

    fn applications() -> Option<PathBuf> {
        Some(data_home()?.join("applications"))
    }

    /// The game's icon in the user's icon theme, named by `ICON`.
    fn icon_file() -> Option<PathBuf> {
        Some(data_home()?.join(format!("icons/hicolor/512x512/apps/{ICON}.png")))
    }

    /// The icon's name in the menu entry.
    const ICON: &str = "rally-championship";

    /// An item's entry name: "rally-championship-rally" for the game's own.
    fn entry_id(name: &str) -> String {
        format!("{GROUP} {name}").to_lowercase().replace(' ', "-")
    }

    fn entry_file(folder: &Path, name: &str) -> PathBuf {
        folder.join(format!("{}.desktop", entry_id(name)))
    }

    pub fn add(installed: &Path) {
        let Some(folder) = applications() else {
            crate::program_log::log("install: no applications folder; no items added");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&folder) {
            crate::program_log::log(&format!("install: {}: {e}", folder.display()));
            return;
        }
        let icon = icon_file().is_some_and(|at| {
            let done = at
                .parent()
                .is_some_and(|f| std::fs::create_dir_all(f).is_ok())
                && std::fs::write(&at, crate::platform::icon::PNG).is_ok();
            crate::program_log::log(&format!(
                "install: icon {}: {}",
                at.display(),
                if done { "written" } else { "failed" }
            ));
            done
        });
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
            // The game's own entry: its icon, and the identifier the window gives
            // (`platform::ffi::sdl::set_app_identifier`), so the desktop shows the window under this entry.
            let game = if name == "Rally" {
                format!(
                    "{}StartupWMClass={}\n",
                    if icon {
                        format!("Icon={ICON}\n")
                    } else {
                        String::new()
                    },
                    entry_id(name)
                )
            } else {
                String::new()
            };
            let text = format!(
                "[Desktop Entry]\nType=Application\nName={name}\nExec={exec}\nPath={}\nCategories=Game;\n{game}",
                installed.display()
            );
            let done = std::fs::write(&at, text);
            crate::program_log::log(&format!(
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
            crate::program_log::log(&format!(
                "install: menu item {} removed: {gone:?}",
                at.display()
            ));
        }
        if let Some(at) = icon_file() {
            let gone = std::fs::remove_file(&at);
            crate::program_log::log(&format!("install: icon {} removed: {gone:?}", at.display()));
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
        crate::program_log::log(&format!(
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

/// A program started in its own folder, not waited for: the installed copy runs where it is
/// installed (its files, its log, and its folder's lock, `single_copy`, not the starting copy's).
pub fn run_program(command: &Path) -> bool {
    let mut run = std::process::Command::new(command);
    if let Some(folder) = command.parent().filter(|f| !f.as_os_str().is_empty()) {
        run.current_dir(folder);
    }
    let started = run.spawn();
    crate::program_log::log(&format!(
        "install: run {}: {}",
        command.display(),
        if started.is_ok() { "started" } else { "failed" }
    ));
    started.is_ok()
}
