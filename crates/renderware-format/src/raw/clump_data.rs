use nom::{
    bits::complete as bic, combinator::cond, number::complete as nc, sequence::tuple, IResult,
};
use num_traits::FromPrimitive;

use super::{constants::*, Color, Frame, GeometryData, Lighting, MorphTarget, UnparsedData};

#[derive(Debug, PartialEq)]
pub struct Texture {
    pub filtering: TextureFiltering,
    pub uv: (TextureAddressing, TextureAddressing),
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
pub struct Raster {
    pub filtering: TextureFiltering,
    pub uv: (TextureAddressing, TextureAddressing),

    pub name: String,
    pub mask_name: String,

    pub raster_format: RasterFormat,
    pub has_alpha: bool,

    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub level_count: u8,
    pub raster_type: u8,
    pub compression: u8,

    pub data: Vec<u8>,
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
    Raster(Raster),
    TextureDictionary(TextureDictionary),
    GeometryList { geometry_count: u32 },
    NodeName(String),
    Unknown,
}
impl ClumpData {
    fn parse_texture(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, filtering) =
            nom::combinator::map(nc::le_u8, |v| FromPrimitive::from_u8(v).unwrap())(input)?;

        let (input, (u, v, mipmaps_used, _padding)): (_, (_, _, u8, u16)) =
            nom::bits::bits::<_, _, nom::error::Error<(&[u8], usize)>, _, _>(tuple((
                bic::take(4usize),
                bic::take(4usize),
                bic::take(1usize),
                bic::take(15usize),
            )))(input)?;

        let uv = (
            FromPrimitive::from_u8(u).unwrap(),
            FromPrimitive::from_u8(v).unwrap(),
        );
        let mipmaps_used = mipmaps_used > 0;

        Ok((
            input,
            ClumpData::Texture(Texture {
                filtering,
                uv,
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

    fn parse_geometry(input: &[u8], version: u32) -> IResult<&[u8], Self> {
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

    fn parse_clump(input: &[u8], version: u32) -> IResult<&[u8], Self> {
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

    fn parse_atomic(input: &[u8]) -> IResult<&[u8], Self> {
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

    fn parse_raster(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, platform_id) = nc::le_u32(input)?;
        // We don't support anything else right now
        assert_eq!(platform_id, 8);

        let (input, (filtering, u, v, _padding)): (_, (_, _, u8, u16)) =
            nom::bits::bits::<_, _, nom::error::Error<(&[u8], usize)>, _, _>(tuple((
                bic::take(8usize),
                bic::take(4usize),
                bic::take(4usize),
                bic::take(16usize),
            )))(input)?;
        let filtering = FromPrimitive::from_u8(filtering).unwrap();
        let uv = (
            FromPrimitive::from_u8(u).unwrap(),
            FromPrimitive::from_u8(v).unwrap(),
        );

        let (input, name) = parse_null_terminated_ascii(input, 32)?;
        let (input, mask_name) = parse_null_terminated_ascii(input, 32)?;

        let (input, raster_format) = nc::le_u32(input)?;
        let raster_format = RasterFormat::new(raster_format);

        let (input, has_alpha) = nc::le_u32(input)?;
        let has_alpha = has_alpha > 0;

        let (input, width) = nc::le_u16(input)?;
        let (input, height) = nc::le_u16(input)?;
        let (input, depth) = nc::le_u8(input)?;
        let (input, level_count) = nc::le_u8(input)?;
        let (input, raster_type) = nc::le_u8(input)?;
        let (input, compression) = nc::le_u8(input)?;

        let (input, raster_size) = nc::le_u32(input)?;
        let (input, raster_data) = nom::bytes::complete::take(raster_size)(input)?;

        let data = raster_data.to_vec();
        Ok((
            input,
            ClumpData::Raster(Raster {
                filtering,
                uv,

                name,
                mask_name,

                raster_format,
                has_alpha,

                width,
                height,
                depth,
                level_count,
                raster_type,
                compression,

                data,
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

    pub(crate) const SUPPORTED_TYPES: &'static [SectionType] = &[
        SectionType::Texture,
        SectionType::Material,
        SectionType::MaterialList,
        SectionType::FrameList,
        SectionType::Geometry,
        SectionType::Clump,
        SectionType::Atomic,
        SectionType::Raster,
        SectionType::TextureDictionary,
        SectionType::GeometryList,
    ];
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
            SectionType::Geometry => Self::parse_geometry(input, version)?,
            SectionType::Clump => Self::parse_clump(input, version)?,
            SectionType::Atomic => Self::parse_atomic(input)?,
            SectionType::Raster => Self::parse_raster(input)?,
            SectionType::TextureDictionary => Self::parse_texture_dictionary(input, version)?,
            SectionType::GeometryList => Self::parse_geometry_list(input)?,
            _ => (&[], ClumpData::Struct(super::UnparsedData(input.to_vec()))),
        })
    }

    pub(crate) fn parse_string(input: &[u8]) -> IResult<&[u8], Self> {
        Ok((&[], ClumpData::String(null_terminated_ascii(input))))
    }

    pub(crate) fn parse_node_name(input: &[u8]) -> IResult<&[u8], Self> {
        Ok((
            &[],
            ClumpData::NodeName(String::from_utf8_lossy(input).to_string()),
        ))
    }
}

fn null_terminated_ascii(input: &[u8]) -> String {
    String::from_utf8_lossy(input)
        .trim_end_matches(char::from(0))
        .to_string()
}

fn parse_null_terminated_ascii(input: &[u8], length: usize) -> IResult<&[u8], String> {
    let (input, str) = nom::bytes::complete::take(length)(input)?;
    Ok((input, null_terminated_ascii(str)))
}
