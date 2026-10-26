//! What the hooks of the three PCX readers share: their `dispatch` and their self-test case.
//! A reader takes the path in eax and returns with the carry flag set if it failed; it
//! reads a file and writes the picture buffer and the variables of `gfx/pcx.rs`, so in
//! shadow mode it can run twice: the Rust version first, then the original, and what each
//! left is compared.

use crate::errors::BadInput;
use crate::files::stream::LastPath;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_write, shadow};
use crate::gfx::pcx::{
    HEIGHT_LIMIT_RVA, WIDTH_LIMIT_RVA, decoded_region, reader_inputs, reader_regions,
};
use crate::gfx::picture::{HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA, picture};
use crate::screens::harness::CallOriginal;
use crate::service::harness::{Region, capture, restore};

/// A reader's Rust version: the path in, whether it worked out.
pub type Reader = unsafe fn(*const u8) -> Result<bool, BadInput>;

fn first_difference(a: &[u8], b: &[u8]) -> Option<usize> {
    if a.len() != b.len() {
        return Some(a.len().min(b.len()));
    }
    a.iter().zip(b).position(|(x, y)| x != y)
}

/// What a reader left: its variables, and the decoded rows as the layout now describes them.
fn left_behind() -> (Vec<u8>, Vec<u8>) {
    (capture(&reader_regions()), capture(&decoded_region()))
}

pub fn dispatch_reader(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, rust: Reader) {
    hook.count_call();
    let path = r.eax as *const u8;
    if !shadow() {
        match unsafe { rust(path) } {
            Ok(worked) => {
                r.set_carry(!worked);
                r.set_zero(worked);
            }
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }
    let mine = unsafe { rust(path) };
    let state = left_behind();
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = left_behind();
    // What the Rust version refuses, the original is left to deal with.
    if let Ok(worked) = mine
        && (worked == o.carry() || theirs != state || !o.preserved_except(r, 0))
    {
        hook.mismatch(|| {
            format!(
                "path={:?}: original worked={} rust worked={worked}; first different byte of the variables {:?}, of the {} decoded bytes {:?}",
                unsafe { std::ffi::CStr::from_ptr(path.cast()) },
                !o.carry(),
                first_difference(&theirs.0, &state.0),
                theirs.1.len(),
                first_difference(&theirs.1, &state.1)
            )
        });
    }
    r.eflags = o.eflags;
}

/// `dispatch_reader` without its hook (a refusal is the caller's).
pub fn reader_entry(r: &mut Regs, rust: Reader) -> Result<(), crate::errors::Refused> {
    let worked = unsafe { rust(r.eax as *const u8) }?;
    r.set_carry(!worked);
    r.set_zero(worked);
    Ok(())
}

/// The game's variables a reader's self-test changes, to be put back afterwards.
pub struct ReaderTest {
    regions: Vec<Region>,
    saved: Vec<u8>,
    last_path: LastPath,
}

/// The first bytes of the picture buffer the tests decode into.
const TEST_PICTURE_BYTES: usize = 0x4000;

impl ReaderTest {
    /// None if there is no picture buffer.
    pub fn begin() -> Option<Self> {
        if picture().is_null() {
            return None;
        }
        let regions = [
            &reader_regions()[..],
            &reader_inputs(),
            &[(picture() as u32, TEST_PICTURE_BYTES)],
        ]
        .concat();
        Some(Self {
            saved: capture(&regions),
            regions,
            last_path: LastPath::save(),
        })
    }

    /// As the picture loader sets things before it reads a file: a buffer of 800 x 600, rows
    /// one after the other; and everything a reader writes filled with 0xAA.
    fn prepare() {
        let written = reader_regions();
        restore(&written, &vec![0xAA; written.iter().map(|r| r.1).sum()]);
        unsafe {
            std::ptr::write_bytes(picture(), 0xAA, TEST_PICTURE_BYTES);
            game_write(WIDTH_RVA, 800u32);
            game_write(HEIGHT_RVA, 600u32);
            game_write(ROW_BYTES_RVA, 0xFFFF_FFFDu32);
            game_write(WIDTH_LIMIT_RVA, 0x800u32);
            game_write(HEIGHT_LIMIT_RVA, 0x800u32);
        }
    }

    /// One case: the code at `original` and the Rust version on the same path. The test
    /// pictures must decode into less than `TEST_PICTURE_BYTES`.
    pub fn check(&self, t: &mut SelfTest, what: &str, original: usize, path: &[u8], rust: Reader) {
        let observed = || {
            (
                capture(&reader_regions()),
                unsafe { std::slice::from_raw_parts(picture(), TEST_PICTURE_BYTES) }.to_vec(),
                LastPath::now(),
            )
        };
        let before = Regs {
            eax: path.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        Self::prepare();
        let mut ro = before;
        unsafe { call_game(&mut ro, original) };
        let theirs = observed();
        Self::prepare();
        let mine = unsafe { rust(path.as_ptr()) };
        let state = observed();
        let preserved = ro.preserved_except(&before, 0);
        t.check(mine == Ok(!ro.carry()) && theirs == state && preserved, || {
            format!(
                "{what}: original worked={} preserved={preserved}; rust {mine:?}; first different byte of the variables {:?}, of the picture {:?}; last path {:#x} / {:#x}",
                !ro.carry(),
                first_difference(&theirs.0, &state.0),
                first_difference(&theirs.1, &state.1),
                theirs.2,
                state.2
            )
        });
    }

    pub fn end(self) {
        restore(&self.regions, &self.saved);
        self.last_path.restore();
    }
}
