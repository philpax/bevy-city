use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r, g, b, a)) = tuple((nc::u8, nc::u8, nc::u8, nc::u8))(input)?;
        Ok((input, Color { r, g, b, a }))
    }

    pub fn as_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02X}{:02X}{:02X}{:02X}",
            self.r, self.g, self.b, self.a
        )
    }
}
