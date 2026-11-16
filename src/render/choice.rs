//! Which renderer the game asks for. `RC2K_RENDERER=software` forces the software rasteriser;
//! `gpu` (or nothing) means the default order: the GPU first, each renderer that cannot start
//! falling back to the next. Desktop: the SDL GPU device, then SDL's own 2D renderer with the
//! software rasteriser. Handheld: OpenGL ES, then the rasteriser into the framebuffer. What
//! actually started is asked of `render::gpu_running`, not of this module.
//!
//! The choice is read afresh at each call, so a later start of the renderers can ask again,
//! with a wanted renderer passed to `decide` in place of `None`.

/// What is asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wish {
    /// The GPU if it starts, else the software rasteriser.
    GpuFirst,
    /// The software rasteriser only.
    Software,
}

/// The wish for a renderer name; `None` for one this platform does not know.
pub fn parse(value: &str) -> Option<Wish> {
    match value.trim().to_ascii_lowercase().as_str() {
        "gpu" => Some(Wish::GpuFirst),
        "gles" if cfg!(all(target_os = "linux", target_arch = "aarch64")) => Some(Wish::GpuFirst),
        "software" => Some(Wish::Software),
        _ => None,
    }
}

/// The renderer wish from the environment's value, `wanted` (a renderer asked for some other
/// way; the environment outranks it) and the default order. An unknown value is passed to
/// `unknown` and skipped.
pub fn decide(
    env: Option<&str>,
    wanted: Option<&str>,
    mut unknown: impl FnMut(&str, &str),
) -> Wish {
    for (source, value) in [("RC2K_RENDERER", env), ("the setting", wanted)] {
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

/// What the environment asks for.
pub fn wish() -> Wish {
    let env = std::env::var("RC2K_RENDERER").ok();
    decide(env.as_deref(), None, |source, value| {
        crate::logfile::program_log::log(&format!(
            "render: {source}: renderer `{value}` is not known here (gpu, gles or software): \
             the default order"
        ));
    })
}

/// Whether a GPU renderer is to be tried.
pub fn gpu_wanted() -> bool {
    wish() == Wish::GpuFirst
}

/// With the `remote` feature, `RC2K_FAIL_RENDERER=<step>[,<step>..]` makes the named start-up
/// steps fail, to test the fallbacks: `device`, `gpu_renderer`, `backend`, `size` (desktop),
/// `gles` (handheld). Always false without the feature.
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
    }

    #[test]
    fn the_environment_outranks_a_wanted_renderer() {
        let wish = |env, wanted| decide(env, wanted, |_, _| {});
        assert_eq!(wish(None, None), Wish::GpuFirst);
        assert_eq!(wish(Some(""), None), Wish::GpuFirst);
        assert_eq!(wish(Some("software"), Some("gpu")), Wish::Software);
        assert_eq!(wish(Some("gpu"), Some("software")), Wish::GpuFirst);
        assert_eq!(wish(None, Some("software")), Wish::Software);
    }

    #[test]
    fn an_unknown_value_is_reported_and_the_next_source_decides() {
        let mut seen = Vec::new();
        let wish = decide(Some("vulkan"), Some("software"), |s, v| {
            seen.push((s.to_string(), v.to_string()));
        });
        assert_eq!(wish, Wish::Software);
        assert_eq!(seen, [("RC2K_RENDERER".to_string(), "vulkan".to_string())]);
    }
}
