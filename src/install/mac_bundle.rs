//! The macOS installation as a self-contained application: the installer copies its own bundle
//! (`Rally Championship.app`, the one that holds the running program) into `/Applications`, or
//! `~/Applications` when the user cannot write there; the game's `files` tree goes inside the
//! copy (`Contents/Resources/files`), and what the game writes (`var`, `savedata`, logs) goes to
//! the user's Application Support folder. Once the copy is done the bundle is signed again (its
//! contents changed), freed of the quarantine mark, registered with Launch Services (so
//! Launchpad lists it) and opened. The bundle stays unchanged while the game runs.
//!
//! A program that does not run from a bundle (a bare binary) installs as the other systems do.
//! The paths and the command lines are tested on every system; the commands themselves run on
//! macOS only.

// The commands run on macOS only; the other Unix systems build the module for its tests.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::path::{Path, PathBuf};

/// The installed bundle's name.
const APP_NAME: &str = "Rally Championship.app";

/// The bundle that holds the program `exe` (`<name>.app/Contents/MacOS/<program>`).
pub fn bundle_of(exe: &Path) -> Option<PathBuf> {
    let programs = exe.parent().filter(|p| p.ends_with("Contents/MacOS"))?;
    programs
        .parent()
        .and_then(Path::parent)
        .filter(|b| b.extension().is_some_and(|e| e.eq_ignore_ascii_case("app")))
        .map(Path::to_path_buf)
}

/// Where the game's read-only files are inside a bundle.
pub fn files_in(bundle: &Path) -> PathBuf {
    bundle.join("Contents/Resources/files")
}

/// The user's Application Support folder for the game (the installer's first folder).
pub fn support_folder() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    Some(PathBuf::from(home).join("Library/Application Support/Rally Championship"))
}

/// The places the installed bundle may go, the first that can be written first.
fn destinations() -> Vec<PathBuf> {
    let mut places = vec![PathBuf::from("/Applications").join(APP_NAME)];
    if let Some(home) = std::env::var_os("HOME").filter(|h| !h.is_empty()) {
        places.push(PathBuf::from(home).join("Applications").join(APP_NAME));
    }
    places
}

/// The folder `from` copied to `to` with its links and permissions.
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::fs::read_link(&source)?, &target)?;
        } else if kind.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            std::fs::copy(&source, &target)?;
        }
    }
    Ok(())
}

/// The bundle installed by `begin`: where it is.
static INSTALLED: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Starts the installation of the bundle: the running program's bundle copied to the first
/// place that takes it; from then on the game's `files` folder in `support` (where the
/// installer is to put it) is the copy's. Ok(false): the program is not in a bundle, so the
/// install goes on as on the other systems. The error is what failed.
pub fn begin(support: &Path) -> Result<bool, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let Some(own) = bundle_of(&exe) else {
        return Ok(false);
    };
    let mut why = String::new();
    for place in destinations() {
        if let Some(folder) = place.parent()
            && std::fs::create_dir_all(folder).is_err()
        {
            continue;
        }
        let _ = std::fs::remove_dir_all(&place);
        match copy_tree(&own, &place) {
            Ok(()) => {
                crate::program_log::log(&format!(
                    "install: the bundle copied to {}",
                    place.display()
                ));
                let files = files_in(&place);
                std::fs::create_dir_all(&files).map_err(|e| e.to_string())?;
                crate::files::redirect::add(
                    &support.join("files").to_string_lossy(),
                    &files.to_string_lossy(),
                );
                let _ = INSTALLED.set(place);
                return Ok(true);
            }
            Err(e) => {
                why = format!("{}: {e}", place.display());
                crate::program_log::log(&format!("install: the bundle not copied: {why}"));
                let _ = std::fs::remove_dir_all(&place);
            }
        }
    }
    Err(why)
}

/// The commands that make the installed bundle `app` ready, in order: the quarantine mark
/// removed from the copy, the copy signed again (ad hoc: no developer account or tools needed),
/// registered with Launch Services, opened as a new instance.
fn finishing_commands(app: &Path) -> Vec<Vec<std::ffi::OsString>> {
    let a = app.as_os_str().to_os_string();
    let lsregister = "/System/Library/Frameworks/CoreServices.framework/Frameworks/\
                      LaunchServices.framework/Support/lsregister";
    let v = |parts: &[&str]| -> Vec<std::ffi::OsString> {
        let mut c: Vec<std::ffi::OsString> = parts.iter().map(Into::into).collect();
        c.push(a.clone());
        c
    };
    vec![
        v(&["/usr/bin/xattr", "-dr", "com.apple.quarantine"]),
        v(&["/usr/bin/codesign", "--force", "--deep", "--sign", "-"]),
        v(&[lsregister, "-f"]),
        v(&["/usr/bin/open", "-n"]),
    ]
}

/// Finishes the installation `begin` made: runs `finishing_commands`. Whether the installed
/// application was opened (false too when there is none).
pub fn finish() -> bool {
    let Some(app) = INSTALLED.get() else {
        return false;
    };
    let mut opened = false;
    for command in finishing_commands(app) {
        let done = std::process::Command::new(&command[0])
            .args(&command[1..])
            .status();
        crate::program_log::log(&format!(
            "install: {:?}: {}",
            command,
            match &done {
                Ok(s) => s.to_string(),
                Err(e) => e.to_string(),
            }
        ));
        opened = done.is_ok_and(|s| s.success());
    }
    opened
}

/// Whether an installation was begun (the bundle is installed).
pub fn installing() -> bool {
    INSTALLED.get().is_some()
}

/// The installed bundle removed (the uninstaller's): the running program's own bundle when it
/// is an installed one, else those the installer may have made.
pub fn remove() {
    let own = std::env::current_exe()
        .ok()
        .and_then(|e| bundle_of(&e))
        .filter(|b| destinations().iter().any(|d| d == b));
    let places = own.map_or_else(destinations, |b| vec![b]);
    for place in places.iter().filter(|p| files_in(p).exists()) {
        let gone = std::fs::remove_dir_all(place);
        crate::program_log::log(&format!(
            "install: the application {} removed: {gone:?}",
            place.display()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundle_of_a_program() {
        let exe = Path::new("/Users/me/Downloads/Rally Championship.app/Contents/MacOS/rc2k");
        assert_eq!(
            bundle_of(exe).as_deref(),
            Some(Path::new("/Users/me/Downloads/Rally Championship.app"))
        );
        assert_eq!(bundle_of(Path::new("/opt/rally/rc2k")), None);
        assert_eq!(bundle_of(Path::new("/x/Contents/MacOS/rc2k")), None);
        assert_eq!(
            files_in(Path::new("/Applications/Rally Championship.app")),
            Path::new("/Applications/Rally Championship.app/Contents/Resources/files")
        );
    }

    #[test]
    fn the_finishing_commands_end_with_the_application() {
        let app = Path::new("/Applications/Rally Championship.app");
        let commands = finishing_commands(app);
        assert_eq!(commands.len(), 4);
        for c in &commands {
            assert_eq!(c.last().unwrap(), app.as_os_str());
        }
        assert_eq!(commands[1][0], "/usr/bin/codesign");
        assert_eq!(commands[3][1], "-n");
    }

    #[test]
    fn a_bundle_is_copied_whole() {
        let base = std::env::temp_dir().join(format!("rc2k_bundle_{}", std::process::id()));
        let from = base.join("A.app");
        std::fs::create_dir_all(from.join("Contents/MacOS")).unwrap();
        std::fs::write(from.join("Contents/MacOS/rc2k"), b"x").unwrap();
        std::os::unix::fs::symlink("MacOS", from.join("Contents/link")).unwrap();
        let to = base.join("B.app");
        copy_tree(&from, &to).unwrap();
        assert_eq!(std::fs::read(to.join("Contents/MacOS/rc2k")).unwrap(), b"x");
        assert_eq!(
            std::fs::read_link(to.join("Contents/link")).unwrap(),
            Path::new("MacOS")
        );
        std::fs::remove_dir_all(base).unwrap();
    }
}
