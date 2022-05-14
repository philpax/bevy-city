use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}
impl Vec3 {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z)) = tuple((nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((input, Vec3 { x, y, z }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Mat3([Vec3; 3]);
impl Mat3 {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r0, r1, r2)) = tuple((Vec3::parse, Vec3::parse, Vec3::parse))(input)?;
        Ok((input, Mat3([r0, r1, r2])))
    }
}
