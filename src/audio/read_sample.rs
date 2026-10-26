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
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

/// A RIFF file of the given chunks.
pub(crate) fn riff(form: &[u8; 4], chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
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

pub(crate) fn fmt(tag: u16, channels: u16, rate: u32, bits: u16) -> Vec<u8> {
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
