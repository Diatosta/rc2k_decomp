//! A fake file system for the self-tests of the installer's file-copy and -listing code
//! (`run_script`, `list_folder_tree`, `install_files`, `uninstall`, `install_clio`,
//! `check_installed_version`). The copy of an original and the Rust version both make their
//! `FindFirstFileA`/`FindNextFileA`/`FindClose`, file reads and writes, `CreateDirectoryA`,
//! `SetCurrentDirectoryA`, `RemoveDirectoryA`, `DeleteFileA` and `CopyFile` calls through the
//! same stand-ins, which answer from a scripted world here — never the real disk.
//!
//! `resolve_path` is the identity (so a pattern's key is the game-relative path the code
//! built). `find_first` keys the scan by the pattern string; its entries are whatever the
//! script gave for that key, with the search's skip mask (`FindFileData+0x13E`) applied.
//! The reads give scripted bytes; the directory- and file-making and the copying report
//! success unless the script says otherwise, and note themselves so a test can check the
//! sequence through `installer::take_noted`.

use crate::framework::Regs;
use crate::install::installer::{
    CLOSE, CLOSE_SEARCH, COPY_FILE, DELETE_FILE, FIND_FIRST, FIND_NEXT, MAKE_DIR, OPEN, READ,
    READ_FILE, REMOVE_DIR, RESOLVE_PATH, SET_CD, WRITE_FILE, a, g, rb, wb, wd,
};
use std::collections::HashMap;
use std::sync::Mutex;

const FIND_DATA: usize = 0x4E05A8; // FindFileData
const FIND_NAME: usize = 0x4E05A8 + 0x2C; // cFileName
const SKIP_MASK: usize = 0x4E06E6; // FindFileData+0x13E
const LAST_PATH: usize = 0x4F52B0; // g_lastFilePath

/// One directory entry: its name and `dwFileAttributes`.
#[derive(Clone)]
pub struct Entry {
    pub name: &'static str,
    pub attrs: u32,
}

pub const DIR: u32 = 0x10; // FILE_ATTRIBUTE_DIRECTORY

/// A scripted world: the entries each find pattern gives, the bytes each read gives, and
/// which operations fail.
#[derive(Default)]
pub struct World {
    pub listings: HashMap<String, Vec<Entry>>,
    pub reads: HashMap<String, Vec<u8>>,
    pub fail_make: Vec<String>,
    pub fail_cd: Vec<String>,
    pub fail_open: Vec<String>,
    pub fail_write: bool,
}

struct State {
    world: World,
    scan: Vec<Entry>,
    at: usize,
    /// How many bytes an open file (`OPEN`/`READ`/`CLOSE`) has given so far.
    read_at: usize,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// Sets the world for the next side's run (called from `prepare`, before each side).
pub fn set_world(world: World) {
    *STATE.lock().unwrap() = Some(State {
        world,
        scan: Vec::new(),
        at: 0,
        read_at: 0,
    });
}

/// The text at a game address, up to a NUL.
fn text_at(at: u32) -> String {
    let mut s = Vec::new();
    for n in 0..0x200u32 {
        let b = rb(at + n);
        if b == 0 {
            break;
        }
        s.push(b);
    }
    String::from_utf8_lossy(&s).into_owned()
}

fn write_find(entry: &Entry) {
    wd(a(FIND_DATA), entry.attrs);
    let name = a(FIND_NAME);
    for (k, b) in entry.name.bytes().enumerate() {
        wb(name + k as u32, b);
    }
    wb(name + entry.name.len() as u32, 0);
}

/// Advances to the next entry the skip mask lets through; writes it and returns true, or
/// returns false (carry: no more).
fn next_wanted(st: &mut State) -> bool {
    let skip = g(SKIP_MASK);
    while st.at < st.scan.len() {
        let e = st.scan[st.at].clone();
        st.at += 1;
        if e.attrs & skip == 0 {
            write_find(&e);
            return true;
        }
    }
    false
}

/// The stand-in handler: a call by place, the registers to answer in. Install it with
/// `installer::answer`. It also leaves each call noted (by `installer::note`).
pub fn answer(place: u32, r: &mut Regs) {
    let mut guard = STATE.lock().unwrap();
    let Some(st) = guard.as_mut() else {
        return;
    };
    // The IDA address of a call place (a game address): place - base + 0x400000.
    let ida = place
        .wrapping_sub(crate::framework::game_addr(0) as u32)
        .wrapping_add(0x40_0000) as usize;
    match ida {
        RESOLVE_PATH => {} // identity: eax kept
        FIND_FIRST => {
            wd(a(LAST_PATH), r.eax);
            let key = text_at(r.eax);
            st.scan = st.world.listings.get(&key).cloned().unwrap_or_default();
            st.at = 0;
            r.set_carry(!next_wanted(st));
        }
        FIND_NEXT => {
            r.set_carry(!next_wanted(st));
        }
        CLOSE_SEARCH => {}
        SET_CD => {
            let path = text_at(r.edx);
            let fail = st.world.fail_cd.iter().any(|p| path.contains(p.as_str()));
            r.eax = u32::from(!fail);
            r.set_carry(fail);
        }
        MAKE_DIR => {
            let path = text_at(r.edx);
            let fail = st.world.fail_make.iter().any(|p| path.contains(p.as_str()));
            r.eax = u32::from(!fail);
            r.set_carry(fail);
        }
        REMOVE_DIR | DELETE_FILE => {
            r.eax = 1;
            r.set_carry(false);
        }
        COPY_FILE => r.set_carry(false),
        OPEN => {
            let path = text_at(r.eax);
            wd(a(LAST_PATH), r.eax);
            st.read_at = 0;
            let fail = st.world.fail_open.iter().any(|p| path.contains(p.as_str()));
            r.eax = if fail { u32::MAX } else { 0x100 };
            r.set_carry(fail);
        }
        READ => {
            // the scripted bytes keyed by g_lastFilePath, given in order like a real file
            let key = text_at(a(LAST_PATH));
            let bytes = st.world.reads.get(&key).cloned().unwrap_or_default();
            let remaining = bytes.len().saturating_sub(st.read_at);
            let n = remaining.min(r.ecx as usize);
            for k in 0..n {
                wb(r.ebx + k as u32, bytes[st.read_at + k]);
            }
            st.read_at += n;
            r.eax = n as u32;
            r.set_carry(false);
        }
        CLOSE => {
            r.eax = 1;
            r.set_carry(false);
        }
        READ_FILE => {
            let path = text_at(r.eax);
            if st.world.fail_open.iter().any(|p| path.contains(p.as_str())) {
                r.set_carry(true);
                return;
            }
            let bytes = st.world.reads.get(&path).cloned().unwrap_or_default();
            let n = bytes.len().min(r.ecx as usize);
            for (k, &b) in bytes.iter().take(n).enumerate() {
                wb(r.ebx + k as u32, b);
            }
            r.eax = n as u32;
            r.set_carry(false);
            r.set_zero(true);
        }
        WRITE_FILE => {
            r.eax = r.ecx;
            r.set_carry(st.world.fail_write);
            r.set_zero(!st.world.fail_write);
        }
        _ => {}
    }
}

/// The addresses whose calls a copy of the original sends to stand-ins for this world.
pub const DISK_CALLS: [u32; 14] = [
    RESOLVE_PATH as u32,
    FIND_FIRST as u32,
    FIND_NEXT as u32,
    CLOSE_SEARCH as u32,
    SET_CD as u32,
    MAKE_DIR as u32,
    REMOVE_DIR as u32,
    DELETE_FILE as u32,
    COPY_FILE as u32,
    OPEN as u32,
    READ as u32,
    CLOSE as u32,
    READ_FILE as u32,
    WRITE_FILE as u32,
];
