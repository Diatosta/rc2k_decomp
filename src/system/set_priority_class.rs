//! The process's priority class set to the given class
//! (`SetPriorityClass(GetCurrentProcess(), class)`), unless a switch of the start-up is
//! not 0.

pub fn set_priority_class(system: &crate::system::owner::System, class: u32) {
    if system.window_loop().keep_priority.get() == 0 {
        crate::platform::os::set_priority_class(class);
    }
}
