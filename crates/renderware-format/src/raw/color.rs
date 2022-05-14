use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r, g, b, a)) = tuple((nc::u8, nc::u8, nc::u8, nc::u8))(input)?;
        Ok((input, Color { r, g, b, a }))
    }
}
