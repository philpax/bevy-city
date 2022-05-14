use nom::{number::complete as nc, sequence::tuple, IResult};

use super::Vec3;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z, radius)) =
            tuple((nc::le_f32, nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        let position = Vec3 { x, y, z };
        Ok((input, Sphere { position, radius }))
    }
}
