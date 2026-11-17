//! Which renderer the game asks for. `--renderer software` or `RC2K_RENDERER=software` forces the
//! software rasteriser (the switch outranks the environment variable);
//! `gpu` (or nothing) means the default order: the GPU first, each renderer that cannot start
//! falling back to the next. Desktop: the SDL GPU device, then OpenGL (`gl`, also asked for
//! alone), then SDL's own 2D renderer with the software rasteriser. Handheld: OpenGL ES, then the rasteriser into the framebuffer. What
//! actually started is asked of `render::gpu_running`, not of this module.
//!
//! The choice is read afresh at each call, so a later start of the renderers can ask again,
//! with a wanted renderer passed to `decide` in place of `None`.

/// What is asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wish {
    /// The GPU if it starts, else the software rasteriser.
    GpuFirst,
    /// OpenGL, if it starts, else the software rasteriser (never the GPU API).
    Gl,
    /// The software rasteriser only.
    Software,
}

/// The wish for a renderer name; `None` for one this platform does not know.
pub fn parse(value: &str) -> Option<Wish> {
    match value.trim().to_ascii_lowercase().as_str() {
        "gpu" => Some(Wish::GpuFirst),
        "gles" if cfg!(all(target_os = "linux", target_arch = "aarch64")) => Some(Wish::GpuFirst),
        "gl" | "opengl" if cfg!(not(all(target_os = "linux", target_arch = "aarch64"))) => {
            Some(Wish::Gl)
        }
        "software" => Some(Wish::Software),
        _ => None,
    }
}

/// The names `--renderer` takes on this platform, for the message that refuses another.
pub fn names() -> &'static str {
    if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "gpu, gles or software"
    } else {
        "gpu, gl or software"
    }
}

/// The renderer wish from the command line's `--renderer` value, the environment's value,
/// `wanted` (a renderer asked for some other way) and the default order, in that order of
/// weight. An unknown value is passed to `unknown` and skipped.
pub fn decide(
    switch: Option<&str>,
    env: Option<&str>,
    wanted: Option<&str>,
    mut unknown: impl FnMut(&str, &str),
) -> Wish {
    for (source, value) in [
        ("--renderer", switch),
        ("RC2K_RENDERER", env),
        ("the setting", wanted),
    ] {
        let Some(value) = value.filter(|v| !v.is_empty()) else {
            continue;
        };
        match parse(value) {
            Some(wish) => return wish,
            None => unknown(source, value),
        }
    }
    Wish::GpuFirst
}

/// The `--renderer` value among the program's words, if any (a missing value is refused at
/// the start, `standalone::answer_command_line`).
fn switch_value() -> Option<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    rc2k_core::system::program_switches::Switches::from_args(&args)
        .ok()?
        .renderer
}

/// The renderer the options file names (`[display] renderer`), the weakest of the sources. Set
/// when the options are loaded and by the advanced page; never rewritten by a fallback.
static WANTED: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// The renderer the options name, if any.
pub fn wanted() -> Option<String> {
    WANTED.lock().ok().and_then(|w| w.clone())
}

/// Names the renderer to try first at the next start (`None`: the normal order).
pub fn set_wanted(name: Option<String>) {
    if let Ok(mut w) = WANTED.lock() {
        *w = name;
    }
}

/// What the command line, the environment and the options ask for.
pub fn wish() -> Wish {
    let switch = switch_value();
    let env = std::env::var("RC2K_RENDERER").ok();
    let wanted = wanted();
    decide(
        switch.as_deref(),
        env.as_deref(),
        wanted.as_deref(),
        |source, value| {
            crate::logfile::program_log::log(&format!(
                "render: {source}: renderer `{value}` is not known here ({}): the default order",
                names()
            ));
        },
    )
}

/// Whether the GPU API's renderer is to be tried.
pub fn gpu_wanted() -> bool {
    wish() == Wish::GpuFirst
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
/// Whether the OpenGL renderer is to be tried (after the GPU API's, in the default order).
pub fn gl_wanted() -> bool {
    matches!(wish(), Wish::GpuFirst | Wish::Gl)
}

/// With the `remote` feature, `RC2K_FAIL_RENDERER=<step>[,<step>..]` makes the named start-up
/// steps fail, to test the fallbacks: `device`, `gpu_renderer`, `backend`, `size` (desktop),
/// `gl` (OpenGL, desktop), `gles` (handheld). Always false without the feature.
pub fn forced_failure(step: &str) -> bool {
    #[cfg(feature = "remote")]
    if std::env::var("RC2K_FAIL_RENDERER").is_ok_and(|steps| steps.split(',').any(|s| s == step)) {
        crate::logfile::program_log::log(&format!(
            "render: step `{step}` is made to fail (RC2K_FAIL_RENDERER)"
        ));
        return true;
    }
    let _ = step;
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_of_the_renderers() {
        assert_eq!(parse("gpu"), Some(Wish::GpuFirst));
        assert_eq!(parse(" Software "), Some(Wish::Software));
        assert_eq!(parse("vulkan"), None);
        // OpenGL is a desktop backend; the handheld's GL is its `gpu`.
        assert_eq!(
            parse("GL"),
            cfg!(not(all(target_os = "linux", target_arch = "aarch64"))).then_some(Wish::Gl)
        );
    }

    #[test]
    fn the_environment_outranks_a_wanted_renderer() {
        let wish = |env, wanted| decide(None, env, wanted, |_, _| {});
        assert_eq!(wish(None, None), Wish::GpuFirst);
        assert_eq!(wish(Some(""), None), Wish::GpuFirst);
        assert_eq!(wish(Some("software"), Some("gpu")), Wish::Software);
        assert_eq!(wish(Some("gpu"), Some("software")), Wish::GpuFirst);
        assert_eq!(wish(None, Some("software")), Wish::Software);
    }

    #[test]
    fn the_switch_outranks_the_environment() {
        let wish = |switch, env| decide(switch, env, None, |_, _| {});
        assert_eq!(wish(Some("software"), Some("gpu")), Wish::Software);
        assert_eq!(wish(Some("gpu"), Some("software")), Wish::GpuFirst);
        assert_eq!(wish(None, Some("software")), Wish::Software);
        assert_eq!(wish(Some("vulkan"), Some("software")), Wish::Software);
    }

    #[test]
    fn an_unknown_value_is_reported_and_the_next_source_decides() {
        let mut seen = Vec::new();
        let wish = decide(None, Some("vulkan"), Some("software"), |s, v| {
            seen.push((s.to_string(), v.to_string()));
        });
        assert_eq!(wish, Wish::Software);
        assert_eq!(seen, [("RC2K_RENDERER".to_string(), "vulkan".to_string())]);
    }
}
