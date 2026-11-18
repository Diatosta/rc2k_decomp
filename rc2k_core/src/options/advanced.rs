//! The advanced page's rows: the `ral.cfg` switches shown and toggled in the game, the number of
//! parts of a frame drawn at once and the renderer tried first at start. Each row says when a
//! change takes effect (`When`); `ral.cfg` is written back as the game reads it, the other two
//! go to `rc2k.toml` (`options::model::Display`).

/// When a changed row takes effect.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum When {
    /// At once, at Apply.
    Now,
    /// The next race that is loaded.
    NextRace,
    /// The next start of the program.
    NextStart,
}

/// What a row holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// How many parts of a frame may draw at once.
    Threads,
    /// The renderer: swapped in at once where the platform can, else tried first at the next
    /// start.
    Renderer,
    /// The race's quit question with the controller's pictures in its text (`rc2k.toml`); off,
    /// the question is the game's own line.
    QuitPictures,
    /// Frames between the simulation's ticks drawn blended (`rc2k.toml`); off, a frame shows the
    /// last tick and the frames are at most a hundred a second.
    SmoothFrames,
    /// The triggers' own rumble on a pad that has it (`rc2k.toml`).
    TriggerRumble,
    /// The computer's cars driving a wide road as a wide one (`rc2k.toml`); off, every road is
    /// driven as a narrow one, as the original does.
    WideRoadDrivers,
    /// A `ral.cfg` switch: the key; `inverted` when the key names what is off (`nofmv`), so the
    /// page shows the feature, not its absence.
    Flag { key: &'static str, inverted: bool },
}

/// One row.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub kind: Kind,
    pub when: When,
}

const fn flag(key: &'static str, inverted: bool, when: When) -> Row {
    Row {
        kind: Kind::Flag { key, inverted },
        when,
    }
}

/// The rows in the order the page shows them: the first part the drawing and the screen, the
/// second driving, sound and controls.
pub const ROWS: [Row; 15] = [
    Row {
        kind: Kind::Threads,
        when: When::Now,
    },
    // At Apply on the desktop; the handheld's context is made once, so there it is the next
    // start (the page says so beside the value).
    Row {
        kind: Kind::Renderer,
        when: When::Now,
    },
    flag("framecounter", false, When::Now),
    Row {
        kind: Kind::SmoothFrames,
        when: When::Now,
    },
    flag("nofmv", true, When::Now),
    flag("screensnapshot", false, When::Now),
    flag("noprecipitation", true, When::NextRace),
    flag("nonlinearsteering", false, When::Now),
    flag("noautoneutral", true, When::Now),
    flag("noforcefeedback", true, When::NextStart),
    Row {
        kind: Kind::TriggerRumble,
        when: When::Now,
    },
    flag("nomusic", true, When::NextStart),
    Row {
        kind: Kind::QuitPictures,
        when: When::Now,
    },
    // Read when a saved game is loaded, so a change counts from the next load.
    flag("usesavegamekeys", false, When::Now),
    Row {
        kind: Kind::WideRoadDrivers,
        when: When::Now,
    },
];

/// The page's row items: the menu has this many, each part fills them from the top and hides
/// the rest.
pub const ROW_SLOTS: usize = 12;

/// How many rows the first part of the page shows; the second shows the rest. Kept short so
/// the page does not run down the screen.
pub const PAGE_ROWS: usize = 7;

/// The rows of a part of the page (0 or 1) as a range of `ROWS`.
pub fn rows_on_page(page: usize) -> std::ops::Range<usize> {
    if page == 0 {
        0..PAGE_ROWS
    } else {
        PAGE_ROWS..ROWS.len()
    }
}

/// The renderers the page offers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Renderer {
    Gpu,
    /// OpenGL alone (the desktop's; on the handheld the GPU renderer is already GL ES).
    Gl,
    Software,
}

impl Renderer {
    /// The name `rc2k.toml` and `--renderer` use.
    pub fn name(self) -> &'static str {
        match self {
            Renderer::Gpu => "gpu",
            Renderer::Gl => "gl",
            Renderer::Software => "software",
        }
    }

    /// The renderer a stored name asks for (`gles` is the handheld's GPU renderer); none for a
    /// name this does not offer, which the page shows as the normal order, the GPU first.
    pub fn of(name: Option<&str>) -> Renderer {
        match name.map(|n| n.trim().to_ascii_lowercase()).as_deref() {
            Some("software") => Renderer::Software,
            Some("gl" | "opengl") => Renderer::Gl,
            _ => Renderer::Gpu,
        }
    }

    /// The next renderer offered, either way round; OpenGL only where `gl` says it is offered.
    pub fn other(self, up: bool, gl: bool) -> Renderer {
        let offered: &[Renderer] = if gl {
            &[Renderer::Gpu, Renderer::Gl, Renderer::Software]
        } else {
            &[Renderer::Gpu, Renderer::Software]
        };
        let at = offered.iter().position(|&r| r == self).unwrap_or(0);
        let n = offered.len();
        offered[if up { (at + 1) % n } else { (at + n - 1) % n }]
    }
}

/// The page's values: what Apply would make real.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Values {
    /// Parts of a frame drawn at once, 1 ..= the most the machine has.
    pub threads: u32,
    pub renderer: Renderer,
    /// The quit question's pictures on.
    pub quit_pictures: bool,
    /// Frames drawn between the ticks.
    pub smooth_frames: bool,
    /// The triggers' rumble on.
    pub trigger_rumble: bool,
    /// The computer's cars drive wide roads as wide.
    pub wide_road_drivers: bool,
    /// Each `Flag` row's switch as the page shows it (on = the feature on), in `ROWS` order
    /// (the other rows' entries are unused).
    pub flags: [bool; ROWS.len()],
}

impl Values {
    /// The row `row` stepped: a number up or down within `1 ..= most`, a choice or a switch
    /// flipped either way.
    pub fn step(&mut self, row: usize, up: bool, most: u32, gl: bool) {
        match ROWS[row].kind {
            Kind::Threads => {
                self.threads = if up {
                    (self.threads + 1).min(most)
                } else {
                    self.threads.saturating_sub(1).max(1)
                };
            }
            Kind::Renderer => self.renderer = self.renderer.other(up, gl),
            Kind::QuitPictures => self.quit_pictures = !self.quit_pictures,
            Kind::SmoothFrames => self.smooth_frames = !self.smooth_frames,
            Kind::TriggerRumble => self.trigger_rumble = !self.trigger_rumble,
            Kind::WideRoadDrivers => self.wide_road_drivers = !self.wide_road_drivers,
            Kind::Flag { .. } => self.flags[row] = !self.flags[row],
        }
    }
}

/// The `ral.cfg` number of a switch shown as `on`.
pub fn flag_number(inverted: bool, on: bool) -> u32 {
    u32::from(on != inverted)
}

/// Whether a switch with `ral.cfg` number `number` shows as on.
pub fn flag_shown(inverted: bool, number: u32) -> bool {
    (number != 0) != inverted
}

/// `text` (the lines of a `ral.cfg`) with key `key` set to `number`: the key's line, wherever it
/// is, becomes `key=number` (the first one; later lines of the key stay as they are, which the
/// game's reader does not look at), or the line is added at the end. The other lines are kept
/// byte for byte, the line ends too (CR LF as the game's files have them).
pub fn set_key(text: &[u8], key: &str, number: u32) -> Vec<u8> {
    let new = format!("{key}={number}");
    let mut out = Vec::with_capacity(text.len() + new.len() + 2);
    let mut done = false;
    for line in text.split_inclusive(|&b| b == b'\n') {
        let body_len = line
            .iter()
            .rposition(|&b| b != b'\n' && b != b'\r')
            .map_or(0, |at| at + 1);
        if !done && line_key(&line[..body_len]).is_some_and(|k| k.eq_ignore_ascii_case(key)) {
            out.extend_from_slice(new.as_bytes());
            out.extend_from_slice(&line[body_len..]);
            done = true;
        } else {
            out.extend_from_slice(line);
        }
    }
    if !done {
        if !out.is_empty() && !out.ends_with(b"\n") {
            out.extend_from_slice(b"\r\n");
        }
        out.extend_from_slice(new.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    out
}

/// The key a `ral.cfg` line sets: its first word, up to a space or `=`; none for an empty
/// line.
fn line_key(line: &[u8]) -> Option<&str> {
    let start = line.iter().position(|&b| b > b' ')?;
    let rest = &line[start..];
    let end = rest
        .iter()
        .position(|&b| b <= b' ' || b == b'=')
        .unwrap_or(rest.len());
    std::str::from_utf8(&rest[..end]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_show_the_feature_not_its_absence() {
        assert!(flag_shown(true, 0));
        assert!(!flag_shown(true, 1));
        assert!(flag_shown(false, 1));
        assert_eq!(flag_number(true, true), 0);
        assert_eq!(flag_number(true, false), 1);
        assert_eq!(flag_number(false, true), 1);
        assert_eq!(flag_number(false, false), 0);
    }

    #[test]
    fn every_row_has_its_place() {
        assert!(matches!(ROWS[0].kind, Kind::Threads));
        assert!(matches!(ROWS[1].kind, Kind::Renderer));
        assert!(matches!(ROWS[3].kind, Kind::SmoothFrames));
        assert!(matches!(ROWS[ROWS.len() - 3].kind, Kind::QuitPictures));
        // Every part fits the menu's row items.
        assert!(rows_on_page(0).len() <= ROW_SLOTS);
        assert!(rows_on_page(1).len() <= ROW_SLOTS);
        assert_eq!(rows_on_page(0).end, rows_on_page(1).start);
        assert_eq!(rows_on_page(1).end, ROWS.len());
        // Each flag key once.
        for (n, a) in ROWS.iter().enumerate() {
            for b in &ROWS[n + 1..] {
                if let (Kind::Flag { key: x, .. }, Kind::Flag { key: y, .. }) = (a.kind, b.kind) {
                    assert_ne!(x, y);
                }
            }
        }
    }

    #[test]
    fn the_saved_game_controls_switch_is_a_shown_row() {
        let at = ROWS
            .iter()
            .position(|r| {
                matches!(
                    r.kind,
                    Kind::Flag {
                        key: "usesavegamekeys",
                        ..
                    }
                )
            })
            .unwrap();
        // The row before the last, on the second part of the page.
        assert_eq!(at, ROWS.len() - 2);
        assert!(rows_on_page(1).contains(&at));
        // Not inverted: on means the saved game's own controls come back.
        assert!(matches!(
            ROWS[at].kind,
            Kind::Flag {
                inverted: false,
                ..
            }
        ));
        assert_eq!(ROWS[at].when, When::Now);
        assert_eq!(
            set_key(
                b"usesavegamekeys=0\r\n",
                "usesavegamekeys",
                flag_number(false, true)
            ),
            b"usesavegamekeys=1\r\n"
        );
    }

    #[test]
    fn steps_stay_in_range() {
        let mut v = Values {
            threads: 1,
            renderer: Renderer::Gpu,
            quit_pictures: true,
            smooth_frames: true,
            trigger_rumble: true,
            wide_road_drivers: false,
            flags: [false; ROWS.len()],
        };
        v.step(0, false, 4, false);
        assert_eq!(v.threads, 1);
        for _ in 0..6 {
            v.step(0, true, 4, false);
        }
        assert_eq!(v.threads, 4);
        v.step(1, true, 4, false);
        assert_eq!(v.renderer, Renderer::Software);
        v.step(2, false, 4, false);
        assert!(v.flags[2]);
        let quit = ROWS
            .iter()
            .position(|r| matches!(r.kind, Kind::QuitPictures))
            .unwrap();
        v.step(quit, true, 4, false);
        assert!(!v.quit_pictures);
        v.step(3, true, 4, false);
        assert!(!v.smooth_frames);
        let at = ROWS
            .iter()
            .position(|r| matches!(r.kind, Kind::TriggerRumble))
            .unwrap();
        v.step(at, true, 4, false);
        assert!(!v.trigger_rumble);
        let wide = ROWS
            .iter()
            .position(|r| matches!(r.kind, Kind::WideRoadDrivers))
            .unwrap();
        v.step(wide, true, 4, false);
        assert!(v.wide_road_drivers);
        assert!(rows_on_page(1).contains(&wide));
        // On the second part of the page, beside the other controls' rows.
        assert!(rows_on_page(1).contains(&at));
    }

    #[test]
    fn a_key_is_replaced_in_place_or_added() {
        let text = b"; ral.cfg\r\nlogfile=10\r\n  NoFMV = 0\r\nframecounter 1\r\n";
        assert_eq!(
            set_key(text, "nofmv", 1),
            b"; ral.cfg\r\nlogfile=10\r\nnofmv=1\r\nframecounter 1\r\n"
        );
        assert_eq!(
            set_key(text, "framecounter", 0),
            b"; ral.cfg\r\nlogfile=10\r\n  NoFMV = 0\r\nframecounter=0\r\n"
        );
        assert_eq!(
            set_key(text, "nojoy", 1),
            b"; ral.cfg\r\nlogfile=10\r\n  NoFMV = 0\r\nframecounter 1\r\nnojoy=1\r\n"
        );
        assert_eq!(set_key(b"", "nojoy", 1), b"nojoy=1\r\n");
        assert_eq!(set_key(b"a=1", "b", 2), b"a=1\r\nb=2\r\n");
        // A key that only begins like another is not it.
        assert_eq!(
            set_key(b"nofmvx=3\r\n", "nofmv", 1),
            b"nofmvx=3\r\nnofmv=1\r\n"
        );
    }

    #[test]
    fn the_written_file_reads_back() {
        let case = crate::strings::case::CaseTable::ascii();
        let text = set_key(b"logfile=10\r\n", "nojoy", 1);
        let mut with_nul = text;
        with_nul.push(0);
        let parsed = crate::files::settings::parse_settings(&with_nul, &case).unwrap();
        assert!(parsed.iter().any(|(s, n)| s.name() == b"nojoy" && *n == 1));
    }

    #[test]
    fn renderer_names() {
        assert_eq!(Renderer::of(None), Renderer::Gpu);
        assert_eq!(Renderer::of(Some("Software")), Renderer::Software);
        assert_eq!(Renderer::of(Some("gles")), Renderer::Gpu);
        assert_eq!(Renderer::of(Some("OpenGL")), Renderer::Gl);
        assert_eq!(Renderer::Gpu.other(true, true), Renderer::Gl);
        assert_eq!(Renderer::Gpu.other(false, true), Renderer::Software);
        assert_eq!(Renderer::Gpu.other(true, false), Renderer::Software);
        assert_eq!(Renderer::Software.name(), "software");
    }
}
