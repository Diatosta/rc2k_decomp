//! With `RC2K_MIX_HASH` (a check, remote builds only): what the device mixer outputs during a
//! race, hashed per sound turn and logged as "mix hash N: ...".
//!
//! The device mixer normally runs in SDL's audio callback, paced by the device's clock, so the
//! samples it has made at a given command are not the same in two runs. While a race is loaded
//! and the check is on, the callback therefore plays silence and the audio thread mixes instead:
//! once after each audio turn that took a turn's end command, a fixed `TURN_FRAMES` frames. The
//! buffers' positions then advance only by those mixes, so the hash depends on the commands
//! alone.
//!
//! With `RC2K_MIX_WAV=<file>` as well (or alone) the mixed frames of each turn are also written
//! to a 16-bit stereo 44100 Hz WAV file, a new one for each race, to listen to what the mixer
//! makes on a machine without an audio device.

use crate::debug::check_hash::CheckHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock};

/// Frames mixed per sound turn: 20 ms of the device's 44100 Hz.
const TURN_FRAMES: usize = 882;

/// A race is loaded (set by the command queue, which knows it).
static RACING: AtomicBool = AtomicBool::new(false);

/// The audio thread took a turn's end command and has not mixed for it yet.
static TURN_TAKEN: AtomicBool = AtomicBool::new(false);

/// The turns of this race hashed so far.
static TURNS: Mutex<u32> = Mutex::new(0);

fn asked() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        std::env::var_os("RC2K_MIX_HASH").is_some() || std::env::var_os("RC2K_MIX_WAV").is_some()
    })
}

/// The WAV file of the race being mixed, with the frames written to it.
static WAV: Mutex<Option<(std::fs::File, u32)>> = Mutex::new(None);

/// Appends `pcm` (stereo frames) to the race's WAV file, its header's sizes brought up to date.
fn write_wav(pcm: &[i16]) {
    use std::io::{Seek, SeekFrom, Write};
    let Some(path) = std::env::var_os("RC2K_MIX_WAV") else {
        return;
    };
    let Ok(mut wav) = WAV.lock() else {
        return;
    };
    if wav.is_none() {
        let Ok(mut file) = std::fs::File::create(path) else {
            return;
        };
        let _ = file.write_all(&[0u8; 44]);
        *wav = Some((file, 0));
    }
    let Some((file, bytes)) = wav.as_mut() else {
        return;
    };
    let data: Vec<u8> = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
    *bytes += data.len() as u32;
    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(36 + *bytes).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    // PCM, two channels, 44100 Hz, four bytes a frame, sixteen bits.
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&2u16.to_le_bytes());
    header.extend_from_slice(&44100u32.to_le_bytes());
    header.extend_from_slice(&(44100u32 * 4).to_le_bytes());
    header.extend_from_slice(&4u16.to_le_bytes());
    header.extend_from_slice(&16u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&bytes.to_le_bytes());
    let _ = file.seek(SeekFrom::End(0));
    let _ = file.write_all(&data);
    let _ = file.seek(SeekFrom::Start(0));
    let _ = file.write_all(&header);
}

/// Whether the audio thread mixes for the check (and the device callback plays silence).
pub fn deterministic() -> bool {
    asked() && RACING.load(Relaxed)
}

/// Records whether a race is loaded; a race starts a fresh count.
pub fn set_racing(racing: bool) {
    if RACING.swap(racing, Relaxed) != racing {
        TURN_TAKEN.store(false, Relaxed);
        if let Ok(mut turns) = TURNS.lock() {
            *turns = 0;
        }
        if let Ok(mut wav) = WAV.lock() {
            *wav = None;
        }
    }
}

/// The audio thread has taken a turn's end command.
pub fn turn_end_taken() {
    TURN_TAKEN.store(true, Relaxed);
}

/// After the audio thread's turn: if it took a turn's end, mixes `TURN_FRAMES` frames and logs
/// their hash.
pub fn audio_turn_done() {
    if !deterministic() || !TURN_TAKEN.swap(false, Relaxed) {
        return;
    }
    let pcm = crate::platform::sound::mixed_frames(TURN_FRAMES);
    write_wav(&pcm);
    let mut hasher = CheckHasher::default();
    pcm.hash(&mut hasher);
    let Ok(mut turns) = TURNS.lock() else {
        return;
    };
    crate::logfile::program_log::log(&format!("mix hash {turns}: {:016x}", hasher.finish()));
    *turns += 1;
}
