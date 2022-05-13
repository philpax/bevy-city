use nom::{bytes::complete as bc, number::complete as nc, sequence::tuple, Finish, IResult};
use std::{
    fmt::{Debug, Display},
    fs::File,
    io,
    path::Path,
};
use thiserror::Error;

mod constants;
use constants::*;

#[derive(PartialEq, Eq)]
pub struct UnparsedData(Vec<u8>);
impl UnparsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for byte in &self.0 {
            write!(f, "{:0>2x}", byte)?;
        }
        Ok(())
    }
}

impl Display for UnparsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Debug for UnparsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("parsing error")]
    NomError(#[from] nom::error::Error<UnparsedData>),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, PartialEq)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}
impl Vec3 {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z)) = tuple((nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((input, Vec3 { x, y, z }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Mat3([Vec3; 3]);
impl Mat3 {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r0, r1, r2)) = tuple((Vec3::parse, Vec3::parse, Vec3::parse))(input)?;
        Ok((input, Mat3([r0, r1, r2])))
    }
}

#[derive(Debug, PartialEq)]
pub struct Frame {
    rotation: Mat3,
    position: Vec3,
}
impl Frame {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, rotation) = Mat3::parse(input)?;
        let (input, position) = Vec3::parse(input)?;
        let (input, _current_frame_index) = nc::le_u32(input)?;
        let (input, _matrix_creation_flags) = nc::le_u32(input)?;
        Ok((input, Frame { rotation, position }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Lighting {
    ambient: f32,
    specular: f32,
    diffuse: f32,
}
impl Lighting {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
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

#[derive(Debug, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl Color {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (r, g, b, a)) = tuple((nc::u8, nc::u8, nc::u8, nc::u8))(input)?;
        Ok((input, Color { r, g, b, a }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Triangle {
    vertex1: u16,
    vertex2: u16,
    vertex3: u16,
    material_id: u16,
}
impl Triangle {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (vertex2, vertex1, material_id, vertex3)) =
            tuple((nc::le_u16, nc::le_u16, nc::le_u16, nc::le_u16))(input)?;
        Ok((
            input,
            Triangle {
                vertex1,
                vertex2,
                vertex3,
                material_id,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Sphere {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
}
impl Sphere {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (x, y, z, radius)) =
            tuple((nc::le_f32, nc::le_f32, nc::le_f32, nc::le_f32))(input)?;
        Ok((input, Sphere { x, y, z, radius }))
    }
}

#[derive(Debug, PartialEq)]
pub struct GeometryData {
    prelit_color: Option<Vec<Color>>,
    texture_sets: Vec<Vec<(f32, f32)>>,
    triangles: Vec<Triangle>,
}
impl GeometryData {
    fn parse(
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

#[derive(Debug, PartialEq)]
pub struct MorphTarget {
    bounding_sphere: Sphere,
    vertices: Option<Vec<Vec3>>,
    normals: Option<Vec<Vec3>>,
}
impl MorphTarget {
    fn parse(input: &[u8], vertices_count: u32) -> IResult<&[u8], Self> {
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

#[derive(Debug, PartialEq)]
pub enum SectionData {
    Struct(UnparsedData),
    String(String),
    Texture {
        filtering: TextureFiltering,
        u: TextureAddressing,
        v: TextureAddressing,
        mipmaps_used: bool,
    },
    Material {
        color: Color,
        is_textured: bool,
        lighting: Option<Lighting>,
    },
    MaterialList {
        material_indices: Vec<i32>,
    },
    FrameList(Vec<Frame>),
    Geometry {
        format: constants::GeometryFormat,
        lighting: Option<Lighting>,
        geometry_data: Option<GeometryData>,
        morph_targets: Vec<MorphTarget>,
    },
    Clump {
        atomic_count: u32,
        light_count: u32,
        camera_count: u32,
    },
    Atomic {
        frame_index: u32,
        geometry_index: u32,
        // Render if in view frustum
        render: bool,
    },
    TextureDictionary {
        texture_count: u32,
        device_id: Option<u16>,
    },
    GeometryList {
        geometry_count: u32,
    },
    NodeName(String),
    Unknown,
}
impl SectionData {
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
            SectionData::Texture {
                filtering,
                u,
                v,
                mipmaps_used,
            },
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
            SectionData::Material {
                color,
                is_textured,
                lighting,
            },
        ))
    }

    fn parse_material_list(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, material_count) = nc::le_u32(input)?;
        let (input, material_indices) =
            nom::multi::count(nc::le_i32, material_count as usize)(input)?;
        Ok((input, SectionData::MaterialList { material_indices }))
    }

    fn parse_frame_list(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, frame_count) = nc::le_u32(input)?;
        let (input, frames) = nom::multi::count(Frame::parse, frame_count as usize)(input)?;
        Ok((input, SectionData::FrameList(frames)))
    }

    fn parse_geometry_data(input: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (input, (format, triangle_count, vertices_count, morph_target_count)) =
            tuple((nc::le_u32, nc::le_u32, nc::le_u32, nc::le_u32))(input)?;

        let texture_set_count = ((format & 0x00FF0000) >> 16) as u16;
        let format = GeometryFormat::from_bits(format & 0x0000_FFFF).unwrap();

        let (input, lighting) = nom::combinator::cond(version < 0x3_4000, Lighting::parse)(input)?;
        let (input, geometry_data) =
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
            SectionData::Geometry {
                format,
                lighting,
                geometry_data,
                morph_targets,
            },
        ))
    }

    fn parse_clump_data(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (atomic_count, light_count, camera_count)) =
            tuple((nc::le_u32, nc::le_u32, nc::le_u32))(input)?;

        Ok((
            input,
            SectionData::Clump {
                atomic_count,
                light_count,
                camera_count,
            },
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
            SectionData::Atomic {
                frame_index,
                geometry_index,
                render,
            },
        ))
    }

    fn parse_texture_dictionary(input: &[u8], version: u32) -> IResult<&[u8], Self> {
        if version < 0x3_6000 {
            let (input, texture_count) = nc::le_u32(input)?;
            Ok((
                input,
                SectionData::TextureDictionary {
                    texture_count,
                    device_id: None,
                },
            ))
        } else {
            let (input, (texture_count, device_id)) = tuple((nc::le_u16, nc::le_u16))(input)?;
            Ok((
                input,
                SectionData::TextureDictionary {
                    texture_count: texture_count as u32,
                    device_id: Some(device_id),
                },
            ))
        }
    }

    fn parse_geometry_list(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, geometry_count) = nc::le_u32(input)?;
        Ok((input, SectionData::GeometryList { geometry_count }))
    }

    fn parse_struct(input: &[u8], parent_type: SectionType, version: u32) -> IResult<&[u8], Self> {
        Ok(match parent_type {
            SectionType::Texture => Self::parse_texture(input)?,
            SectionType::Material => Self::parse_material(input, version)?,
            SectionType::MaterialList => Self::parse_material_list(input)?,
            SectionType::FrameList => Self::parse_frame_list(input)?,
            SectionType::Geometry => Self::parse_geometry_data(input, version)?,
            SectionType::Clump => Self::parse_clump_data(input)?,
            SectionType::Atomic => Self::parse_atomic_data(input)?,
            SectionType::TextureDictionary => Self::parse_texture_dictionary(input, version)?,
            SectionType::GeometryList => Self::parse_geometry_list(input)?,
            _ => (&[], SectionData::Struct(UnparsedData(input.to_vec()))),
        })
    }

    fn parse_string(input: &[u8]) -> IResult<&[u8], Self> {
        Ok((
            &[],
            SectionData::String(
                String::from_utf8_lossy(input)
                    .trim_end_matches(char::from(0))
                    .to_string(),
            ),
        ))
    }

    fn parse_node_name(input: &[u8]) -> IResult<&[u8], Self> {
        Ok((
            &[],
            SectionData::NodeName(String::from_utf8_lossy(input).to_string()),
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub section_type: SectionType,
    pub version: u32,
    pub children: Vec<Section>,
    pub data: SectionData,
}

impl Section {
    fn parse(input: &[u8], parent_type: Option<SectionType>) -> IResult<&[u8], Section> {
        let (input, section_type) = nc::le_u32(input)?;
        let section_type = num_traits::FromPrimitive::from_u32(section_type)
            .expect(&format!("unexpected section type {:X}", section_type));

        let (input, section_size) = nc::le_u32(input)?;
        let (input, version) = nc::le_u32(input)?;
        let version = {
            if version & 0xFFFF0000 != 0 {
                (version >> 14 & 0x3FF00) + 0x30000 | (version >> 16 & 0x3F)
            } else {
                version << 8
            }
        };
        let (input, data) = bc::take(section_size)(input)?;

        let (mut data, section_data) = match section_type {
            SectionType::Struct => SectionData::parse_struct(data, parent_type.unwrap(), version)?,
            SectionType::String => SectionData::parse_string(data)?,
            SectionType::NodeName => SectionData::parse_node_name(data)?,
            SectionType::Clump
            | SectionType::GeometryList
            | SectionType::FrameList
            | SectionType::MaterialList
            | SectionType::Extension
            | SectionType::Material
            | SectionType::Texture
            | SectionType::Geometry
            | SectionType::Atomic
            | SectionType::Raster
            | SectionType::TextureDictionary => (data, SectionData::Unknown),
            _ => (&[] as &[u8], SectionData::Unknown),
        };

        let mut children = vec![];
        while !data.is_empty() {
            let section: Section;
            (data, section) = Section::parse(data, Some(section_type))?;
            children.push(section);
        }

        Ok((
            input,
            Section {
                section_type,
                version,
                children,
                data: section_data,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct BinaryStreamFile {
    pub sections: Vec<Section>,
}

impl BinaryStreamFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_reader(io::BufReader::new(File::open(path)?))
    }

    pub fn from_reader<R: io::Read + io::Seek + Send + 'static>(
        mut reader: R,
    ) -> Result<Self, Error> {
        // We only support one section for now
        let mut data = vec![];
        reader.read_to_end(&mut data)?;

        let (_, section) = Section::parse(&data, None)
            .map_err(|err| err.map_input(|i| UnparsedData(i.to_owned())))
            .finish()?;

        Ok(BinaryStreamFile {
            sections: vec![section],
        })
    }
}
