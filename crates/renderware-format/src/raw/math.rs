use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z)) = tuple((nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((input, Vec3 { x, y, z }))
    }

    pub fn as_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Debug, PartialEq)]
pub struct Mat3(pub [Vec3; 3]);
impl Mat3 {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r0, r1, r2)) = tuple((Vec3::parse, Vec3::parse, Vec3::parse))(input)?;
        Ok((input, Mat3([r0, r1, r2])))
    }
}
