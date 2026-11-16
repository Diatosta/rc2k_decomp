//! `RC2K_2D_CHECK`: at the end of each frame the recorded commands are replayed into a canvas
//! and compared byte for byte with what the funnels wrote on the screen. A difference or a
//! command that cannot be replayed is logged, and the commands of each screen are counted.

use super::{Command, UNWRITTEN, replay::replay, take_frame};
use crate::menu::menus_store::MenuId;
use std::sync::Mutex;

/// What the check has seen so far.
#[derive(Default)]
struct Totals {
    frames: u64,
    commands: u64,
    differing: u64,
    problems: u64,
    logged: u32,
    /// The menu and counts of the last line written for a screen.
    last_screen: Option<(Option<MenuId>, Counts)>,
}

/// The commands of one frame by kind.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Counts {
    pub glyphs: u32,
    /// Glyph blits of the sprite table (no palette).
    pub sprites: u32,
    pub rects: u32,
    pub backgrounds: u32,
    pub images: u32,
    pub clears: u32,
}

impl Counts {
    pub fn of(commands: &[Command]) -> Self {
        let mut counts = Self::default();
        for command in commands {
            match command {
                Command::Glyph(g) if g.call.palette.is_some() => counts.glyphs += 1,
                Command::Glyph(_) => counts.sprites += 1,
                Command::Rect(_) => counts.rects += 1,
                Command::Background(_) => counts.backgrounds += 1,
                Command::Image(_) => counts.images += 1,
                Command::Clear(_) => counts.clears += 1,
            }
        }
        counts
    }
}

static TOTALS: Mutex<Option<Totals>> = Mutex::new(None);

/// The most lines the check logs about frames.
const LOG_LINES: u32 = 600;

/// Ends the frame: replays what was recorded and compares. `menu` labels the log.
pub fn end_frame(gfx: &crate::gfx::owner::Gfx, menu: Option<MenuId>) {
    let fonts = &gfx.fonts;
    if !super::enabled() {
        return;
    }
    let (frame, written, outside) = take_frame();
    let mut guard = TOTALS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let totals = guard.get_or_insert_with(Totals::default);
    totals.frames += 1;
    if frame.commands.is_empty() && written.is_empty() {
        return;
    }
    totals.commands += frame.commands.len() as u64;
    let counts = Counts::of(&frame.commands);
    let say = |totals: &mut Totals, line: String| {
        if totals.logged < LOG_LINES {
            totals.logged += 1;
            crate::logfile::program_log::log(&line);
        }
    };
    if totals.last_screen != Some((menu, counts)) {
        totals.last_screen = Some((menu, counts));
        say(
            totals,
            format!(
                "2d check: menu {menu:?} frame {}: {counts:?}",
                totals.frames
            ),
        );
    }
    let mut canvas = vec![UNWRITTEN; written.len()];
    let replayed = replay(&frame, fonts, &gfx.pictures.buffers, &mut canvas);
    if let Err((n, problem)) = &replayed {
        totals.problems += 1;
        say(
            totals,
            format!(
                "2d check: MISMATCH menu {menu:?}: command {n} of {}: {problem:?}",
                frame.commands.len()
            ),
        );
    }
    if outside {
        totals.problems += 1;
        say(
            totals,
            format!("2d check: MISMATCH menu {menu:?}: a write past the screen"),
        );
    }
    if replayed.is_ok() {
        let differing = canvas.iter().zip(&written).filter(|(a, b)| a != b).count();
        if differing > 0 {
            totals.differing += 1;
            let first = canvas
                .iter()
                .zip(&written)
                .position(|(a, b)| a != b)
                .unwrap_or(0);
            say(
                totals,
                format!(
                    "2d check: MISMATCH menu {menu:?}: {differing} bytes differ, first at byte {first}; {counts:?}"
                ),
            );
        }
    }
    if totals.frames.is_multiple_of(600) {
        let line = format!(
            "2d check: {} frames, {} commands, {} frames differ, {} problems",
            totals.frames, totals.commands, totals.differing, totals.problems
        );
        crate::logfile::program_log::log(&line);
    }
}
