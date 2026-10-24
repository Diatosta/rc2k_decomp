//! Phase 5, the session core: the variables many modules and the game's threads share,
//! moved out of RAL.EXE's image (`moved.rs`). The screen's size and format, the window's
//! state (`g_windowActive`, `g_quitting`, ...), what the processor is and has, the window
//! handles, and the clocks the key thread and the menus keep.
//!
//! Each dword is an atomic (`Dword`, plain loads and stores as the game's code did: the
//! audio, key and network threads read them without a lock). The blocks keep the image's
//! layout, so that an address taken of a field is the field's place and a walk from one
//! variable to its neighbour reads the same neighbour (the vendor text of the processor,
//! three dwords; the surface format, six dwords; the Windows version record).

use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// One of the game's dwords: an atomic cell the game's threads read and write.
#[repr(transparent)]
pub struct Dword(AtomicU32);

impl Dword {
    pub const fn new(value: u32) -> Self {
        Self(AtomicU32::new(value))
    }

    pub fn get(&self) -> u32 {
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u32) {
        self.0.store(value, Relaxed)
    }

    /// The atomic itself, for code that wants more than a load and a store.
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
    }

    /// Its place, for code that is handed an address (a Windows call, an `fild`).
    pub fn as_ptr(&self) -> *mut u32 {
        self.0.as_ptr()
    }

    pub fn addr(&self) -> usize {
        self.as_ptr() as usize
    }
}

const fn zeros<const N: usize>() -> [Dword; N] {
    [const { Dword::new(0) }; N]
}

/// The screen: 0x4F494C .. 0x4F4984.
#[repr(C)]
pub struct Screen {
    pub width: Dword,       // g_screenWidth
    pub height: Dword,      // g_screenHeight
    pub bits: Dword,        // g_screenBits
    pub bytes: Dword,       // g_screenBytesPerPixel
    pub row_bytes: Dword,   // g_screenRowBytes
    pub row_dwords: Dword,  // g_screenRowDwords
    pub depth_bits: Dword,  // dword_4F4964
    pub kind: Dword,        // dword_4F4968
    pub format: [Dword; 6], // g_surfaceFormat (a pixel format record, 24 bytes)
}

impl Screen {
    /// Where the pixel format record is (a Windows-style record the game's code reads and
    /// writes by address).
    pub fn format_addr(&self) -> usize {
        self.format.as_ptr() as usize
    }
}

pub static SCREEN: Screen = Screen {
    width: Dword::new(0),
    height: Dword::new(0),
    bits: Dword::new(0),
    bytes: Dword::new(0),
    row_bytes: Dword::new(0),
    row_dwords: Dword::new(0),
    depth_bits: Dword::new(0),
    kind: Dword::new(0),
    format: zeros(),
};

pub const SCREEN_IDA: usize = 0x4F494C;
pub const SCREEN_LEN: usize = core::mem::size_of::<Screen>();

/// The window's state and the main loop's: 0x518370 .. 0x51839C.
#[repr(C)]
pub struct Flags {
    pub f370: Dword,            // dword_518370
    pub window_active: Dword,   // g_windowActive
    pub window_focused: Dword,  // g_windowFocused
    pub quitting: Dword,        // g_quitting
    pub lost: Dword,            // dword_518380
    pub no_task_switch: Dword,  // dword_518384 (`notaskswitch`, a `var\ral.cfg` setting)
    pub keep_priority: Dword,   // g_keepPriority (`noprichange`, a setting)
    pub f38c: Dword,            // dword_51838C
    pub main_loop_stack: Dword, // g_mainLoopStack
    pub f394: Dword,            // dword_518394 (1 at start)
    pub paused: Dword,          // dword_518398
}

pub static FLAGS: Flags = Flags {
    f370: Dword::new(0),
    window_active: Dword::new(0),
    window_focused: Dword::new(0),
    quitting: Dword::new(0),
    lost: Dword::new(0),
    no_task_switch: Dword::new(0),
    keep_priority: Dword::new(0),
    f38c: Dword::new(0),
    main_loop_stack: Dword::new(0),
    f394: Dword::new(1),
    paused: Dword::new(0),
};

pub const FLAGS_IDA: usize = 0x518370;
pub const FLAGS_LEN: usize = core::mem::size_of::<Flags>();

/// What the machine is and has, and the window handles: 0x5183A8 .. 0x5184A8.
#[repr(C)]
pub struct Machine {
    pub has_mmx: Dword,          // g_hasMmx
    pub ignore_mmx: Dword,       // dword_5183AC (`ignoremmx`)
    pub has_sse: Dword,          // g_hasSse
    pub ignore_katmai: Dword,    // dword_5183B4 (`ignorekatmai`)
    pub has_3dnow: Dword,        // g_has3dNow
    pub ignore_3dnow: Dword,     // dword_5183BC (`ignoreamdk3d`)
    pub work: Dword,             // dword_5183C0
    pub work_float: Dword,       // flt_5183C4
    pub gap_c8: Dword,           // 0x5183C8
    pub scratch: Dword,          // g_soundScratch: the x87 code's scratch dword
    pub cpu_family: Dword,       // g_cpuFamily
    pub cpu_signature: Dword,    // g_cpuSignature
    pub cpu_vendor: [Dword; 3],  // g_cpuVendor: the vendor's text
    pub gap_e4: Dword,           // 0x5183E4
    pub cpu_mhz: Dword,          // g_cpuMhz
    pub has_fpu: Dword,          // g_hasFpu
    pub has_tsc: Dword,          // g_hasTsc
    pub has_cmov: Dword,         // g_hasCmov
    pub found_mmx: Dword,        // dword_5183F8
    pub found_sse: Dword,        // dword_5183FC
    pub found_3dnow: Dword,      // dword_518400
    pub keyboard_type: Dword,    // g_keyboardType (1 at start)
    pub version: [Dword; 37],    // VersionInformation (OSVERSIONINFOA, 148 bytes)
    pub exception_resume: Dword, // g_exceptionResume
    pub instance: Dword,         // hInstance
    pub window: Dword,           // hWnd
}

impl Machine {
    /// The processor vendor's text (12 bytes).
    pub fn vendor_addr(&self) -> usize {
        self.cpu_vendor.as_ptr() as usize
    }

    /// The Windows version record (`OSVERSIONINFOA`, 148 bytes).
    pub fn version_addr(&self) -> usize {
        self.version.as_ptr() as usize
    }
}

pub static MACHINE: Machine = Machine {
    has_mmx: Dword::new(0),
    ignore_mmx: Dword::new(0),
    has_sse: Dword::new(0),
    ignore_katmai: Dword::new(0),
    has_3dnow: Dword::new(0),
    ignore_3dnow: Dword::new(0),
    work: Dword::new(0),
    work_float: Dword::new(0),
    gap_c8: Dword::new(0),
    scratch: Dword::new(0),
    cpu_family: Dword::new(0),
    cpu_signature: Dword::new(0),
    cpu_vendor: zeros(),
    gap_e4: Dword::new(0),
    cpu_mhz: Dword::new(0),
    has_fpu: Dword::new(0),
    has_tsc: Dword::new(0),
    has_cmov: Dword::new(0),
    found_mmx: Dword::new(0),
    found_sse: Dword::new(0),
    found_3dnow: Dword::new(0),
    keyboard_type: Dword::new(1),
    version: zeros(),
    exception_resume: Dword::new(0),
    instance: Dword::new(0),
    window: Dword::new(0),
};

pub const MACHINE_IDA: usize = 0x5183A8;
pub const MACHINE_LEN: usize = core::mem::size_of::<Machine>();

/// The key thread's count and the clocks the menus keep: 0x518998 .. 0x5189A4.
#[repr(C)]
pub struct Clocks {
    pub key_ticks: Dword,  // dword_518998: the key thread's turns
    pub frame: Dword,      // dword_51899C: the menus' clock (`update_frame_clock`)
    pub setup_time: Dword, // dword_5189A0
}

pub static CLOCKS: Clocks = Clocks {
    key_ticks: Dword::new(0),
    frame: Dword::new(0),
    setup_time: Dword::new(0),
};

pub const CLOCKS_IDA: usize = 0x518998;
pub const CLOCKS_LEN: usize = core::mem::size_of::<Clocks>();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;

    fn bytes_of<T>(block: &'static T) -> Vec<u8> {
        let p = block as *const T as *const u8;
        (0..core::mem::size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_blocks_start_as_the_images_bytes() {
        assert_eq!(bytes_of(&SCREEN), image_bytes(SCREEN_IDA, SCREEN_LEN));
        assert_eq!(bytes_of(&FLAGS), image_bytes(FLAGS_IDA, FLAGS_LEN));
        assert_eq!(bytes_of(&MACHINE), image_bytes(MACHINE_IDA, MACHINE_LEN));
        assert_eq!(bytes_of(&CLOCKS), image_bytes(CLOCKS_IDA, CLOCKS_LEN));
    }

    #[test]
    fn the_blocks_keep_the_images_layout() {
        use core::mem::offset_of;
        assert_eq!(SCREEN_LEN, 0x4F4984 - 0x4F494C);
        assert_eq!(offset_of!(Screen, format), 0x4F496C - 0x4F494C);
        assert_eq!(FLAGS_LEN, 0x51839C - 0x518370);
        assert_eq!(offset_of!(Flags, quitting), 0x51837C - 0x518370);
        assert_eq!(offset_of!(Flags, paused), 0x518398 - 0x518370);
        assert_eq!(MACHINE_LEN, 0x5184A8 - 0x5183A8);
        assert_eq!(offset_of!(Machine, scratch), 0x5183CC - 0x5183A8);
        assert_eq!(offset_of!(Machine, cpu_vendor), 0x5183D8 - 0x5183A8);
        assert_eq!(offset_of!(Machine, cpu_mhz), 0x5183E8 - 0x5183A8);
        assert_eq!(offset_of!(Machine, found_3dnow), 0x518400 - 0x5183A8);
        assert_eq!(offset_of!(Machine, version), 0x518408 - 0x5183A8);
        assert_eq!(offset_of!(Machine, exception_resume), 0x51849C - 0x5183A8);
        assert_eq!(offset_of!(Machine, window), 0x5184A4 - 0x5183A8);
        assert_eq!(CLOCKS_LEN, 12);
    }
}
