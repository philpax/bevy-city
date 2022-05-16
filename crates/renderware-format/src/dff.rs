use crate::raw::{
    constants::{GeometryFormat, SectionType},
    Atomic, BinaryStreamFile, Frame, Section, SectionData,
};

pub use crate::raw::{Mat3, Vec3};

#[derive(Debug)]
pub struct Transform {
    pub rotation: Mat3,
    pub translation: Vec3,
}
impl From<&Frame> for Transform {
    fn from(frame: &Frame) -> Self {
        Self {
            rotation: frame.rotation,
            translation: frame.translation,
        }
    }
}

#[derive(Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
}
impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

#[derive(Debug)]
pub enum Topology {
    TriangleList,
    TriangleStrip,
}

#[derive(Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub topology: Topology,
}
impl Model {
    fn from_geometry(geometry: &crate::raw::Geometry) -> Model {
        let geometry_data = geometry.data.as_ref().expect("no geometry data");
        let morph_target = &geometry.morph_targets[0];

        let texture_set = if geometry_data.texture_sets.is_empty() {
            vec![[0.0, 0.0]; morph_target.vertices.len()]
        } else {
            geometry_data.texture_sets[0]
                .iter()
                .map(|(u, v)| [*u, *v])
                .collect()
        };

        let normals = if morph_target.normals.is_empty() {
            vec![Vec3::ZERO; morph_target.vertices.len()]
        } else {
            morph_target.normals.clone()
        };

        let vertices: Vec<Vertex> = morph_target
            .vertices
            .iter()
            .zip(normals.iter())
            .zip(texture_set.iter())
            .map(|((position, normal), uv)| Vertex::new(*position, *normal, *uv))
            .collect();

        let indices: Vec<u16> = geometry_data
            .triangles
            .iter()
            .flat_map(|t| [t.vertex1, t.vertex2, t.vertex3])
            .collect();

        let topology = if geometry.format.contains(GeometryFormat::TRI_STRIP) {
            // Topology::TriangleStrip
            // hack(philpax): for some reason, we only get correct rendering with triangle list
            // investigate properly at some point
            Topology::TriangleList
        } else {
            Topology::TriangleList
        };

        Model {
            vertices,
            indices,
            topology,
        }
    }
}

impl Model {
    // todo: replace all the panics with errors
    pub fn from_raw(raw: &BinaryStreamFile) -> Vec<(Transform, Model)> {
        let clump = &raw.sections[0];

        let atomics: Vec<_> = clump
            .find_children_by_type(SectionType::Atomic)
            .filter_map(|s| match s.children[0].data {
                SectionData::Atomic(Atomic {
                    frame_index,
                    geometry_index,
                    render,
                }) if render => Some((frame_index as usize, geometry_index as usize)),
                _ => None,
            })
            .collect();

        let frames = match &clump
            .find_child_by_type(SectionType::FrameList)
            .and_then(Section::get_child_struct_data)
        {
            Some(SectionData::FrameList(v)) => &v[..],
            _ => panic!("no frame list data"),
        };

        let geometry_list = &clump
            .find_child_by_type(SectionType::GeometryList)
            .expect("failed to find geometry list");

        let geometry_count = match geometry_list.get_child_struct_data() {
            Some(SectionData::GeometryList { geometry_count }) => *geometry_count,
            _ => panic!("no geometry list struct"),
        };

        let geometries: Vec<_> = geometry_list
            .find_children_by_type(SectionType::Geometry)
            .map(|s| match s.get_child_struct_data() {
                Some(SectionData::Geometry(geometry)) => geometry,
                _ => panic!("no geometry data"),
            })
            .collect();
        assert_eq!(geometry_count as usize, geometries.len());

        atomics
            .into_iter()
            .map(|(frame_index, geometry_index)| (&frames[frame_index], geometries[geometry_index]))
            .map(|(frame, geometry)| (frame.into(), Model::from_geometry(geometry)))
            .collect()
    }
}
