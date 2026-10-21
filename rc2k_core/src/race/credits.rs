//! The credits rolling up over the screen (converted 2026-10-21 from `race::roll_credits`
//! 0x415FC2 and `race::credit_line` 0x4161BA): a script of words laid out from the top
//! down, the top moving up a pixel each 0x12 ms, the end once the last line is off the
//! top and the music has stopped; and the race the credits show, its stage and car drawn
//! (converted 2026-10-21 from `race::set_up_credits_race` 0x463711).

/// The credits' font of a script word (1, 2, 3, 4, 7, 8; the columns use 4's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Font(pub u16);

/// A line of the credits: a string in a font at (x, y).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub font: Font,
    pub id: u32,
    pub x: i32,
    pub y: i32,
}

/// Where the credits start: below the screen's bottom.
pub fn start_top(screen_height: u32) -> u32 {
    screen_height.wrapping_add(0x15E)
}

/// The script's lines from `top` down (`word` the script's next word; `line_height` a
/// font's): 1, 2, 7 a string centred, 8 one at a sixteenth of the width, 4 one right of
/// the middle, each moving down a line; 3 one left of the middle (not moving); 5 three
/// strings in columns (moving down once); 6 a signed move (its next word); 9 nothing; any
/// other the end. Gives the lines in order and the place below the last.
pub fn lay_out(
    mut word: impl FnMut() -> u16,
    width: i32,
    top: i32,
    line_height: impl Fn(Font) -> i32,
) -> (Vec<Line>, i32) {
    let mut lines = Vec::new();
    let mut y = top;
    loop {
        let op = word();
        let placed = match op {
            1 | 2 | 7 => Some((width >> 1, true)),
            8 => Some((width >> 4, true)),
            3 => Some(((width >> 1) - 0xF, false)),
            4 => Some(((width >> 1) + 0xF, true)),
            _ => None,
        };
        if let Some((x, moves)) = placed {
            let font = Font(op);
            let id = word() as u32;
            lines.push(Line { font, id, x, y });
            if moves {
                y = y.wrapping_add(line_height(font));
            }
            continue;
        }
        match op {
            5 => {
                let edge = width >> 4;
                let column = ((width as u32)
                    .wrapping_sub(edge as u32)
                    .wrapping_sub(edge as u32)
                    / 3) as i32;
                let font = Font(4);
                for k in 0..3 {
                    let id = word() as u32;
                    lines.push(Line {
                        font,
                        id,
                        x: column.wrapping_mul(k).wrapping_add(edge),
                        y,
                    });
                    if k == 2 {
                        y = y.wrapping_add(line_height(font));
                    }
                }
            }
            6 => y = y.wrapping_add(word() as i16 as i32),
            9 => {}
            _ => return (lines, y),
        }
    }
}

/// `race::credit_line`'s test: a line is drawn unless it is at or above -0x1B (off the
/// top) or at or past the screen's bottom.
pub fn shown(y: i32, screen_height: u32) -> bool {
    y > -0x1B && y < screen_height as i32
}

/// The clock caught up: while 0x12 ms or more have passed since `clock` (`now` asked each
/// time), the top one pixel up and the clock 0x12 on. Gives the top, the clock and the
/// time passed last seen.
pub fn catch_up(mut top: u32, mut clock: u32, mut now: impl FnMut() -> u32) -> (u32, u32, u32) {
    loop {
        let elapsed = now().wrapping_sub(clock);
        if (elapsed.wrapping_sub(0x12) as i32) < 0 {
            return (top, clock, elapsed);
        }
        top = top.wrapping_sub(1);
        clock = clock.wrapping_add(0x12);
    }
}

/// Whether the credits are over: the place below the last line at or above -0xAF and the
/// music (`playing`, 1 while it plays) stopped.
pub fn rolled_off(below_last: i32, playing: u32) -> bool {
    below_last <= -0xAF && playing != 1
}

/// The stages the credits' race is drawn from (`dword_71C3E4`).
pub const RACE_STAGES: u32 = 36;
/// The car kinds it is drawn from (`dword_71C474`).
pub const RACE_KINDS: u32 = 23;

/// A draw (`math::next_random`) as a place among `n`: the high dword of draw x n, held to
/// 0 .. n - 1 by the original's sign tests.
pub fn drawn_place(draw: u32, n: u32) -> u32 {
    let p = ((draw as u64 * n as u64) >> 32) as u32;
    let p = if (p as i32) < 0 { 0 } else { p };
    if (p.wrapping_sub(n - 1) as i32) >= 0 {
        n - 1
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        let script = [
            1u16, 10, 3, 11, 4, 12, 6, 0xFFF6, 5, 20, 21, 22, 9, 8, 13, 0,
        ];
        let mut at = 0;
        let (lines, y) = lay_out(
            || {
                at += 1;
                script[at - 1]
            },
            800,
            100,
            |f| f.0 as i32 * 10,
        );
        assert_eq!(at, script.len());
        assert_eq!(
            lines
                .iter()
                .map(|l| (l.font.0, l.id, l.x, l.y))
                .collect::<Vec<_>>(),
            [
                (1, 10, 400, 100),
                (3, 11, 385, 110),
                (4, 12, 415, 110),
                (4, 20, 50, 140),
                (4, 21, 50 + 233, 140),
                (4, 22, 50 + 466, 140),
                (8, 13, 50, 180),
            ]
        );
        assert_eq!(y, 260);
    }

    #[test]
    fn draws() {
        assert_eq!(drawn_place(0, RACE_STAGES), 0);
        assert_eq!(drawn_place(u32::MAX, RACE_STAGES), 35);
        assert_eq!(drawn_place(0x8000_0000, RACE_KINDS), 11);
    }

    #[test]
    fn clock_and_end() {
        assert_eq!(start_top(600), 600 + 0x15E);
        assert!(shown(-0x1A, 600) && !shown(-0x1B, 600) && !shown(600, 600));
        let mut t = [100u32, 100, 100].into_iter();
        assert_eq!(catch_up(50, 60, || t.next().unwrap()), (48, 96, 4));
        assert_eq!(catch_up(50, 60, || 70), (50, 60, 10));
        assert!(rolled_off(-0xAF, 0) && !rolled_off(-0xAE, 0) && !rolled_off(-0x100, 1));
    }
}
