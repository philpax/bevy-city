use nom::{number::complete as nc, sequence::tuple, IResult};

#[derive(Debug, PartialEq)]
pub struct Lighting {
    ambient: f32,
    specular: f32,
    diffuse: f32,
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
