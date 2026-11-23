//! A place in a plain record that a table of fields hands out: one word of a record, whatever
//! its type (`u32`, `i32`, `f32`), read and written as its bits.

/// One word of a plain record, to write.
pub enum WordMut<'a> {
    Dword(&'a mut u32),
    Int(&'a mut i32),
    Float(&'a mut f32),
}

impl WordMut<'_> {
    /// The word's bits.
    pub fn bits(&self) -> u32 {
        match self {
            Self::Dword(v) => **v,
            Self::Int(v) => **v as u32,
            Self::Float(v) => v.to_bits(),
        }
    }

    /// The word put in, as bits.
    pub fn set_bits(&mut self, bits: u32) {
        match self {
            Self::Dword(v) => **v = bits,
            Self::Int(v) => **v = bits as i32,
            Self::Float(v) => **v = f32::from_bits(bits),
        }
    }
}

impl<'a> From<&'a mut u32> for WordMut<'a> {
    fn from(v: &'a mut u32) -> Self {
        Self::Dword(v)
    }
}

impl<'a> From<&'a mut i32> for WordMut<'a> {
    fn from(v: &'a mut i32) -> Self {
        Self::Int(v)
    }
}

impl<'a> From<&'a mut f32> for WordMut<'a> {
    fn from(v: &'a mut f32) -> Self {
        Self::Float(v)
    }
}
