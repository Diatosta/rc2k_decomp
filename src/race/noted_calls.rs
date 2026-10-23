//! For the self-tests of functions that are mostly calls: a copy of the original whose
//! calls go to stand-ins by place, each noting its place and the registers (which it
//! leaves as they came), and one case run on the copy and on the Rust version with the
//! same stand-ins, the calls, the registers and the memory compared.

use crate::errors::Refused;
use crate::framework::Regs;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
