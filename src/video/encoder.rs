//! How the video's picture is encoded: the software encoder (`libx264`) or one of the machine's
//! own, found in the `ffmpeg` on the path.
//!
//! With the encoder choice on automatic every hardware encoder `ffmpeg` lists that this system
//! can have is tried, once, on a few frames of the video's own size, H.264 first (the most
//! compatible: NVENC, AMF, VAAPI, Quick Sync) and then AV1 (the same four); the first that makes
//! a file is used. When none does, or the choice is "software", `libx264` encodes. A card's H.264
//! encoder stops at 4096 pixels on a side, so a wider picture gets the card's AV1 encoder when it
//! has one (an `.mp4` holds AV1; players older than about 2019 do not play it). The log says
//! which one was chosen and why the others were not.
//!
//! The card's H.265 encoders are left out: on the Radeon RX 9070 XT the one VAAPI offers wrote
//! pictures that do not decode (a coded block with more bins than the decoder takes) in about
//! half of the 5120 x 1440 renders at a quantiser of 17 and a quarter at 24, its H.264 encoder
//! none in 11 renders up to 3840 x 2160.
//!
//! The picture reaches the encoder as NV12 (converted on the card when the renderer can, else on
//! the processor with the same numbers, `rc2k_core::gfx::nv12`), so the colours do not depend on
//! the encoder.

use rc2k_core::race::render_dialog::Quality;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The family a hardware encoder belongs to: how it is set up and what its quality number is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Nvenc,
    Amf,
    Vaapi,
    Qsv,
    /// Apple's, on macOS.
    VideoToolbox,
}

/// A hardware encoder of `ffmpeg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hardware {
    /// The encoder's name in `ffmpeg`.
    codec: &'static str,
    kind: Kind,
    /// Whether it writes AV1 (else H.264).
    av1: bool,
}

/// The hardware encoders worth a try, in the order they are tried: H.264 first, as the most
/// compatible, then AV1.
const CANDIDATES: [Hardware; 9] = [
    Hardware::new("h264_nvenc", Kind::Nvenc, false),
    Hardware::new("h264_amf", Kind::Amf, false),
    Hardware::new("h264_vaapi", Kind::Vaapi, false),
    Hardware::new("h264_qsv", Kind::Qsv, false),
    Hardware::new("av1_nvenc", Kind::Nvenc, true),
    Hardware::new("av1_amf", Kind::Amf, true),
    Hardware::new("av1_vaapi", Kind::Vaapi, true),
    Hardware::new("av1_qsv", Kind::Qsv, true),
    // The only one macOS has: it comes last, and is the only candidate there.
    Hardware::new("h264_videotoolbox", Kind::VideoToolbox, false),
];

impl Hardware {
    const fn new(codec: &'static str, kind: Kind, av1: bool) -> Self {
        Self { codec, kind, av1 }
    }

    /// Whether this kind of encoder can exist on the system the program runs on (VAAPI is a
    /// Linux interface, VideoToolbox a macOS one).
    fn possible_here(self) -> bool {
        match self.kind {
            Kind::Vaapi => cfg!(target_os = "linux"),
            Kind::VideoToolbox => cfg!(target_os = "macos"),
            Kind::Nvenc | Kind::Amf | Kind::Qsv => cfg!(any(target_os = "linux", windows)),
        }
    }
}

/// Which encoder writes the picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoder {
    /// `libx264`.
    Software,
    /// The machine's own.
    Hardware(Hardware),
    /// `ffmpeg` has no H.264 encoder: its own choice for the file's extension, at its own
    /// settings.
    Default,
}

impl Encoder {
    /// What the log calls it.
    pub fn name(self) -> String {
        match self {
            Self::Software => "libx264, software".into(),
            Self::Hardware(h) => format!("{}, hardware", h.codec),
            Self::Default => "its default codec".into(),
        }
    }
}

/// Which of `CANDIDATES` the list `ffmpeg -encoders` printed holds, as a bit for each.
pub fn found_in(listed: &str) -> u16 {
    CANDIDATES
        .iter()
        .enumerate()
        .filter(|(_, h)| h.possible_here() && listed.split_whitespace().any(|w| w == h.codec))
        .fold(0, |bits, (i, _)| bits | 1 << i)
}

/// The software encoder's preset at a quality. The constant rate factor (`Quality::crf`) keeps
/// the picture's quality; the preset trades the file's size for the time: the faster presets
/// make a somewhat larger file of the same quality.
fn preset(quality: Quality) -> &'static str {
    match quality {
        Quality::Low | Quality::Medium => "veryfast",
        Quality::High => "faster",
        Quality::Best => "fast",
    }
}

/// The hardware encoders' quantiser at a quality (lower keeps more of the picture); chosen to
/// make files near the software encoder's for the same quality word.
fn quantiser(quality: Quality) -> u32 {
    match quality {
        Quality::Low => 32,
        Quality::Medium => 27,
        Quality::High => 22,
        Quality::Best => 17,
    }
}

/// VideoToolbox's quality number at a quality (1 to 100; higher keeps more of the picture).
fn quality_videotoolbox(quality: Quality) -> u32 {
    match quality {
        Quality::Low => 40,
        Quality::Medium => 55,
        Quality::High => 70,
        Quality::Best => 85,
    }
}

/// The AV1 encoders' quantiser at a quality: AV1's scale runs to 255 where H.264's runs to 51, and
/// the numbers make files smaller than the H.264 ones at about the same PSNR (measured on a
/// replay's frames with the card's encoders).
fn quantiser_av1(quality: Quality) -> u32 {
    match quality {
        Quality::Low => 165,
        Quality::Medium => 120,
        Quality::High => 65,
        Quality::Best => 30,
    }
}

/// The picture cut to an even size on both sides, as 4:2:0 wants.
const EVEN: &str = "crop=trunc(iw/2)*2:trunc(ih/2)*2";

/// The render device VAAPI uses: the first of the system's.
fn render_device() -> String {
    let mut found: Vec<_> = std::fs::read_dir("/dev/dri")
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("renderD"))
        })
        .collect();
    found.sort();
    found
        .first()
        .map_or_else(|| "/dev/dri/renderD128".into(), |p| p.display().to_string())
}

fn strings(words: &[&str]) -> impl Iterator<Item = String> {
    words
        .iter()
        .map(|w| (*w).to_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

/// The arguments that go before the input: the hardware device the encoder needs.
pub fn device_args(encoder: Encoder) -> Vec<String> {
    match encoder {
        Encoder::Hardware(Hardware {
            kind: Kind::Vaapi, ..
        }) => vec![
            "-init_hw_device".into(),
            format!("vaapi=va:{}", render_device()),
            "-filter_hw_device".into(),
            "va".into(),
        ],
        _ => Vec::new(),
    }
}

/// The arguments that go after the input: the filters and the encoder's settings.
pub fn encode_args(encoder: Encoder, quality: Quality) -> Vec<String> {
    let mut args = Vec::new();
    match encoder {
        Encoder::Default => {}
        Encoder::Software => {
            args.extend(strings(&["-vf", EVEN, "-c:v", "libx264", "-preset"]));
            args.push(preset(quality).into());
            args.extend(strings(&["-crf"]));
            args.push(quality.crf().to_string());
            args.extend(strings(&["-pix_fmt", "yuv420p"]));
        }
        Encoder::Hardware(h) => {
            let vaapi = h.kind == Kind::Vaapi;
            let q = if h.av1 {
                quantiser_av1(quality)
            } else {
                quantiser(quality)
            }
            .to_string();
            let filters = if vaapi {
                format!("{EVEN},format=nv12,hwupload")
            } else {
                EVEN.to_owned()
            };
            args.extend(["-vf".into(), filters, "-c:v".into(), h.codec.into()]);
            match h.kind {
                Kind::Vaapi if h.av1 => {
                    args.extend(strings(&["-rc_mode", "CQP", "-global_quality"]));
                    args.push(q);
                }
                Kind::Vaapi => args.extend(["-qp".into(), q]),
                Kind::Nvenc => {
                    args.extend(strings(&["-preset", "p5", "-tune", "hq", "-rc", "constqp"]));
                    args.extend(["-qp".into(), q]);
                    args.extend(strings(&["-pix_fmt", "yuv420p"]));
                }
                Kind::Amf => {
                    args.extend(strings(&["-usage", "transcoding", "-quality", "quality"]));
                    args.extend(strings(&["-rc", "cqp"]));
                    // AV1 has no B-frame quantiser here.
                    let flags: &[&str] = if h.av1 {
                        &["-qp_i", "-qp_p"]
                    } else {
                        &["-qp_i", "-qp_p", "-qp_b"]
                    };
                    for flag in flags {
                        args.extend([(*flag).into(), q.clone()]);
                    }
                    args.extend(strings(&["-pix_fmt", "nv12"]));
                }
                Kind::VideoToolbox => {
                    // The card's chips only: a constant quality (1 to 100, higher keeps more), no
                    // software fallback inside it, and no real-time shortcuts.
                    args.extend(strings(&["-allow_sw", "0", "-realtime", "0", "-q:v"]));
                    args.push(quality_videotoolbox(quality).to_string());
                    args.extend(strings(&["-pix_fmt", "nv12"]));
                }
                Kind::Qsv => {
                    args.extend(strings(&["-preset", "slow", "-global_quality"]));
                    args.push(q);
                    args.extend(strings(&["-pix_fmt", "nv12"]));
                }
            }
        }
    }
    args
}

/// How long a trial of an encoder may take before it is taken as not working (a driver that
/// hangs).
const TRIAL_TIME: Duration = Duration::from_secs(30);

/// Whether `hardware` makes a file of `width` x `height` pictures: `ffmpeg` encodes a few
/// frames of that size to nowhere. `Err`: the first line it said.
fn works(hardware: Hardware, width: u32, height: u32, quality: Quality) -> Result<(), String> {
    let encoder = Encoder::Hardware(hardware);
    let mut command = Command::new("ffmpeg");
    command.args(["-hide_banner", "-loglevel", "error", "-y"]);
    command.args(device_args(encoder));
    command.args(["-f", "lavfi", "-i"]);
    command.arg(format!("color=c=gray:s={width}x{height}:r=30,format=nv12"));
    command.args(["-frames:v", "3"]);
    command.args(encode_args(encoder, quality));
    command.args(["-f", "null", "-"]);
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    // The error text is read on a thread of its own so that a full pipe cannot hold ffmpeg.
    let mut stderr = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(pipe) = stderr.as_mut() {
            let _ = std::io::Read::read_to_string(pipe, &mut text);
        }
        text
    });
    let began = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if began.elapsed() < TRIAL_TIME => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Err(e) => return Err(e.to_string()),
        }
    };
    let said = reader.join().unwrap_or_default();
    // The first line says what went wrong; the ones after it say that nothing was written.
    let last = said.lines().next().unwrap_or("").trim().to_owned();
    match status {
        Some(status) if status.success() => Ok(()),
        Some(status) => Err(if last.is_empty() {
            status.to_string()
        } else {
            last
        }),
        None => Err("it did not finish in time".into()),
    }
}

/// The encoder the render uses: `software` when asked for (or `libx264` is missing and there is
/// no hardware either), else the first working hardware encoder among the `found` ones.
/// `log` gets one line for each encoder tried.
pub fn choose(
    automatic: bool,
    h264: bool,
    found: u16,
    (width, height): (u32, u32),
    quality: Quality,
    log: &dyn Fn(&str),
) -> Encoder {
    if automatic {
        // The size the encoder is asked to make: even on both sides.
        let even = (width & !1, height & !1);
        for (i, hardware) in CANDIDATES.iter().enumerate() {
            if found & (1 << i) == 0 {
                continue;
            }
            match works(*hardware, even.0, even.1, quality) {
                Ok(()) => return Encoder::Hardware(*hardware),
                Err(why) => log(&format!("{} does not work here: {why}", hardware.codec)),
            }
        }
    }
    if h264 {
        Encoder::Software
    } else {
        Encoder::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
        args.iter()
            .position(|a| a == name)
            .map(|at| args[at + 1].as_str())
    }

    #[test]
    fn software_presets_get_faster_at_lower_quality() {
        let preset_of = |q| flag(&encode_args(Encoder::Software, q), "-preset").map(str::to_owned);
        assert_eq!(preset_of(Quality::Best).as_deref(), Some("fast"));
        assert_eq!(preset_of(Quality::Low).as_deref(), Some("veryfast"));
        let args = encode_args(Encoder::Software, Quality::High);
        assert_eq!(flag(&args, "-crf"), Some("18"));
        assert_eq!(flag(&args, "-c:v"), Some("libx264"));
    }

    #[test]
    fn the_default_codec_gets_no_settings() {
        assert!(encode_args(Encoder::Default, Quality::Best).is_empty());
    }

    #[test]
    fn found_encoders_are_read_from_the_listing() {
        let listed = " V....D h264_vaapi H.264/AVC (VAAPI)\n V....D hevc_nvenc NVENC\n V....D av1_vaapi AV1\n V..... libx264 x\n";
        let bits = found_in(listed);
        let names: Vec<_> = CANDIDATES
            .iter()
            .enumerate()
            .filter(|(i, _)| bits & (1 << i) != 0)
            .map(|(_, h)| h.codec)
            .collect();
        // The H.265 encoders are not candidates; H.264 comes before AV1.
        if cfg!(target_os = "linux") {
            assert_eq!(names, ["h264_vaapi", "av1_vaapi"]);
        }
        assert_eq!(found_in("libx264"), 0);
    }

    #[test]
    fn the_hardware_encoder_gets_its_own_settings() {
        let vaapi = Encoder::Hardware(CANDIDATES[2]);
        let args = encode_args(vaapi, Quality::Best);
        assert_eq!(flag(&args, "-c:v"), Some("h264_vaapi"));
        assert_eq!(flag(&args, "-qp"), Some("17"));
        assert!(
            flag(&args, "-vf")
                .unwrap()
                .ends_with("format=nv12,hwupload")
        );
        assert!(device_args(vaapi).contains(&"-init_hw_device".to_owned()));
        let nvenc = encode_args(Encoder::Hardware(CANDIDATES[0]), Quality::Low);
        assert_eq!(flag(&nvenc, "-qp"), Some("32"));
        assert_eq!(flag(&nvenc, "-rc"), Some("constqp"));
        assert!(device_args(Encoder::Software).is_empty());
    }

    #[test]
    fn av1_encoders_get_their_own_quantiser_and_flags() {
        let vaapi = encode_args(Encoder::Hardware(CANDIDATES[6]), Quality::Best);
        assert_eq!(flag(&vaapi, "-c:v"), Some("av1_vaapi"));
        assert_eq!(flag(&vaapi, "-rc_mode"), Some("CQP"));
        assert_eq!(flag(&vaapi, "-global_quality"), Some("30"));
        let amf = encode_args(Encoder::Hardware(CANDIDATES[5]), Quality::Low);
        assert_eq!(flag(&amf, "-qp_i"), Some("165"));
        assert_eq!(flag(&amf, "-qp_b"), None);
        // H.264 keeps the order ahead of AV1 and the AV1 ones all write AV1.
        assert!(CANDIDATES[..4].iter().all(|h| !h.av1));
        assert!(CANDIDATES[4..8].iter().all(|h| h.av1));
    }

    #[test]
    fn videotoolbox_gets_a_constant_quality_and_no_software_fallback() {
        let toolbox = Encoder::Hardware(CANDIDATES[8]);
        let args = encode_args(toolbox, Quality::Best);
        assert_eq!(flag(&args, "-c:v"), Some("h264_videotoolbox"));
        assert_eq!(flag(&args, "-q:v"), Some("85"));
        assert_eq!(flag(&args, "-allow_sw"), Some("0"));
        assert_eq!(flag(&args, "-realtime"), Some("0"));
        assert_eq!(flag(&args, "-pix_fmt"), Some("nv12"));
        // The picture goes to it as it is: no upload filter and no device.
        assert_eq!(flag(&args, "-vf"), Some(EVEN));
        assert!(device_args(toolbox).is_empty());
        assert_eq!(
            flag(&encode_args(toolbox, Quality::Low), "-q:v"),
            Some("40")
        );
        let listed = " V....D h264_videotoolbox VideoToolbox H.264\n";
        assert_eq!(found_in(listed) != 0, cfg!(target_os = "macos"));
    }

    #[test]
    fn software_is_chosen_when_asked_for_or_when_nothing_is_found() {
        let log = |_: &str| {};
        let all = 0xFFFF;
        assert_eq!(
            choose(false, true, all, (1280, 720), Quality::High, &log),
            Encoder::Software
        );
        assert_eq!(
            choose(true, true, 0, (1280, 720), Quality::High, &log),
            Encoder::Software
        );
        assert_eq!(
            choose(false, false, all, (1280, 720), Quality::High, &log),
            Encoder::Default
        );
    }
}
