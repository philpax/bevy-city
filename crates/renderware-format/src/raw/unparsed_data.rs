use std::fmt::{Debug, Display};

#[derive(PartialEq, Eq)]
pub struct UnparsedData(pub Vec<u8>);
impl UnparsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for byte in &self.0 {
            write!(f, "{:0>2x}", byte)?;
        }
        Ok(())
    }
}

impl Display for UnparsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Debug for UnparsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}
