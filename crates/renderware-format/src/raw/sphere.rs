use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Sphere {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
}

impl Sphere {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z, radius)) =
            tuple((nc::le_f32, nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((input, Sphere { x, y, z, radius }))
    }
}
