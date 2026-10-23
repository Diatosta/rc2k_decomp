//! Calls through a COM object's table (DirectSound's buffers, the listener, A3D), and test
//! objects whose methods note their calls instead.

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
