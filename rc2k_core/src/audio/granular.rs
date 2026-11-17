//! A looping engine sample played as overlapping grains, so that the loop's length is not heard
//! as a phrase that comes round again.
//!
//! A plain loop of a sample about a second long, played at the pitch of high revs, repeats
//! every few tenths of a second and the ear learns the pattern. Here two read heads play the
//! sample at the voice's pitch; each runs for a grain (about a tenth of a second, under a
//! raised-cosine window), a new one starting every half grain, so that the windows add to 1
//! all the time. A new grain starts at a random place of the sample, moved to where its waveform
//! best matches what the grain before it is playing (a normalised correlation of a stretch ahead),
//! so that the join keeps the engine's tone instead of beating against it. The sample's
//! tone is the same as the plain loop's; only the order the stretches come in differs.

/// A grain's length in milliseconds of output time.
pub const GRAIN_MS: u32 = 100;
/// The source frames the match looks at, and how far either side of the random place it looks.
const MATCH_FRAMES: usize = 1024;
const MATCH_RANGE: usize = 600;
/// The match looks at every `MATCH_SKIP`th frame of the stretch (the engine's tone is low).
const MATCH_SKIP: usize = 4;
/// The places of the range looked at are every `PLACE_SKIP`th.
const PLACE_SKIP: usize = 2;

/// One read head: where it is in the sample and how far into its grain.
#[derive(Clone, Copy, Debug, Default)]
struct Grain {
    /// The sample frame (with a fraction).
    at: f64,
    /// The output frames since it began; at or past the grain's length it is silent.
    age: u32,
}

/// The two read heads of a looping sample and the random places they start at.
#[derive(Clone, Debug)]
pub struct Granular {
    grains: [Grain; 2],
    /// The grain that started last (the other is the older, fading out).
    newest: usize,
    /// The output frames since the newest began.
    since_start: u32,
    /// The grain's length and the hop between starts, in output frames.
    length: u32,
    hop: u32,
    random: u32,
    /// The window at each age of a grain.
    window: Vec<f32>,
}

impl Granular {
    /// A start at sample frame `at` of the loop, for output at `out_rate` Hz; `seed` makes the
    /// places the grains start at (the same seed, the same sound).
    pub fn new(at: f64, out_rate: u32, seed: u32) -> Self {
        let length = (out_rate / 1000 * GRAIN_MS).max(64) & !1;
        let hop = length / 2;
        Granular {
            // The first grain starts at the top of its window, with the second about to
            // start, so the sound begins at full level.
            grains: [Grain { at, age: hop }, Grain::default()],
            newest: 0,
            since_start: hop,
            length,
            hop,
            random: seed | 1,
            window: (0..length)
                .map(|age| {
                    let s = crate::math::portable::sinf(
                        std::f32::consts::PI * age as f32 / length as f32,
                    );
                    s * s
                })
                .collect(),
        }
    }

    fn next_random(&mut self) -> u32 {
        let mut x = self.random;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.random = x;
        x
    }

    /// The window at `age` of a grain: 0 at its two ends, 1 in the middle, and `window(a) +
    /// window(a + hop)` is 1.
    fn window(&self, age: u32) -> f32 {
        self.window.get(age as usize).copied().unwrap_or(0.0)
    }

    /// The next output frame: the grains' frames added under their windows. `frames` is the
    /// sample's length, `step` the source frames a frame of output moves on, `read(n)` the
    /// sample's frame `n` (left, right; `n` is below `frames`). Every grain moves on.
    pub fn next_frame(
        &mut self,
        frames: usize,
        step: f64,
        read: &dyn Fn(usize) -> (f32, f32),
    ) -> (f32, f32) {
        if self.since_start >= self.hop {
            self.start_grain(frames, step, read);
        }
        self.since_start += 1;
        let (mut l, mut r) = (0.0, 0.0);
        for n in 0..2 {
            let g = self.grains[n];
            if g.age >= self.length {
                continue;
            }
            let w = self.window(g.age);
            let at = g.at as usize % frames;
            let t = (g.at - g.at.floor()) as f32;
            let (l0, r0) = read(at);
            let (l1, r1) = read((at + 1) % frames);
            l += (l0 + (l1 - l0) * t) * w;
            r += (r0 + (r1 - r0) * t) * w;
            let g = &mut self.grains[n];
            g.age += 1;
            g.at += step;
            if g.at >= frames as f64 {
                g.at -= frames as f64 * (g.at / frames as f64).floor();
            }
        }
        (l, r)
    }

    /// Starts the next grain in the older one's place, at a random place moved to the best
    /// match with what the newest grain goes on to play.
    fn start_grain(&mut self, frames: usize, step: f64, read: &dyn Fn(usize) -> (f32, f32)) {
        let ahead = self.grains[self.newest].at;
        let target = self.next_random() as usize % frames;
        let at = best_match(frames, ahead, target, step, read);
        let older = 1 - self.newest;
        self.grains[older] = Grain { at, age: 0 };
        self.newest = older;
        self.since_start = 0;
    }
}

/// The frame near `target` (within `MATCH_RANGE`) after which the sample looks most like it does
/// after `ahead`: the largest of `2 * sum(a * b) / (sum(a * a) + sum(b * b))` over the stretch,
/// which is 1 for the same wave, 0 for waves that do not go together, and falls when the levels
/// differ. A stretch is read at the voice's pitch, as the grains play it.
fn best_match(
    frames: usize,
    ahead: f64,
    target: usize,
    step: f64,
    read: &dyn Fn(usize) -> (f32, f32),
) -> f64 {
    // A short sample has no room to look: the target as it is.
    if frames < 8 * MATCH_FRAMES {
        return target as f64;
    }
    let count = MATCH_FRAMES / MATCH_SKIP;
    // The offsets the stretch's frames are at from its start.
    let offsets: Vec<usize> = (0..count)
        .map(|n| (n as f64 * MATCH_SKIP as f64 * step) as usize)
        .collect();
    let span = offsets[count - 1] + 1;
    // The sample from `first` on, for the range and the stretch at its far end.
    let first = (target + frames - MATCH_RANGE % frames) % frames;
    let window: Vec<f32> = (0..2 * MATCH_RANGE + span)
        .map(|n| read((first + n) % frames).0)
        .collect();
    let base = ahead as usize % frames;
    let reference: Vec<f32> = offsets
        .iter()
        .map(|o| read((base + o) % frames).0)
        .collect();
    let energy_a: f32 = reference.iter().map(|v| v * v).sum();
    let mut best = (f32::MIN, 0usize);
    for d in (0..2 * MATCH_RANGE).step_by(PLACE_SKIP) {
        let (mut dot, mut energy_b) = (0f32, 0f32);
        for (a, o) in reference.iter().zip(&offsets) {
            let b = window[d + o];
            dot += a * b;
            energy_b += b * b;
        }
        let score = 2.0 * dot / (energy_a + energy_b + 1.0e-9);
        if score > best.0 {
            best = (score, d);
        }
    }
    ((first + best.1) % frames) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 44100;

    /// A tone with a slow beat, 2 s long.
    fn tone(frames: usize) -> Vec<f32> {
        (0..frames)
            .map(|n| {
                let t = n as f32 / RATE as f32;
                (t * 2.0 * std::f32::consts::PI * 180.0).sin() * (0.6 + 0.3 * (t * 7.0).sin())
            })
            .collect()
    }

    #[test]
    fn the_windows_of_two_heads_add_to_one() {
        let g = Granular::new(0.0, RATE, 1);
        for age in 0..g.hop {
            let sum = g.window(age) + g.window(age + g.hop);
            assert!((sum - 1.0).abs() < 1e-4, "{age}: {sum}");
        }
    }

    #[test]
    fn a_steady_tone_stays_a_steady_tone() {
        let data = tone(RATE as usize * 2);
        let read = |n: usize| (data[n], data[n]);
        let mut g = Granular::new(1000.0, RATE, 7);
        let out: Vec<f32> = (0..RATE as usize * 3)
            .map(|_| g.next_frame(data.len(), 1.3, &read).0)
            .collect();
        assert!(out.iter().all(|v| v.is_finite() && v.abs() < 1.3));
        // Its level is the tone's, over every tenth of a second.
        for chunk in out.chunks(4410).skip(1) {
            let rms = (chunk.iter().map(|v| v * v).sum::<f32>() / chunk.len() as f32).sqrt();
            assert!((0.25..0.75).contains(&rms), "{rms}");
        }
        // The same seed, the same sound.
        let mut again = Granular::new(1000.0, RATE, 7);
        let second: Vec<f32> = (0..2000)
            .map(|_| again.next_frame(data.len(), 1.3, &read).0)
            .collect();
        assert_eq!(&out[..2000], &second[..]);
    }

    /// Writes the demonstration sounds: `RC2K_GRANULAR_WAV=<dir> RC2K_GRANULAR_FILE=<in.wav>
    /// cargo test -p rc2k_core --release granular_demo -- --ignored`.
    #[test]
    #[ignore = "writes sounds for the ear"]
    fn granular_demo() {
        let (Ok(dir), Ok(file)) = (
            std::env::var("RC2K_GRANULAR_WAV"),
            std::env::var("RC2K_GRANULAR_FILE"),
        ) else {
            return;
        };
        let bytes = std::fs::read(&file).unwrap();
        let mut data = bytes[44..].to_vec();
        let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let total = (data.len() / 2) as u32;
        let frames = crate::audio::samples::blend_loop_join(&mut data, 0x120, total, rate) as usize;
        let samples: Vec<f32> = data
            .chunks_exact(2)
            .take(frames)
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
            .collect();
        let read = |n: usize| (samples[n], samples[n]);
        let name = std::path::Path::new(&file)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let seconds = 20;
        let step_at = |t: f64| 2.0 + 0.5 * (t * 0.6).sin();
        let mut plain_out = Vec::new();
        let mut grain_out = Vec::new();
        let mut at = 0.0f64;
        let mut g = Granular::new(0.0, 44100, 12345);
        let started = std::time::Instant::now();
        let mut grain_time = std::time::Duration::ZERO;
        for n in 0..44100 * seconds {
            let step = step_at(n as f64 / 44100.0) * rate as f64 / 44100.0;
            let i = at as usize;
            let t = (at - i as f64) as f32;
            let (a, b) = (read(i % frames).0, read((i + 1) % frames).0);
            plain_out.push(a + (b - a) * t);
            at = (at + step) % frames as f64;
            let t0 = std::time::Instant::now();
            grain_out.push(g.next_frame(frames, step, &read).0);
            grain_time += t0.elapsed();
        }
        let _ = started;
        eprintln!(
            "granular: {:.2} ms of CPU for {seconds} s of sound ({:.3} % of real time)",
            grain_time.as_secs_f64() * 1000.0,
            grain_time.as_secs_f64() * 100.0 / seconds as f64
        );
        let write = |label: &str, v: &[f32]| {
            let pcm: Vec<u8> = v
                .iter()
                .flat_map(|x| ((x * 20000.0).clamp(-32768.0, 32767.0) as i16).to_le_bytes())
                .collect();
            let mut out = Vec::new();
            out.extend_from_slice(b"RIFF");
            out.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
            out.extend_from_slice(b"WAVEfmt ");
            out.extend_from_slice(&16u32.to_le_bytes());
            out.extend_from_slice(&[1, 0, 1, 0]);
            out.extend_from_slice(&44100u32.to_le_bytes());
            out.extend_from_slice(&88200u32.to_le_bytes());
            out.extend_from_slice(&[2, 0, 16, 0]);
            out.extend_from_slice(b"data");
            out.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
            out.extend_from_slice(&pcm);
            std::fs::write(format!("{dir}/{name}_{label}.wav"), out).unwrap();
        };
        write("before", &plain_out);
        write("after", &grain_out);
    }
}
