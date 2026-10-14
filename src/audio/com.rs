//! Calls through a COM object's table (DirectSound's buffers, the listener, A3D), and test
//! objects whose methods note their calls instead.

use std::cell::RefCell;

/// The method at `offset` of the object's table.
pub unsafe fn method(object: u32, offset: u32) -> usize {
    unsafe {
        let table = (object as *const u32).read_unaligned();
        ((table + offset) as *const u32).read_unaligned() as usize
    }
}

/// A method that takes only the object.
pub unsafe fn call(object: u32, offset: u32) -> i32 {
    unsafe {
        let f: unsafe extern "system" fn(u32) -> i32 = std::mem::transmute(method(object, offset));
        f(object)
    }
}

/// A method that takes one dword more.
pub unsafe fn call1(object: u32, offset: u32, a: u32) -> i32 {
    unsafe {
        let f: unsafe extern "system" fn(u32, u32) -> i32 =
            std::mem::transmute(method(object, offset));
        f(object, a)
    }
}

pub const RELEASE: u32 = 8;

// ---- test objects ----

/// A call a test object noted: the object, the method's offset, its arguments.
pub type Noted = (u32, u32, Vec<u32>);

thread_local!(static NOTED: RefCell<Vec<Noted>> = const { RefCell::new(Vec::new()) });

fn note(this: u32, offset: u32, args: &[u32]) -> i32 {
    NOTED.with(|n| n.borrow_mut().push((this, offset, args.to_vec())));
    0
}

/// The calls noted since the last `take_noted`.
pub fn take_noted() -> Vec<Noted> {
    NOTED.with(|n| std::mem::take(&mut *n.borrow_mut()))
}

macro_rules! stubs {
    ($($one:ident / $two:ident / $five:ident = $offset:expr;)*) => {
        $(
            unsafe extern "system" fn $one(this: u32) -> i32 { note(this, $offset, &[]) }
            unsafe extern "system" fn $two(this: u32, a: u32) -> i32 { note(this, $offset, &[a]) }
            unsafe extern "system" fn $five(this: u32, a: u32, b: u32, c: u32, d: u32) -> i32 {
                note(this, $offset, &[a, b, c, d])
            }
        )*
        /// The method at an offset, as a stub taking `args` dwords after the object.
        fn stub(offset: u32, args: u32) -> u32 {
            match (offset, args) {
                $(($offset, 0) => $one as *const () as u32,
                  ($offset, 1) => $two as *const () as u32,
                  ($offset, 4) => $five as *const () as u32,)*
                _ => 0,
            }
        }
    };
}

stubs! {
    one_08 / two_08 / four_08 = 0x08;
    one_24 / two_24 / four_24 = 0x24;
    one_30 / two_30 / four_30 = 0x30;
    one_34 / two_34 / four_34 = 0x34;
    one_38 / two_38 / four_38 = 0x38;
    one_3c / two_3c / four_3c = 0x3C;
    one_40 / two_40 / four_40 = 0x40;
    one_44 / two_44 / four_44 = 0x44;
    one_48 / two_48 / four_48 = 0x48;
    one_4c / two_4c / four_4c = 0x4C;
}

/// Objects of the test's: each two dwords, its table and a number; the table's methods at
/// the offsets given take the given number of dwords after the object (0, 1 or 4).
pub struct TestObjects {
    table: Box<[u32; 0x20]>,
    objects: Box<[[u32; 2]]>,
}

impl TestObjects {
    pub fn new(count: usize, methods: &[(u32, u32)]) -> Self {
        let mut table = Box::new([0u32; 0x20]);
        for &(offset, args) in methods {
            table[offset as usize / 4] = stub(offset, args);
        }
        let at = table.as_ptr() as u32;
        let objects = (0..count as u32).map(|n| [at, n]).collect();
        take_noted();
        Self { table, objects }
    }

    /// The address of object `n`.
    pub fn get(&self, n: usize) -> u32 {
        let _ = &self.table;
        self.objects[n].as_ptr() as u32
    }
}

/// The method layout of a DirectSound buffer, as far as the audio thread uses it:
/// `Release`, `GetStatus`, `Play` (three dwords), `SetCurrentPosition`, `SetVolume`,
/// `SetPan`, `SetFrequency`, `Stop`. (`Play` and `GetStatus` are left out: none of the
/// functions tested with these objects calls them.)
pub const BUFFER_METHODS: &[(u32, u32)] = &[(0x08, 0), (0x3C, 1), (0x40, 1), (0x44, 1), (0x48, 0)];
/// A3D's, as far as used: `Release`, its stop (+0x38), +0x3C (no argument).
pub const A3D_METHODS: &[(u32, u32)] = &[(0x08, 0), (0x38, 0), (0x3C, 0)];
