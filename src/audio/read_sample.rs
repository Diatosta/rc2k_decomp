//! `audio::read_sample` (0x40B21B): a sample file read and parsed.
//!
//! In:  eax = the file's path
//! Out: carry = no sample found; every register preserved
//!
//! See `audio/samples.rs`. `g_sampleFile` gets one channel, 22050 Hz and 16 bits first;
//! the file is read whole into the work buffer (`files::read_file`, at most
//! `g_sampleBufferSize` bytes). RIFF / WAVE: the chunks are walked (no padding), `fmt `
//! must be PCM (tag 1) and gives channels, rate and bits (from a 12-byte chunk as words
//! at +4 and +0xA), `data` gives the data (at most what was read) and ends the walk; a
//! walk that runs out fails. Anything else is raw data, the whole read. Then: the length
//! rounded down to four, 4 .. 0x10000000; bits 8 or 16 and channels 1 or 2 by their tables
//! (`g_sampleBitsFlags`, `g_sampleChannelFlags`) make the format; the rate held to 100 ..
//! 200,000 (unsigned); bytes a frame (not 0) and frames. Callers: `audio::read_sample_of`
//! and the music's stream (0x40C24E, 0x40CFC2, with the buffer swapped for the stream's).
//! Refuses what `files::read_file` refuses.
//! Compared in shadow mode (the file read twice). Self-test: a redirected copy whose
//! buffer and sample file are the test's, on 20 files of the test's in `var`.

use crate::audio::samples::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::read_sample", rva(0x40B21B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    let (buffer, size): (u32, u32) = unsafe { (game_read(BUFFER_RVA), game_read(BUFFER_SIZE_RVA)) };
    let regions = [
        (buffer, size.min(0x100_0000) as usize),
        (
            game_addr(SAMPLE_FILE_RVA) as u32,
            std::mem::size_of::<SampleFile>(),
        ),
        (game_addr(rva(0x4F52B0)) as u32, 4),
    ];
    let rust = |r: &Regs| -> Result<Left, Refused> {
        Ok(Left::carry(!unsafe {
            read_sample_in_game(r.eax as *const u8)
        }?))
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

/// A RIFF file of the given chunks.
fn riff(form: &[u8; 4], chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut body = form.to_vec();
    for (id, data) in chunks {
        body.extend_from_slice(*id);
        body.extend_from_slice(&(data.len() as u32).to_le_bytes());
        body.extend_from_slice(data);
    }
    let mut file = b"RIFF".to_vec();
    file.extend_from_slice(&(body.len() as u32).to_le_bytes());
    file.extend(body);
    file
}

fn fmt(tag: u16, channels: u16, rate: u32, bits: u16) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend_from_slice(&tag.to_le_bytes());
    f.extend_from_slice(&channels.to_le_bytes());
    f.extend_from_slice(&rate.to_le_bytes());
    f.extend_from_slice(&(rate * channels as u32 * bits as u32 / 8).to_le_bytes());
    f.extend_from_slice(&(channels * bits / 8).to_le_bytes());
    f.extend_from_slice(&bits.to_le_bytes());
    f
}

pub fn test_files() -> Vec<(&'static str, Vec<u8>)> {
    let data: Vec<u8> = (0..400u32).map(|i| (i * 7) as u8).collect();
    let short_fmt = {
        let mut f = Vec::new();
        f.extend_from_slice(&1u16.to_le_bytes());
        f.extend_from_slice(&2u16.to_le_bytes());
        f.extend_from_slice(&11025u16.to_le_bytes());
        f.extend_from_slice(&[0, 0, 0, 0]);
        f.extend_from_slice(&8u16.to_le_bytes());
        f
    };
    vec![
        (
            "16 mono",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 1, 22050, 16)), (b"data", &data)],
            ),
        ),
        (
            "8 stereo",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 2, 11025, 8)), (b"data", &data)],
            ),
        ),
        (
            "16 stereo, a chunk between",
            riff(
                b"WAVE",
                &[
                    (b"fmt ", &fmt(1, 2, 44100, 16)),
                    (b"LIST", b"abcdefgh"),
                    (b"data", &data),
                ],
            ),
        ),
        (
            "12-byte fmt",
            riff(b"WAVE", &[(b"fmt ", &short_fmt), (b"data", &data)]),
        ),
        (
            "data first",
            riff(b"WAVE", &[(b"data", &data), (b"fmt ", &fmt(1, 1, 8000, 8))]),
        ),
        (
            "not PCM",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(2, 1, 22050, 16)), (b"data", &data)],
            ),
        ),
        (
            "24 bits",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 1, 22050, 24)), (b"data", &data)],
            ),
        ),
        (
            "3 channels",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 3, 22050, 16)), (b"data", &data)],
            ),
        ),
        (
            "rate 50",
            riff(b"WAVE", &[(b"fmt ", &fmt(1, 1, 50, 16)), (b"data", &data)]),
        ),
        (
            "rate 300000",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 1, 300_000, 8)), (b"data", &data)],
            ),
        ),
        (
            "no data",
            riff(b"WAVE", &[(b"fmt ", &fmt(1, 1, 22050, 16))]),
        ),
        (
            "data of 3",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 1, 22050, 16)), (b"data", &data[..3])],
            ),
        ),
        (
            "data of 6",
            riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 1, 22050, 16)), (b"data", &data[..6])],
            ),
        ),
        ("data longer than the file", {
            let mut f = riff(
                b"WAVE",
                &[(b"fmt ", &fmt(1, 1, 22050, 16)), (b"data", &data[..40])],
            );
            let at = f.len() - 44;
            f[at..at + 4].copy_from_slice(&1000u32.to_le_bytes());
            f
        }),
        ("not WAVE", riff(b"AVI ", &[(b"data", &data)])),
        ("raw", data.clone()),
        ("raw of 7", data[..7].to_vec()),
        ("RIFF only", b"RIFF\x04\x00\x00\x00WAVE".to_vec()),
        ("a chunk running out", {
            let mut f = riff(b"WAVE", &[(b"fmt ", &fmt(1, 1, 22050, 16))]);
            f.extend_from_slice(b"junk\xFF\x00\x00\x00");
            f
        }),
        ("empty", Vec::new()),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The test's buffer (0x800 bytes), its two variables and its sample file.
    let mut buffer = vec![0u8; 0x800];
    let mut vars = Box::new([0u32; 2 + 8]);
    let at = vars.as_mut_ptr() as u32;
    vars[0] = buffer.as_mut_ptr() as u32;
    vars[1] = 0x800;
    let mut pairs = vec![(0x506930, at), (0x506934, at + 4)];
    for n in 0..8 {
        pairs.push((0x506938 + n * 4, at + 8 + n as u32 * 4));
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B21B,
            len: 0x17D,
            entry: &[
                0x60, 0xC7, 0x05, 0x44, 0x69, 0x50, 0x00, 0x01, 0x00, 0x00, 0x00,
            ],
            calls: &[0x2D, 0x104, 0x11D],
            jumps_out: &[],
        },
        &|_| None,
        &pairs,
        23,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let path = "var\\rc2k_hook_selftest.wav";
    let full = path;
    let mut cpath = path.as_bytes().to_vec();
    cpath.push(0);
    let mut files = test_files();
    files.push(("missing", Vec::new()));
    for (name, bytes) in &files {
        if *name == "missing" {
            let _ = std::fs::remove_file(full);
        } else if std::fs::write(full, bytes).is_err() {
            return t.skip("the test's file cannot be written");
        }
        let prepare = |b: &mut Vec<u8>, v: &mut [u32; 10]| {
            b.fill(0xAA);
            v[2..].fill(0x5555_5555);
        };
        prepare(&mut buffer, &mut vars);
        let before = Regs {
            eax: cpath.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (buffer.clone(), *vars);
        prepare(&mut buffer, &mut vars);
        let mine = unsafe {
            read_sample(
                cpath.as_ptr(),
                vars[0],
                vars[1],
                (at + 8) as *mut SampleFile,
            )
        };
        let ok = mine.as_ref().is_ok_and(|&found| found == !ro.carry())
            && theirs == (buffer.clone(), *vars)
            && ro.preserved_except(&before, 0);
        t.check(ok, || {
            format!(
                "file \"{name}\": original carry {} {:x?}; rust {mine:?} {:x?}",
                ro.carry(),
                &theirs.1[2..],
                &vars[2..]
            )
        });
    }
    let _ = std::fs::remove_file(full);
    t.finish()
}
