//! The program's Start menu items and the program's copy (platform code): shell shortcuts in
//! the user's Start menu Programs folder, group "Rally Championship": "Rally", "Uninstall Rally"
//! (`rc2k.exe -u`; only that copy can remove the installation) and "View Readme"; the removal
//! deletes the three and the group's folder. On Unix the items are desktop entries in
//! `~/.local/share/applications`; on macOS small application bundles in
//! `~/Applications/Rally Championship/`. And the program installed: `rc2k.exe` and its DLLs
//! copied from the running program's folder into the current one.

use std::path::{Path, PathBuf};

/// The group's folder name.
pub const GROUP: &str = "Rally Championship";
/// The program's file.
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
            crate::logfile::program_log::log(
                "install: no Start menu Programs folder; no items added",
            );
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&group) {
            crate::logfile::program_log::log(&format!("install: {}: {e}", group.display()));
            return;
        }
        let items = items(installed);
        with_com(|| {
            for (name, target, arguments) in items {
                let at = group.join(format!("{name}.lnk"));
                let done = write_link(&at, &target, arguments, name);
                crate::logfile::program_log::log(&format!(
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
        crate::logfile::program_log::log(&format!(
            "install: Start menu group {} removed: {gone:?}",
            group.display()
        ));
    }
}

#[cfg(target_os = "macos")]
mod start_menu {
    use super::{GROUP, items};
    use std::path::{Path, PathBuf};

    fn group_folder() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Applications").join(GROUP))
    }

    /// A path quoted for the shell (`'..'`, a `'` inside closed, escaped and reopened).
    fn quoted(path: &Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', r"'\''"))
    }

    /// An item's bundle: `<name>.app` with its property list and its script.
    fn write_bundle(at: &Path, name: &str, script: &str) -> std::io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let programs = at.join("Contents/MacOS");
        std::fs::create_dir_all(&programs)?;
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n<dict>\n\
             <key>CFBundleName</key><string>{name}</string>\n\
             <key>CFBundleExecutable</key><string>run</string>\n\
             <key>CFBundleIdentifier</key><string>org.rc2k.item.{id}</string>\n\
             <key>CFBundlePackageType</key><string>APPL</string>\n\
             </dict>\n</plist>\n",
            id = name.to_lowercase().replace(' ', "-"),
        );
        std::fs::write(at.join("Contents/Info.plist"), plist)?;
        let run = programs.join("run");
        std::fs::write(&run, script)?;
        std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o755))
    }

    pub fn add(installed: &Path) {
        let Some(group) = group_folder() else {
            crate::logfile::program_log::log("install: no home folder; no applications added");
            return;
        };
        for (name, target, arguments) in items(installed) {
            let at = group.join(format!("{name}.app"));
            let start = if target
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
            {
                format!("exec open {}", quoted(&target))
            } else {
                format!("exec {} {arguments}", quoted(&target))
                    .trim_end()
                    .to_string()
            };
            let script = format!("#!/bin/sh\ncd {} || exit 1\n{start}\n", quoted(installed));
            let done = write_bundle(&at, name, &script);
            crate::logfile::program_log::log(&format!(
                "install: application {} -> {start}: {}",
                at.display(),
                match &done {
                    Ok(()) => "written".to_string(),
                    Err(e) => e.to_string(),
                }
            ));
        }
    }

    pub fn remove() {
        let Some(group) = group_folder() else {
            return;
        };
        let gone = std::fs::remove_dir_all(&group);
        crate::logfile::program_log::log(&format!(
            "install: applications group {} removed: {gone:?}",
            group.display()
        ));
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod start_menu {
    use super::{GROUP, items};
    use std::path::{Path, PathBuf};

    fn applications() -> Option<PathBuf> {
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
        Some(data.join("applications"))
    }

    fn entry_file(folder: &Path, name: &str) -> PathBuf {
        let id = format!("{GROUP} {name}").to_lowercase().replace(' ', "-");
        folder.join(format!("{id}.desktop"))
    }

    pub fn add(installed: &Path) {
        let Some(folder) = applications() else {
            crate::logfile::program_log::log("install: no applications folder; no items added");
            return;
        };
        if let Err(e) = std::fs::create_dir_all(&folder) {
            crate::logfile::program_log::log(&format!("install: {}: {e}", folder.display()));
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
            crate::logfile::program_log::log(&format!(
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
            crate::logfile::program_log::log(&format!(
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
        crate::logfile::program_log::log(&format!(
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

/// A program started, not waited for.
pub fn run_program(command: &Path) -> bool {
    let started = std::process::Command::new(command).spawn();
    crate::logfile::program_log::log(&format!(
        "install: run {}: {}",
        command.display(),
        if started.is_ok() { "started" } else { "failed" }
    ));
    started.is_ok()
}
