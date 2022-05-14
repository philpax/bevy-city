use nom::{number::complete as nc, sequence::tuple, IResult};

use super::{Sphere, Vec3};

#[derive(Debug, PartialEq)]
pub struct MorphTarget {
    bounding_sphere: Sphere,
    vertices: Option<Vec<Vec3>>,
    normals: Option<Vec<Vec3>>,
}
impl MorphTarget {
    pub(crate) fn parse(input: &[u8], vertices_count: u32) -> IResult<&[u8], Self> {
        let (input, bounding_sphere) = Sphere::parse(input)?;
        let (input, (has_vertices, has_normals)) = tuple((nc::le_u32, nc::le_u32))(input)?;
        let (input, vertices) = nom::combinator::cond(
            has_vertices > 0,
            nom::multi::count(Vec3::parse, vertices_count as usize),
        )(input)?;
        let (input, normals) = nom::combinator::cond(
            has_normals > 0,
            nom::multi::count(Vec3::parse, vertices_count as usize),
        )(input)?;

        Ok((
            input,
            MorphTarget {
                bounding_sphere,
                vertices,
                normals,
            },
        ))
    }
}
