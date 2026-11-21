//! What the system under the game can do, as constants a page or a menu asks.
//!
//! Each is fixed for a build and folds away, so the branch of a missing capability costs nothing
//! where it is not taken. Code that needs a library only some builds have (Steam's, the
//! network-discovery crate, the touch menu's) stays behind its own condition: a constant cannot
//! hide an item that does not exist in the build.

/// Network play: a rally over a network or the internet. The browser cannot open sockets.
pub const NETWORK: bool = !cfg!(web);

/// Steam's lobbies and invitations: the desktop systems, where the program can load Valve's
/// library when it is installed (not the handheld, nor the phones, nor the browser).
pub const STEAM: bool = cfg!(steam_support);

/// Folders of the player's own on the machine: where the game's files can be looked for and
/// picked. The browser keeps its files in the page's private storage.
pub const FOLDERS: bool = !cfg!(web);

/// A camera to read a QR code with: where the SDL build has the camera subsystem.
pub const CAMERA: bool = cfg!(any(
    windows,
    target_os = "ios",
    target_os = "android",
    web,
    all(target_os = "linux", not(target_arch = "aarch64"))
));

/// A share sheet to send the log with (Android's).
pub const SHARE_SHEET: bool = cfg!(target_os = "android");

/// The OpenGL renderer can be chosen: not on the handheld, whose GPU renderer is GL ES
/// already, nor on Android, where the one window is the GPU renderer's.
pub const OPENGL_CHOICE: bool = !cfg!(any(handheld, target_os = "android"));
