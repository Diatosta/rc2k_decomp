//! `install::read_sku` (0x411696): reads the edition file `files\d\gfx\sku`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the key `sku`. Its value may start with the marks `oem:` and
//! `dvd:`, in any order and number, each setting a switch to 1 (`dword_5190F8`,
//! `dword_5190FC`); then a number (`dword_5190F4`), after the next `:` a second
//! (`byte_519101`) and after the next `.` within 254 characters a third (`byte_519100`).
//! What is not there keeps its value. One caller, `sub_411239`.
//! Self-test: a redirected copy that reads a file of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::install::copy_reading;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use crate::settings::find_key::find_key;
use crate::settings::parse_number::parse_number;
use crate::settings::read_text::read_text;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::read_sku", rva(0x411696), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const PATH_RVA: usize = rva(0x519102); // "files\d\gfx\sku"
const KEY_RVA: usize = rva(0x519112); // "sku"
const NUMBER_RVA: usize = rva(0x5190F4); // dword_5190F4
const OEM_RVA: usize = rva(0x5190F8); // dword_5190F8
const DVD_RVA: usize = rva(0x5190FC); // dword_5190FC
const THIRD_RVA: usize = rva(0x519100); // byte_519100
const SECOND_RVA: usize = rva(0x519101); // byte_519101

// ---- Rust version ----

pub unsafe fn read_sku(path: *const u8) -> Result<(), BadInput> {
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        let Some(mut at) = find_key(text, game_addr(KEY_RVA) as *const u8)? else {
            return Ok(());
        };
        loop {
            match &(at as *const [u8; 4]).read() {
                b"oem:" => game_write(OEM_RVA, 1u32),
                b"dvd:" => game_write(DVD_RVA, 1u32),
                _ => break,
            }
            at = at.add(4);
        }
        game_write(NUMBER_RVA, parse_number(at));
        // On to after the next `:`, however far.
        loop {
            let c = at.read();
            at = at.add(1);
            match c {
                0 => return Ok(()),
                b':' => break,
                _ => {}
            }
        }
        game_write(SECOND_RVA, parse_number(at) as u8);
        // On to after the next `.`: the original's count gives up at the 255th character.
        for seen in 1.. {
            let c = at.read();
            at = at.add(1);
            if c == 0 || seen == 255 {
                return Ok(());
            }
            if c == b'.' {
                break;
            }
        }
        game_write(THIRD_RVA, parse_number(at) as u8);
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 4] {
    [
        at(NUMBER_RVA),
        at(OEM_RVA),
        at(DVD_RVA),
        (game_addr(THIRD_RVA) as u32, 2),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_sku(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const LEN: usize = 0x95;
const CALLS: [usize; 5] = [0x07, 0x18, 0x50, 0x6E, 0x89];
const TEST_FILE: &str = "var\\rc2k_hook_sku";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_sku\0";

fn files() -> Vec<Vec<u8>> {
    let mut files: Vec<Vec<u8>> = [
        &b"sku=1234:5.6\r\n"[..],
        b"sku=oem:77:8.9\r\n",
        b"sku=dvd:oem:dvd:3:$1f.0x20\r\n",
        b"sku oem:\r\n",
        b"sku=42\r\n",
        b"sku=42:7\r\n",
        b"sku=OEM:5:6.7\r\n",
        b"sku=9:300.-1\r\nother=1.5\r\n",
        b"sku=1:2\r\nnext line.3\r\n",
        b"other=1\r\n",
        b"sku=\r\n",
        b"",
    ]
    .iter()
    .map(|file| file.to_vec())
    .collect();
    // The `.` as the 254th and as the 255th character after the `:`.
    for dots_after in [253usize, 254] {
        let mut file = b"sku=1:2".to_vec();
        file.extend(std::iter::repeat_n(b'x', dots_after - 1));
        file.extend(b".9\r\n");
        files.push(file);
    }
    files
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe { copy_reading(rva(0x411696), LEN, PATH_RVA, &CALLS, TEST_PATH) } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    let last = crate::files::stream::LastPath::save();
    let mut made = true;
    let files = files();
    for file in files.iter().map(Some).chain([None]) {
        let _ = std::fs::remove_file(TEST_FILE);
        if let Some(file) = file
            && std::fs::write(TEST_FILE, file).is_err()
        {
            made = false;
            break;
        }
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            let done = if rust {
                read_sku(TEST_PATH.as_ptr()).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (done, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!(
                "file={:?}: original done={} {:x?}; rust done={} {:x?}",
                file.map(|file| String::from_utf8_lossy(&file[..file.len().min(40)])),
                theirs.0,
                theirs.1,
                mine.0,
                mine.1
            )
        });
    }
    restore(&regions, &saved);
    last.restore();
    let _ = std::fs::remove_file(TEST_FILE);
    if !made {
        return t.skip("cannot create a test file in the game's var folder");
    }
    t.finish()
}
