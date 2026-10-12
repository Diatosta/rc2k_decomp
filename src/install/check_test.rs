//! For the self-tests of the installation checks: what a test says about the world
//! outside (which files exist, which drives are CD-ROMs, which directories can be gone
//! to, what the installation's record says), and the function that answers the
//! stand-ins and the Rust versions from it.

use crate::framework::{Regs, game_addr, game_write, rva};
use crate::install::outside::{
    DRIVE_TYPE, FILE_EXISTS, LOG, READ_RECORD, READ_REGISTRY, SET_DIRECTORY,
};
use crate::install::read_registry::{SOURCE_RVA, STATUS_RVA};
use std::sync::Mutex;

pub const MEDIA_MODE_RVA: usize = rva(0x5189A4); // g_mediaMode

#[derive(Clone, Default)]
pub struct World {
    /// Paths that exist, in lower case.
    pub files: Vec<&'static [u8]>,
    pub cd_drives: &'static [u8],
    /// Directories that can be made the current one, in lower case.
    pub directories: Vec<&'static [u8]>,
    /// What reading the record leaves in `g_mediaMode`, if anything.
    pub record_mode: Option<u32>,
    /// The registry's status and source, if it has the game's key. (The owner's has it,
    /// with a status of 0x80 and no source, which is why the tests do not ask it.)
    pub registry: Option<(u8, Vec<u8>)>,
}

static WORLD: Mutex<(World, Vec<String>)> = Mutex::new((
    World {
        files: Vec::new(),
        cd_drives: &[],
        directories: Vec::new(),
        record_mode: None,
        registry: None,
    },
    Vec::new(),
));

/// Starts a run in a world; nothing asked yet.
pub fn begin(world: &World) {
    *WORLD.lock().unwrap() = (world.clone(), Vec::new());
    crate::install::stand_ins::set_handler(answer);
}

/// What was asked and done in the run, in order.
pub fn asked() -> Vec<String> {
    WORLD.lock().unwrap().1.clone()
}

fn text(at: u32) -> Vec<u8> {
    unsafe { std::ffi::CStr::from_ptr(at as *const _) }
        .to_bytes()
        .to_ascii_lowercase()
}

fn answer(place: u32, regs: &mut Regs) {
    let mut guard = WORLD.lock().unwrap();
    let (world, asked) = &mut *guard;
    match place {
        FILE_EXISTS => {
            let path = text(regs.eax);
            let exists = world.files.iter().any(|file| *file == &path[..]);
            asked.push(format!("exists {:?}", String::from_utf8_lossy(&path)));
            regs.set_carry(!exists);
            regs.set_zero(exists);
        }
        DRIVE_TYPE => {
            let letter = regs.eax as u8;
            regs.eax = if world.cd_drives.contains(&letter) {
                5
            } else {
                3
            };
        }
        SET_DIRECTORY => {
            let path = text(regs.edx);
            let done = world
                .directories
                .iter()
                .any(|directory| *directory == &path[..]);
            asked.push(format!("go to {:?}", String::from_utf8_lossy(&path)));
            regs.set_carry(!done);
        }
        READ_RECORD => {
            asked.push("read the record".into());
            if let Some(mode) = world.record_mode {
                unsafe { game_write(MEDIA_MODE_RVA, mode) };
            }
        }
        READ_REGISTRY => {
            asked.push("read the registry".into());
            if let Some((status, source)) = &world.registry {
                unsafe {
                    game_write(STATUS_RVA, *status);
                    let place = game_addr(SOURCE_RVA) as *mut u8;
                    std::ptr::copy(source.as_ptr(), place, source.len());
                    place.add(source.len()).write(0);
                }
            }
        }
        LOG => asked.push(format!(
            "log {:?}",
            String::from_utf8_lossy(&text(regs.eax))
        )),
        _ => asked.push(format!("place {place}?")),
    }
}
