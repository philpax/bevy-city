use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z)) = tuple((nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((input, Vec3 { x, y, z }))
    }

    pub fn as_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Mat3(pub [f32; 9]);
impl Mat3 {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, data) = nom::multi::count(nc::le_f32, 9)(input)?;

        let mut buf = [0.0; 9];
        buf.copy_from_slice(&data);
        Ok((input, Mat3(buf)))
    }
}
