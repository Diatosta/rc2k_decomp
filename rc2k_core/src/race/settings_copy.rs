//! The race's settings copied between the front end's variables and the race's, and
//! logged (converted 2026-10-21 from `race::copy_settings_in` 0x446EBE, "COPY GRAHAM TO
//! SHAUN", and `race::copy_settings_out` 0x4486FB, "COPY SHAUN TO GRAHAM"): two long runs
//! of a few kinds of step, kept as step tables (the hook crate's, made from the
//! disassembly) that `run` does in order as the original's code does. The variables are
//! named by an address type of the caller's; what touches the game (its variables, the
//! x87's conversions, the logs and the calls) goes through `Machine`.

/// How a value is logged after its word (each a `log_..._nl`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Int,
    Hex8,
}

/// One step; jump targets are step numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step<A = usize> {
    /// The text logged (`log_write_l1`, `_nl` with true).
    Text(&'static core::ffi::CStr, bool),
    /// The dword logged (`log_int_l1_nl` / `log_hex8_l1_nl`).
    Value(A, Kind),
    /// The float logged (`log_float_l1_nl`, two decimals).
    Float(A),
    /// `push from` / `pop to`.
    Copy(A, A),
    /// `dec` of the dword.
    Dec(A),
    /// `fild from` / `fstp to`.
    IntToFloat(A, A),
    /// `fld from` / `fistp to`.
    FloatToInt(A, A),
    /// `fld from` / `fstp to`.
    FloatCopy(A, A),
    /// The dword from held: below 0 is 0, 100 or more 0x100, else x 2.56 rounded, into
    /// to.
    Brake(A, A),
    /// `mov ebx, [var]`; `cmp ebx, 0`; `jz`; `or eax, bit`.
    BitIf(A, u32),
    /// eax the dword.
    Load(A),
    /// eax the value.
    LoadImm(u32),
    /// The dword eax.
    Store(A),
    /// The dword the value.
    Put(A, u32),
    /// eax less the dword.
    SubEax(A),
    /// `cmp eax, value` / `jnz`.
    JumpIfEaxNot(u32, usize),
    /// `cmp [var], value` / `jnz`.
    JumpIfNot(A, u32, usize),
    Jump(usize),
    /// A call.
    Call(A),
    Return,
}

/// What the steps run on.
pub trait Machine<A> {
    type Error;
    fn get(&mut self, at: A) -> u32;
    fn set(&mut self, at: A, value: u32);
    fn eax(&self) -> u32;
    fn set_eax(&mut self, value: u32);
    fn set_ebx(&mut self, value: u32);
    /// `fild` / `fstp` of a whole number.
    fn int_to_float(&mut self, value: u32) -> u32;
    /// `fld` / `fistp` of a float (the control word's rounding).
    fn float_to_int(&mut self, value: u32) -> u32;
    /// `fld` / `fstp` of a float.
    fn float_copy(&mut self, value: u32) -> u32;
    /// A brake value 0 .. 99 x 2.56, rounded (`fild` / `fmul` / `fistp`).
    fn scaled_brake(&mut self, value: i32) -> u32;
    /// The logs (inside `pusha` / `popa`: the registers kept).
    fn text(&mut self, text: &'static core::ffi::CStr, newline: bool) -> Result<(), Self::Error>;
    fn value(&mut self, value: u32, kind: Kind) -> Result<(), Self::Error>;
    fn float(&mut self, bits: u32) -> Result<(), Self::Error>;
    fn call(&mut self, at: A) -> Result<(), Self::Error>;
}

/// The brake value from a setting: below 0 is 0, 100 or more 0x100, else `scaled`.
pub fn held_brake(v: i32, scaled: impl FnOnce(i32) -> u32) -> u32 {
    crate::race::cars::held_brake(v, scaled)
}

/// The steps from the first, as the original runs them, up to the `Return`.
pub fn run<A: Copy, M: Machine<A>>(steps: &[Step<A>], m: &mut M) -> Result<(), M::Error> {
    use Step::*;
    let mut pc = 0;
    loop {
        let step = steps[pc];
        pc += 1;
        match step {
            Text(text, newline) => m.text(text, newline)?,
            Value(v, kind) => {
                let v = m.get(v);
                m.value(v, kind)?;
            }
            Float(v) => {
                let v = m.get(v);
                m.float(v)?;
            }
            Copy(to, from) => {
                let v = m.get(from);
                m.set(to, v);
            }
            Dec(at) => {
                let v = m.get(at).wrapping_sub(1);
                m.set(at, v);
            }
            IntToFloat(to, from) => {
                let v = m.get(from);
                let v = m.int_to_float(v);
                m.set(to, v);
            }
            FloatToInt(to, from) => {
                let v = m.get(from);
                let v = m.float_to_int(v);
                m.set(to, v);
            }
            FloatCopy(to, from) => {
                let v = m.get(from);
                let v = m.float_copy(v);
                m.set(to, v);
            }
            Brake(to, from) => {
                let v = m.get(from) as i32;
                let v = held_brake(v, |v| m.scaled_brake(v));
                m.set(to, v);
            }
            BitIf(var, bit) => {
                let v = m.get(var);
                m.set_ebx(v);
                if v != 0 {
                    let eax = m.eax() | bit;
                    m.set_eax(eax);
                }
            }
            Load(at) => {
                let v = m.get(at);
                m.set_eax(v);
            }
            LoadImm(v) => m.set_eax(v),
            Store(at) => {
                let v = m.eax();
                m.set(at, v);
            }
            Put(at, v) => m.set(at, v),
            SubEax(at) => {
                let v = m.eax().wrapping_sub(m.get(at));
                m.set_eax(v);
            }
            JumpIfEaxNot(v, to) => {
                if m.eax() != v {
                    pc = to;
                }
            }
            JumpIfNot(var, v, to) => {
                if m.get(var) != v {
                    pc = to;
                }
            }
            Jump(to) => pc = to,
            Call(at) => m.call(at)?,
            Return => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A machine on a map of variables, noting what it is asked; past 1000 reads it
    /// panics (a runaway table).
    #[derive(Default)]
    struct Test {
        vars: HashMap<u32, u32>,
        eax: u32,
        ebx: u32,
        log: Vec<String>,
        reads: u32,
    }

    impl Machine<u32> for Test {
        type Error = ();
        fn get(&mut self, at: u32) -> u32 {
            self.reads += 1;
            assert!(self.reads < 1000, "runaway");
            *self.vars.get(&at).unwrap_or(&0)
        }
        fn set(&mut self, at: u32, value: u32) {
            self.vars.insert(at, value);
        }
        fn eax(&self) -> u32 {
            self.eax
        }
        fn set_eax(&mut self, value: u32) {
            self.eax = value;
        }
        fn set_ebx(&mut self, value: u32) {
            self.ebx = value;
        }
        fn int_to_float(&mut self, value: u32) -> u32 {
            (value as i32 as f32).to_bits()
        }
        fn float_to_int(&mut self, value: u32) -> u32 {
            f32::from_bits(value).round() as i32 as u32
        }
        fn float_copy(&mut self, value: u32) -> u32 {
            value
        }
        fn scaled_brake(&mut self, value: i32) -> u32 {
            (value as f32 * 2.56).round() as u32
        }
        fn text(&mut self, text: &'static core::ffi::CStr, newline: bool) -> Result<(), ()> {
            self.log
                .push(format!("text {} {newline}", text.to_str().unwrap()));
            Ok(())
        }
        fn value(&mut self, value: u32, kind: Kind) -> Result<(), ()> {
            self.log.push(format!("{kind:?} {value}"));
            Ok(())
        }
        fn float(&mut self, bits: u32) -> Result<(), ()> {
            self.log.push(format!("float {}", f32::from_bits(bits)));
            Ok(())
        }
        fn call(&mut self, at: u32) -> Result<(), ()> {
            self.log.push(format!("call {at}"));
            if at == 99 {
                return Err(());
            }
            self.eax = 7;
            Ok(())
        }
    }

    #[test]
    fn steps() {
        use Step::*;
        let table: [Step<u32>; 16] = [
            Text(c"one", true),
            Copy(10, 11),
            Dec(10),
            Value(10, Kind::Int),
            IntToFloat(12, 10),
            Float(12),
            FloatToInt(13, 12),
            Brake(14, 15),
            Brake(16, 17),
            LoadImm(0),
            BitIf(18, 4),
            BitIf(19, 8),
            Store(20),
            JumpIfEaxNot(4, 15),
            Put(21, 1),
            Return,
        ];
        let mut m = Test::default();
        m.vars.extend([(11, 5), (15, 50), (17, u32::MAX), (18, 3)]);
        run(&table, &mut m).unwrap();
        assert_eq!(m.vars[&10], 4);
        assert_eq!(m.vars[&13], 4);
        assert_eq!(m.vars[&14], 128);
        assert_eq!(m.vars[&16], 0);
        assert_eq!(m.vars[&20], 4);
        assert_eq!(m.ebx, 0);
        // eax 4: no jump, the put done.
        assert_eq!(m.vars[&21], 1);
        assert_eq!(m.log, ["text one true", "Int 4", "float 4"]);
        let calls = [
            Load(1),
            SubEax(2),
            JumpIfNot(3, 0, 4),
            Call(5),
            Call(99),
            Return,
        ];
        let mut m = Test::default();
        m.vars.extend([(1, 10), (2, 3), (3, 1)]);
        assert_eq!(run(&calls, &mut m), Err(()));
        assert_eq!(m.log, ["call 99"]);
        assert_eq!(m.eax, 7);
        let mut m = Test::default();
        run(&[Load(1), Call(5), Jump(4), Put(4, 4), Return], &mut m).unwrap();
        assert_eq!((m.eax, m.vars.get(&4)), (7, None));
    }
}
