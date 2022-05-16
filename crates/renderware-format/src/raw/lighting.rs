use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Lighting {
    pub ambient: f32,
    pub specular: f32,
    pub diffuse: f32,
}
impl Lighting {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (ambient, specular, diffuse)) =
            tuple((nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((
            input,
            Lighting {
                ambient,
                specular,
                diffuse,
            },
        ))
    }
}
