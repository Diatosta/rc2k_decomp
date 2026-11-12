//! The system's own facts on Windows: what has no std or SDL form, each a plain
//! call with a typed answer; the calls themselves are `ffi::win32`'s. A Linux build answers
//! the same questions in its own file.

pub use crate::ffi::win32::{
    computer_name, discard_pages, disk_free, drive_type, keyboard_subtype, language_id, map_pages,
    memory, set_error_mode, set_priority_class, terminate, user_name, version_record, volume,
};

/// The folder `dir` removed by a hidden `cmd.exe` once this program lets go of its files in
/// it (Windows removes no folder a running program's files are in): tried each second for
/// five minutes, ending as soon as it is gone. Whether the command started (not for a path
/// `cmd` would read otherwise: one with `%` or `"`).
pub fn remove_folder_after_exit(dir: &std::path::Path) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let (Some(text), Some(parent)) = (dir.to_str(), dir.parent()) else {
        return false;
    };
    let text = text.trim_end_matches('\\');
    if text.contains(['%', '"']) {
        return false;
    }
    // `ping` as the wait: `timeout` needs a console (and Wine's does not wait).
    let line = format!(
        "/C for /L %i in (1,1,300) do @if exist \"{text}\" (ping -n 2 127.0.0.1 >nul & rmdir /s /q \"{text}\")"
    );
    std::process::Command::new("cmd.exe")
        .raw_arg(line)
        .current_dir(parent)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .is_ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_folder_is_removed_after_the_end() {
        let dir = std::env::temp_dir().join("rc2k removal test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub").join("a.txt"), b"a").unwrap();
        assert!(super::remove_folder_after_exit(&dir));
        let gone = (0..100).any(|_| {
            std::thread::sleep(std::time::Duration::from_millis(200));
            !dir.exists()
        });
        assert!(gone, "{} is still there", dir.display());
    }
}
