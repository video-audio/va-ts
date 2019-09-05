use std::fmt;

#[derive(Debug)]
pub struct ISO639([char; 3]);

impl ISO639 {
    pub fn must_from_bytes_3(b: &[u8]) -> ISO639 {
        ISO639([char::from(b[0]), char::from(b[1]), char::from(b[2])])
    }
}

impl fmt::Display for ISO639 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.0[0], self.0[1], self.0[2])
    }
}
