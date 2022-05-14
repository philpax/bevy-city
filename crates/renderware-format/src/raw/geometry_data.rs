use nom::{number::complete as nc, sequence::tuple, IResult};

use super::{Color, Triangle};

#[derive(Debug, PartialEq)]
pub struct GeometryData {
    pub prelit_color: Option<Vec<Color>>,
    pub texture_sets: Vec<Vec<(f32, f32)>>,
    pub triangles: Vec<Triangle>,
}
impl GeometryData {
    pub(crate) fn parse(
        input: &[u8],
        is_prelit: bool,
        texture_set_count: u16,
        vertices_count: u32,
        triangle_count: u32,
    ) -> IResult<&[u8], Self> {
        let (input, prelit_color) = nom::combinator::cond(
            is_prelit,
            nom::multi::count(Color::parse, vertices_count as usize),
        )(input)?;

        let tex_coord_parser = |input| tuple((nc::le_f32, nc::le_f32))(input);
        let texture_set_parser =
            |input| nom::multi::count(tex_coord_parser, vertices_count as usize)(input);
        let (input, texture_sets) =
            nom::multi::count(texture_set_parser, texture_set_count as usize)(input)?;

        let (input, triangles) =
            nom::multi::count(Triangle::parse, triangle_count as usize)(input)?;

        Ok((
            input,
            GeometryData {
                prelit_color,
                texture_sets,
                triangles,
            },
        ))
    }
}
