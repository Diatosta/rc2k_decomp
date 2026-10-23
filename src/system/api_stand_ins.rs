//! For self-tests: stand-ins for the Win32 functions an original calls (`stdcall`, the
//! arguments on the stack, popped by the callee). Each notes its place and its arguments
//! and gives in eax what the test set for the place (`set_gives`); ecx and edx are kept.
//! The Rust versions' calls of the same functions go through `Api` (in the game the real
//! functions, in the tests `note`), so both sides' calls can be compared.
//! `stand_in(place, args)` gives the address of the stand-in for a place that takes
//! `args` arguments (places 0 .. 7, 0 .. 6 arguments).

/// A Win32 call by place: the arguments in, eax out.
pub type Api<'a> = &'a mut dyn FnMut(u32, &[u32]) -> u32;
