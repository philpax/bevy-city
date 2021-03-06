use nom::{number::complete as nc, IResult};

use super::{Mat3, Vec3};

#[derive(Debug, PartialEq)]
pub struct Frame {
    pub rotation: Mat3,
    pub translation: Vec3,
}
impl Frame {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, rotation) = Mat3::parse(input)?;
        let (input, translation) = Vec3::parse(input)?;
        let (input, _current_frame_index) = nc::le_u32(input)?;
        let (input, _matrix_creation_flags) = nc::le_u32(input)?;

        Ok((
            input,
            Frame {
                rotation,
                translation,
            },
        ))
    }
}
