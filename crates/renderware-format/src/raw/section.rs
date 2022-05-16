use nom::{bytes::complete as bc, number::complete as nc, IResult};

use super::{constants::SectionType, ClumpData};

#[derive(Debug, PartialEq)]
pub struct Section {
    pub section_type: SectionType,
    pub version: u32,
    pub children: Vec<Section>,
    pub data: ClumpData,
}

impl Section {
    pub(crate) fn parse(input: &[u8], parent_type: Option<SectionType>) -> IResult<&[u8], Section> {
        let (input, section_type) = nc::le_u32(input)?;
        let section_type = num_traits::FromPrimitive::from_u32(section_type)
            .unwrap_or_else(|| panic!("unexpected section type {:X}", section_type));

        let (input, section_size) = nc::le_u32(input)?;
        let (input, version) = nc::le_u32(input)?;
        let version = {
            if version & 0xFFFF0000 != 0 {
                ((version >> 14 & 0x3FF00) + 0x30000) | (version >> 16 & 0x3F)
            } else {
                version << 8
            }
        };
        let (input, data) = bc::take(section_size)(input)?;

        let (mut data, section_data) = match section_type {
            SectionType::Struct => ClumpData::parse_struct(data, parent_type.unwrap(), version)?,
            SectionType::String => ClumpData::parse_string(data)?,
            SectionType::NodeName => ClumpData::parse_node_name(data)?,
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
            | SectionType::TextureDictionary => (data, ClumpData::Unknown),
            _ => (&[] as &[u8], ClumpData::Unknown),
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

    pub fn find_children_by_type(
        &self,
        section_type: SectionType,
    ) -> impl Iterator<Item = &Section> {
        self.children
            .iter()
            .filter(move |s| s.section_type == section_type)
    }

    pub fn find_child_by_type(&self, section_type: SectionType) -> Option<&Section> {
        self.find_children_by_type(section_type).next()
    }

    pub fn get_child_struct_data(&self) -> Option<&ClumpData> {
        Some(&self.find_child_by_type(SectionType::Struct)?.data)
    }
}
