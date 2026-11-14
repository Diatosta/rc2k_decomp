//! A stage load's timing, for builds with the `remote` feature run with `RC2K_LOAD_TIMES=1`: one
//! log line per load step, `load time <step>: <ms> ms (<category> <ms> [<count>], ..)`, then the
//! total and the time to the first race frame. The categories are exclusive (a file read inside
//! a picture decode counts as a file read only); what no category covers is the step's own work.
//! Without the feature, or without the variable, every call here does nothing.

/// What a measured scope waits on or works at.
#[derive(Clone, Copy)]
pub enum Category {
    /// Opening and reading a file (not the decoding of what it holds).
    FileRead,
    /// Decoding a PNG / PCX picture into pixels.
    PictureDecode,
    /// Converting a picture to a texture's levels and writing them.
    TextureConvert,
    /// Unlocking a texture's surface (the upload to the drawing backend).
    SurfaceUnlock,
    /// The loading bar's frame, its present excepted.
    BarFrame,
    /// Presenting the loading bar's frame.
    Present,
}

#[cfg(feature = "remote")]
mod active {
    use super::Category;
    use std::cell::RefCell;
    use std::time::{Duration, Instant};

    const NAMES: [&str; 6] = [
        "file read",
        "picture decode",
        "texture convert",
        "surface unlock",
        "bar frame",
        "present",
    ];

    /// The load being timed on this thread.
    struct State {
        begun: Instant,
        phase_name: String,
        phase_begun: Instant,
        totals: [Duration; 6],
        counts: [u32; 6],
        /// Each open scope's children's time.
        children: Vec<Duration>,
    }

    thread_local! {
        static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
        /// When the last finished load began, until its first race frame is logged.
        static FIRST: RefCell<Option<Instant>> = const { RefCell::new(None) };
    }

    fn enabled() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("RC2K_LOAD_TIMES").is_some())
    }

    pub fn begin(name: &str) {
        if !enabled() {
            return;
        }
        let now = Instant::now();
        STATE.with(|s| {
            *s.borrow_mut() = Some(State {
                begun: now,
                phase_name: name.to_string(),
                phase_begun: now,
                totals: [Duration::ZERO; 6],
                counts: [0; 6],
                children: Vec::new(),
            });
        });
    }

    fn close_phase(state: &mut State, now: Instant) {
        let mut line = format!(
            "load time {}: {:.1} ms",
            state.phase_name,
            now.duration_since(state.phase_begun).as_secs_f64() * 1000.0
        );
        let parts: Vec<String> = NAMES
            .iter()
            .enumerate()
            .filter(|(i, _)| state.counts[*i] != 0)
            .map(|(i, name)| {
                format!(
                    "{name} {:.1} [{}]",
                    state.totals[i].as_secs_f64() * 1000.0,
                    state.counts[i]
                )
            })
            .collect();
        if !parts.is_empty() {
            line += &format!(" ({})", parts.join(", "));
        }
        crate::framework::log(&line);
        state.totals = [Duration::ZERO; 6];
        state.counts = [0; 6];
    }

    pub fn phase(name: &str) {
        STATE.with(|s| {
            if let Some(state) = s.borrow_mut().as_mut() {
                let now = Instant::now();
                close_phase(state, now);
                state.phase_name = name.to_string();
                state.phase_begun = now;
            }
        });
    }

    pub fn end() {
        STATE.with(|s| {
            if let Some(mut state) = s.borrow_mut().take() {
                let now = Instant::now();
                close_phase(&mut state, now);
                crate::framework::log(&format!(
                    "load time total: {:.1} ms",
                    now.duration_since(state.begun).as_secs_f64() * 1000.0
                ));
                FIRST.with(|f| *f.borrow_mut() = Some(state.begun));
            }
        });
    }

    pub fn first_frame() {
        FIRST.with(|f| {
            if let Some(begun) = f.borrow_mut().take() {
                crate::framework::log(&format!(
                    "load time to the first race frame: {:.1} ms",
                    begun.elapsed().as_secs_f64() * 1000.0
                ));
            }
        });
    }

    /// A measured stretch, counted when dropped.
    pub struct Scope(Option<(Category, Instant)>);

    pub fn scope(category: Category) -> Scope {
        let open = STATE.with(|s| {
            s.borrow_mut()
                .as_mut()
                .map(|state| state.children.push(Duration::ZERO))
        });
        Scope(open.map(|()| (category, Instant::now())))
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            let Some((category, begun)) = self.0 else {
                return;
            };
            let elapsed = begun.elapsed();
            STATE.with(|s| {
                if let Some(state) = s.borrow_mut().as_mut() {
                    let own_children = state.children.pop().unwrap_or_default();
                    if let Some(parent) = state.children.last_mut() {
                        *parent += elapsed;
                    }
                    state.totals[category as usize] += elapsed.saturating_sub(own_children);
                    state.counts[category as usize] += 1;
                }
            });
        }
    }
}

#[cfg(not(feature = "remote"))]
mod active {
    use super::Category;
    pub fn begin(_: &str) {}
    pub fn phase(_: &str) {}
    pub fn end() {}
    pub fn first_frame() {}
    pub struct Scope;
    pub fn scope(_: Category) -> Scope {
        Scope
    }
}

pub use active::{begin, end, first_frame, phase, scope};
