use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl Color {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r, g, b, a)) = tuple((nc::u8, nc::u8, nc::u8, nc::u8))(input)?;
        Ok((input, Color { r, g, b, a }))
    }
}
