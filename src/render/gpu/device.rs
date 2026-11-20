//! Making the GPU device and starting and stopping the backend on it.

use super::*;

/// Whether a device is to be tried: unless `--renderer software` or `RC2K_RENDERER=software`.
pub fn wanted() -> bool {
    // The browser has no GPU device API; its GPU is the WebGL of `gles`.
    cfg!(not(web)) && crate::render::choice::gpu_wanted()
}

pub(super) fn sdl_error() -> String {
    sdl_gpu::error()
}

/// A device made ahead of the window (`probe_early`), until `make_device` takes it.
pub(super) static EARLY: Mutex<Option<Device>> = Mutex::new(None);

/// Whether the early probe found no device.
pub(super) static NO_DEVICE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// A device that takes our shaders; `None` (logged) when the system has none. The device the
/// early probe made is the one handed out first.
pub fn make_device() -> Option<Device> {
    if let Some(device) = EARLY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
    {
        return Some(device);
    }
    let device = Device::create();
    if device.is_none() {
        log(&format!("render: no GPU device: {}", sdl_error()));
    }
    device
}

/// Phones (Android): whether a GPU device can be made is found out before the screen modes are
/// chosen, because the race's size is chosen with it (`software_phone`): a phone whose driver
/// SDL refuses draws the 3D on the software rasteriser, which cannot carry the screen's full size.
/// The device made is kept for the window's renderer.
#[cfg(target_os = "android")]
pub fn probe_early() {
    if !wanted() {
        return;
    }
    let device = Device::create();
    match device {
        Some(device) => {
            log(&format!(
                "render: a GPU device is available ({})",
                device.driver()
            ));
            *EARLY
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(device);
        }
        None => {
            log(&format!("render: no GPU device: {}", sdl_error()));
            NO_DEVICE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Whether this is a phone whose 3D goes on the software rasteriser: no GPU device could be made,
/// or the software rasteriser was asked for.
pub fn software_phone() -> bool {
    cfg!(target_os = "android")
        && (NO_DEVICE.load(std::sync::atomic::Ordering::Relaxed) || !wanted())
}

/// Starts the backend on `device`; whether it runs. A backend that does not start has let go
/// of what it made (the device is the caller's).
pub fn start(device: Device) -> bool {
    let made = if crate::render::choice::forced_failure("backend") {
        Err("made to fail".to_string())
    } else {
        Gpu::new(device)
    };
    match made {
        Ok(gpu) => {
            log(&format!("render: the GPU backend on {}", device.driver()));
            *GPU.lock().unwrap() = Some(gpu);
            crate::render::gpu_started();
            true
        }
        Err(why) => {
            log(&format!(
                "render: the GPU backend does not start ({why}): the software rasteriser draws the 3D"
            ));
            false
        }
    }
}

/// Stops the backend if it runs: everything it made is let go, and the device is handed back
/// (the caller destroys it once the window's renderer on it is gone).
pub fn stop() -> Option<Device> {
    let gpu = GPU.lock().unwrap().take()?;
    crate::render::gpu_stopped();
    let device = gpu.device;
    gpu.release();
    Some(device)
}
