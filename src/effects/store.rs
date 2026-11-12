//! The effects' variables' place in RAL.EXE (IDA 0x764F40 .. 0x93CCC8): the weather and
//! surface effects' settings, floats, work areas and tables, each a typed static of the
//! module that owns it. The tests' copies of the original's x87 runs reach their operands
//! by IDA's address less `START` (`effects::asm_test::with_copy`).

/// IDA's address of the effects' first byte.
pub const START: usize = 0x764F40;
