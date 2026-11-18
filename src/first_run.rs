//! The first run: the program's folder holds no game data. A page asks for the folder or the
//! zip the data is in (picked on disk, dropped on the page, or named by `--game-data`); then
//! by what it holds the program goes on in it: an installed game's folder is the game's folder
//! as it is, the download / CD layout is the installer's source (`install::portable`), and a
//! zip is unpacked into a folder of its own beside the program's data first. Not the web
//! build, whose page does this in the browser (`tools/web/`).

use rc2k_core::files::game_data::{self, Kind};
use std::path::{Path, PathBuf};

/// Whether `rel` (with `/`) exists under `base`, each name found without regard to case.
fn exists(base: &Path, rel: &str) -> bool {
    let mut at = base.to_path_buf();
    for name in rel.split('/') {
        let exact = at.join(name);
        if exact.exists() {
            at = exact;
            continue;
        }
        let Ok(list) = std::fs::read_dir(&at) else {
            return false;
        };
        match list
            .flatten()
            .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        {
            Some(e) => at = e.path(),
            None => return false,
        }
    }
    true
}

fn kind_of(folder: &Path) -> Kind {
    let kind = game_data::classify(|rel| exists(folder, rel));
    // The phone has no installer: the game's files are played where they are, whatever
    // layout they came in.
    if cfg!(target_os = "android") && kind == Kind::Source {
        return Kind::Installed;
    }
    kind
}

/// Where a zip is unpacked: the user's data folder, in a folder of its own.
fn staging_folder() -> PathBuf {
    // On Android the game's own folder (the program runs in it): the zip is unpacked there.
    #[cfg(target_os = "android")]
    if let Ok(here) = std::env::current_dir() {
        return here;
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .or_else(|| std::env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(std::env::temp_dir);
    base.join("rally-championship-source")
}

/// The zip at `path` unpacked into `into`: every safe name, stored or deflated.
fn unpack(path: &Path, into: &Path) -> Result<(), String> {
    use std::io::{Read, Seek, SeekFrom};
    let name = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let mut file = std::fs::File::open(path).map_err(|e| name(&e))?;
    let length = file.metadata().map_err(|e| name(&e))?.len();
    let tail_from = length.saturating_sub(70_000);
    file.seek(SeekFrom::Start(tail_from))
        .map_err(|e| name(&e))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail).map_err(|e| name(&e))?;
    let (offset, count) = game_data::directory_of(&tail).map_err(|e| name(&e))?;
    file.seek(SeekFrom::Start(offset)).map_err(|e| name(&e))?;
    let mut directory = Vec::new();
    file.by_ref()
        .take(length - offset)
        .read_to_end(&mut directory)
        .map_err(|e| name(&e))?;
    let entries = game_data::entries_of(&directory, count).map_err(|e| name(&e))?;
    for entry in entries {
        let Some(parts) = game_data::safe_parts(&entry.name) else {
            continue;
        };
        let target = parts
            .iter()
            .fold(into.to_path_buf(), |p, part| p.join(part));
        if entry.name.ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|e| name(&e))?;
            continue;
        }
        if entry.size > 1 << 30 {
            return Err(name(&"a file of the zip is over 1 GB"));
        }
        let mut local = [0u8; 30];
        file.seek(SeekFrom::Start(entry.header))
            .map_err(|e| name(&e))?;
        file.read_exact(&mut local).map_err(|e| name(&e))?;
        let start = game_data::data_start(&entry, &local).map_err(|e| name(&e))?;
        file.seek(SeekFrom::Start(start)).map_err(|e| name(&e))?;
        let mut packed = vec![0u8; entry.packed as usize];
        file.read_exact(&mut packed).map_err(|e| name(&e))?;
        let data = match entry.method {
            0 => packed,
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(&packed, entry.size as usize)
                .map_err(|e| name(&format!("{}: {e:?}", entry.name)))?,
            m => {
                return Err(name(&format!(
                    "{}: compression {m} is not read",
                    entry.name
                )));
            }
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| name(&e))?;
        }
        std::fs::write(&target, data).map_err(|e| name(&e))?;
    }
    Ok(())
}

/// The folder the game goes on in for what was picked, with its kind; a zip is unpacked.
fn game_folder(picked: &Path) -> Result<(PathBuf, Kind), String> {
    let folder = if picked.is_file() {
        let into = staging_folder();
        unpack(picked, &into)?;
        // The copy of a file the Android picker named is not kept: it is as big as the game.
        #[cfg(target_os = "android")]
        if picked.file_name().is_some_and(|n| n == "picked.zip") {
            let _ = std::fs::remove_file(picked);
        }
        into
    } else {
        picked.to_path_buf()
    };
    // The game's folder is the picked one, or a folder just inside it (a zip of the folder).
    let mut candidates = vec![folder.clone()];
    if let Ok(list) = std::fs::read_dir(&folder) {
        let mut inside: Vec<PathBuf> = list
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        inside.sort();
        candidates.extend(inside);
    }
    candidates
        .into_iter()
        .find_map(|c| match kind_of(&c) {
            Kind::Nothing => None,
            kind => Some((c, kind)),
        })
        .ok_or_else(|| {
            format!(
                "{}: no game data here (files/d/gfx/jade.bun is not in it, nor in a folder just inside it)",
                picked.display()
            )
        })
}

/// What the picker gave as a path: Android's picker names a `content://` address, copied to a
/// file beside the game's folder first (and the folders it cannot name are refused).
fn local_copy(picked: &str) -> Result<String, String> {
    #[cfg(target_os = "android")]
    if picked.starts_with("content://") {
        let to = staging_folder().join("picked.zip");
        let size = crate::platform::ffi::sdl::copy_content(picked, &to)?;
        eprintln!(
            "rc2k: the picked file ({size} bytes) is copied to {}",
            to.display()
        );
        return Ok(to.to_string_lossy().into_owned());
    }
    Ok(picked.to_string())
}

/// A folder with no game data in the current folder is replaced by the one picked (the program
/// goes on there), or the program ends when none is picked.
pub fn find_game_data(given: Option<&str>) {
    // An installed macOS bundle holds its own files (`files::redirect`).
    #[cfg(unix)]
    if crate::files::redirect::active() {
        return;
    }
    if cfg!(target_os = "emscripten") || kind_of(Path::new(".")) != Kind::Nothing {
        return;
    }
    // The app's folder held a zip of the game's folder: the game is the folder just inside.
    #[cfg(target_os = "android")]
    if let Ok((folder, _)) = game_folder(Path::new("."))
        && std::env::set_current_dir(&folder).is_ok()
    {
        eprintln!("rc2k: the game's data is in {}", folder.display());
        return;
    }
    let mut wish = given.map(str::to_string);
    let mut message = String::new();
    loop {
        let picked = match wish.take() {
            Some(p) => p,
            None => match page(&message) {
                Some(p) => p,
                None => {
                    eprintln!("rc2k: no game data in this folder, and none was picked.");
                    std::process::exit(1);
                }
            },
        };
        let picked = match local_copy(&picked) {
            Ok(path) => path,
            Err(why) => {
                eprintln!("rc2k: {why}");
                if given.is_some() {
                    std::process::exit(1);
                }
                message = why;
                continue;
            }
        };
        match game_folder(Path::new(&picked)) {
            Ok((folder, kind)) => {
                let entered = std::env::set_current_dir(&folder);
                eprintln!(
                    "rc2k: the game's data is in {} ({kind:?}): {}",
                    folder.display(),
                    if entered.is_ok() {
                        "entered"
                    } else {
                        "cannot be entered"
                    }
                );
                if entered.is_ok() {
                    return;
                }
                message = format!("{} cannot be entered", folder.display());
            }
            Err(why) => {
                eprintln!("rc2k: {why}");
                if given.is_some() {
                    std::process::exit(1);
                }
                message = why;
            }
        }
    }
}

/// The page: where the game's data is. The path picked, dropped or None when it is closed.
fn page(message: &str) -> Option<String> {
    use crate::platform::ffi::sdl;
    use sdl3_sys::everything::*;
    if !sdl::init_video() {
        return None;
    }
    let window = sdl::create_window(c"Rally Championship", 960, 540)?;
    let renderer = sdl::create_renderer(window)?;
    // The page is 480 by 270 units, as large as the window allows (2 pixels a unit on the
    // 960 x 540 window of a desktop).
    let scale = window
        .pixel_size()
        .map_or(2.0, |(_, height)| (height as f32 / 270.0).max(1.0));
    renderer.set_scale(scale);
    // The buttons in the page's units.
    let folder = [40.0, 150.0, 190.0, 30.0];
    let zip = [250.0, 150.0, 190.0, 30.0];
    let inside = |(x, y): (f32, f32), r: [f32; 4]| {
        let (x, y) = (x / scale, y / scale);
        x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3]
    };
    let mut shown = message.to_string();
    let mut answer = None;
    'page: loop {
        while let Some(event) = sdl::poll_event() {
            if event.kind() == SDL_EVENT_QUIT
                || event
                    .key()
                    .is_some_and(|(code, down)| down && code == SDL_SCANCODE_ESCAPE.0)
            {
                break 'page;
            }
            if let Some(path) = event.dropped() {
                answer = Some(path);
                break 'page;
            }
            if let Some(at) = event.click() {
                if inside(at, folder) {
                    if cfg!(target_os = "android") {
                        shown = "Folders cannot be picked here: pick a zip.".to_string();
                    } else {
                        sdl::show_open_dialog(window, true);
                    }
                } else if inside(at, zip) {
                    sdl::show_open_dialog(window, false);
                }
            }
        }
        if let Some(done) = sdl::dialog_answer() {
            match done {
                Some(path) => {
                    answer = Some(path);
                    break;
                }
                None => shown = "Nothing was picked.".to_string(),
            }
        }
        renderer.clear([24, 28, 40]);
        let white = [240, 240, 240];
        renderer.text(white, 40.0, 30.0, "Rally Championship");
        renderer.text(white, 40.0, 60.0, "This folder has no game data.");
        renderer.text(
            white,
            40.0,
            76.0,
            "Pick the game's folder (installed, or the",
        );
        renderer.text(white, 40.0, 88.0, "download / CD files) or a zip of it, or");
        renderer.text(
            white,
            40.0,
            100.0,
            "drop one on this window. Nothing is downloaded.",
        );
        for (rect, label) in [(folder, "Pick a folder"), (zip, "Pick a zip file")] {
            renderer.fill([60, 80, 140], rect);
            renderer.text(white, rect[0] + 12.0, rect[1] + 11.0, label);
        }
        renderer.text(
            [255, 190, 120],
            40.0,
            200.0,
            &shown.chars().take(52).collect::<String>(),
        );
        renderer.text(white, 40.0, 250.0, "Esc: quit");
        renderer.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    sdl::destroy(window, renderer);
    answer
}
