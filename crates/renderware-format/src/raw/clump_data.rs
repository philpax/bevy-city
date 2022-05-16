use nom::{combinator::cond, number::complete as nc, sequence::tuple, IResult};

use super::{constants::*, Color, Frame, GeometryData, Lighting, MorphTarget, UnparsedData};

#[derive(Debug, PartialEq)]
pub struct Texture {
    pub filtering: TextureFiltering,
    pub u: TextureAddressing,
    pub v: TextureAddressing,
    pub mipmaps_used: bool,
}

#[derive(Debug, PartialEq)]
pub struct Material {
    pub color: Color,
    pub is_textured: bool,
    pub lighting: Option<Lighting>,
}

#[derive(Debug, PartialEq)]
pub struct Geometry {
    pub format: GeometryFormat,
    pub lighting: Option<Lighting>,
    pub data: Option<GeometryData>,
    pub morph_targets: Vec<MorphTarget>,
}

#[derive(Debug, PartialEq)]
pub struct Clump {
    pub atomic_count: u32,
    pub light_count: u32,
    pub camera_count: u32,
}

#[derive(Debug, PartialEq)]
pub struct Atomic {
    pub frame_index: u32,
    pub geometry_index: u32,
    // Render if in view frustum
    pub render: bool,
}

#[derive(Debug, PartialEq)]
pub struct TextureDictionary {
    pub texture_count: u32,
    pub device_id: Option<u16>,
}

#[derive(Debug, PartialEq)]
pub enum ClumpData {
    Struct(UnparsedData),
    String(String),
    Texture(Texture),
    Material(Material),
    MaterialList { material_indices: Vec<i32> },
    FrameList(Vec<Frame>),
    Geometry(Geometry),
    Clump(Clump),
    Atomic(Atomic),
    TextureDictionary(TextureDictionary),
    GeometryList { geometry_count: u32 },
    NodeName(String),
    Unknown,
}
impl ClumpData {
    fn parse_texture(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, filtering) = nom::combinator::map(nc::le_u8, |v| {
            num_traits::FromPrimitive::from_u8(v).unwrap()
        })(input)?;

        let (input, (u, v, mipmaps_used, _padding)): (_, (_, _, u8, u16)) =
            nom::bits::bits::<_, _, nom::error::Error<(&[u8], usize)>, _, _>(tuple((
                nom::bits::complete::take(4usize),
                nom::bits::complete::take(4usize),
                nom::bits::complete::take(1usize),
                nom::bits::complete::take(15usize),
            )))(input)?;

        let u = num_traits::FromPrimitive::from_u8(u).unwrap();
        let v = num_traits::FromPrimitive::from_u8(v).unwrap();
        let mipmaps_used = mipmaps_used > 0;

        Ok((
            input,
            ClumpData::Texture(Texture {
                filtering,
                u,
                v,
                mipmaps_used,
            }),
        ))
    }

    fn parse_material(input: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (input, _flags) = nc::le_u32(input)?;
        let (input, color) = Color::parse(input)?;
        let (input, _unused) = nc::le_u32(input)?;
        let (input, is_textured) = nom::combinator::map(nc::le_u32, |v| v > 0)(input)?;
        let (input, lighting) = nom::combinator::cond(version > 0x3_0400, Lighting::parse)(input)?;
        Ok((
            input,
            ClumpData::Material(Material {
                color,
                is_textured,
                lighting,
            }),
        ))
    }

    fn parse_material_list(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, material_count) = nc::le_u32(input)?;
        let (input, material_indices) =
            nom::multi::count(nc::le_i32, material_count as usize)(input)?;
        Ok((input, ClumpData::MaterialList { material_indices }))
    }

    fn parse_frame_list(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, frame_count) = nc::le_u32(input)?;
        let (input, frames) = nom::multi::count(Frame::parse, frame_count as usize)(input)?;
        Ok((input, ClumpData::FrameList(frames)))
    }

    fn parse_geometry_data(input: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (input, (format, triangle_count, vertices_count, morph_target_count)) =
            tuple((nc::le_u32, nc::le_u32, nc::le_u32, nc::le_u32))(input)?;

        let texture_set_count = ((format & 0x00FF0000) >> 16) as u16;
        let format = GeometryFormat::from_bits(format & 0x0000_FFFF).unwrap();

        let (input, lighting) = nom::combinator::cond(version < 0x3_4000, Lighting::parse)(input)?;
        let (input, data) =
            nom::combinator::cond(!format.contains(GeometryFormat::NATIVE), |input| {
                GeometryData::parse(
                    input,
                    format.contains(GeometryFormat::PRELIT),
                    texture_set_count,
                    vertices_count,
                    triangle_count,
                )
            })(input)?;

        let (input, morph_targets) = nom::multi::count(
            |data| MorphTarget::parse(data, vertices_count),
            morph_target_count as usize,
        )(input)?;

        Ok((
            input,
            ClumpData::Geometry(Geometry {
                format,
                lighting,
                data,
                morph_targets,
            }),
        ))
    }

    fn parse_clump_data(input: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (input, atomic_count) = nc::le_u32(input)?;
        let (input, extra_counts) = cond(
            version > 0x3_3000 && input.len() > 4,
            tuple((nc::le_u32, nc::le_u32)),
        )(input)?;
        let (light_count, camera_count) = extra_counts.unwrap_or_default();

        Ok((
            input,
            ClumpData::Clump(Clump {
                atomic_count,
                light_count,
                camera_count,
            }),
        ))
    }

    fn parse_atomic_data(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (frame_index, geometry_index, flags, _unused)) =
            tuple((nc::le_u32, nc::le_u32, nc::le_u32, nc::le_u32))(input)?;

        // 0x01 rpATOMICCOLLISIONTEST - A generic collision flag to indicate that the atomic should be considered in collision tests.
        // It wasn't used in GTA games since they don't use RW collision system.
        // 0x04 rpATOMICRENDER        - The atomic is rendered if it is in the view frustum. It's set to TRUE for all models by default.
        let render = flags & 0x04 > 0;

        Ok((
            input,
            ClumpData::Atomic(Atomic {
                frame_index,
                geometry_index,
                render,
            }),
        ))
    }

    fn parse_texture_dictionary(input: &[u8], version: u32) -> IResult<&[u8], Self> {
        if version < 0x3_6000 {
            let (input, texture_count) = nc::le_u32(input)?;
            Ok((
                input,
                ClumpData::TextureDictionary(TextureDictionary {
                    texture_count,
                    device_id: None,
                }),
            ))
        } else {
            let (input, (texture_count, device_id)) = tuple((nc::le_u16, nc::le_u16))(input)?;
            Ok((
                input,
                ClumpData::TextureDictionary(TextureDictionary {
                    texture_count: texture_count as u32,
                    device_id: Some(device_id),
                }),
            ))
        }
    }

    fn parse_geometry_list(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, geometry_count) = nc::le_u32(input)?;
        Ok((input, ClumpData::GeometryList { geometry_count }))
    }

    pub(crate) fn parse_struct(
        input: &[u8],
        parent_type: SectionType,
        version: u32,
    ) -> IResult<&[u8], Self> {
        Ok(match parent_type {
            SectionType::Texture => Self::parse_texture(input)?,
            SectionType::Material => Self::parse_material(input, version)?,
            SectionType::MaterialList => Self::parse_material_list(input)?,
            SectionType::FrameList => Self::parse_frame_list(input)?,
            SectionType::Geometry => Self::parse_geometry_data(input, version)?,
            SectionType::Clump => Self::parse_clump_data(input, version)?,
            SectionType::Atomic => Self::parse_atomic_data(input)?,
            SectionType::TextureDictionary => Self::parse_texture_dictionary(input, version)?,
            SectionType::GeometryList => Self::parse_geometry_list(input)?,
            _ => (&[], ClumpData::Struct(super::UnparsedData(input.to_vec()))),
        })
    }

    pub(crate) fn parse_string(input: &[u8]) -> IResult<&[u8], Self> {
        Ok((
            &[],
            ClumpData::String(
                String::from_utf8_lossy(input)
                    .trim_end_matches(char::from(0))
                    .to_string(),
            ),
        ))
    }

    pub(crate) fn parse_node_name(input: &[u8]) -> IResult<&[u8], Self> {
        Ok((
            &[],
            ClumpData::NodeName(String::from_utf8_lossy(input).to_string()),
        ))
    }
}
