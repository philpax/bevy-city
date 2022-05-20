use crate::raw::{
    constants::SectionType, Atomic, BinaryStreamFile, ClumpData, Frame, GeometryData, Section,
};

pub use crate::raw::{
    constants::{TextureAddressing, TextureFiltering},
    Color, Lighting, Mat3, Triangle, Vec3,
};

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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum Topology {
    TriangleList,
    TriangleStrip,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub filtering: TextureFiltering,
    pub uv: (TextureAddressing, TextureAddressing),
    pub name: String,
    pub alpha_name: String,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub color: Color,
    pub is_textured: bool,
    pub lighting: Option<Lighting>,
    pub texture: Option<Texture>,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    pub topology: Topology,
    pub materials: Vec<Material>,
    pub material_indices: Vec<usize>,
}

impl Model {
    fn from_geometry_split(
        geometry: &crate::raw::Geometry,
        material_list: Option<&crate::raw::Section>,
    ) -> Model {
        let (geometry_data, vertices, topology, materials) =
            extract_mesh_data_from_geometry(geometry, material_list);

        let triangles = geometry_data.triangles.clone();

        let (materials, material_indices) = match materials {
            Some(tup) => tup,
            None => (vec![], vec![]),
        };

        Model {
            vertices,
            triangles,
            topology,
            materials,
            material_indices,
        }
    }

    // todo: replace all the panics with errors
    pub fn from_raw(raw: &BinaryStreamFile) -> Vec<(Transform, Model)> {
        let clump = &raw.sections[0];

        let atomics = clump
            .find_children_by_type(SectionType::Atomic)
            .filter_map(|s| match s.children[0].data {
                ClumpData::Atomic(Atomic {
                    frame_index,
                    geometry_index,
                    render,
                }) if render => Some((frame_index as usize, geometry_index as usize)),
                _ => None,
            });

        let frames = match &clump
            .find_child_by_type(SectionType::FrameList)
            .and_then(Section::get_child_struct_data)
        {
            Some(ClumpData::FrameList(v)) => &v[..],
            _ => panic!("no frame list data"),
        };

        let geometry_list = &clump
            .find_child_by_type(SectionType::GeometryList)
            .expect("failed to find geometry list");

        let geometry_count = match geometry_list.get_child_struct_data() {
            Some(ClumpData::GeometryList { geometry_count }) => *geometry_count,
            _ => panic!("no geometry list struct"),
        };

        let geometries: Vec<_> = geometry_list
            .find_children_by_type(SectionType::Geometry)
            .map(|s| {
                (
                    match s.get_child_struct_data() {
                        Some(ClumpData::Geometry(geometry)) => geometry,
                        _ => panic!("no geometry data"),
                    },
                    s.find_child_by_type(SectionType::MaterialList),
                )
            })
            .collect();
        assert_eq!(geometry_count as usize, geometries.len());

        atomics
            .map(|(frame_index, geometry_index)| (&frames[frame_index], geometries[geometry_index]))
            .map(|(frame, (geometry, material_list))| {
                (
                    frame.into(),
                    Model::from_geometry_split(geometry, material_list),
                )
            })
            .collect()
    }
}

type MeshData<'a> = (
    &'a GeometryData,
    Vec<Vertex>,
    Topology,
    Option<(Vec<Material>, Vec<usize>)>,
);

fn extract_mesh_data_from_geometry<'a>(
    geometry: &'a crate::raw::Geometry,
    material_list: Option<&Section>,
) -> MeshData<'a> {
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

    // HACK(philpax): for some reason, we only get correct rendering with triangle list
    // investigate properly at some point
    // let is_tri_strip = geometry.format.contains(GeometryFormat::TRI_STRIP);
    let is_tri_strip = false;
    let topology = if is_tri_strip {
        Topology::TriangleStrip
    } else {
        Topology::TriangleList
    };

    let materials = material_list.map(|ml| {
        let materials: Vec<_> = ml
            .find_children_by_type(SectionType::Material)
            .filter_map(section_to_material)
            .collect();

        let material_indices = match ml.get_child_struct_data() {
            Some(ClumpData::MaterialList { material_indices }) => {
                let mut present_materials = vec![];
                let mut result = vec![];
                let mut last_index = 0;
                for index in material_indices.iter().copied() {
                    if index == -1 {
                        present_materials.push(last_index);
                        result.push(last_index);
                        last_index += 1;
                    } else {
                        result.push(present_materials[index as usize]);
                    }
                }
                result
            }
            _ => unreachable!("we should always have MaterialList data"),
        };
        (materials, material_indices)
    });

    (geometry_data, vertices, topology, materials)
}

fn section_to_material(material: &Section) -> Option<Material> {
    let material_data = match material.get_child_struct_data()? {
        ClumpData::Material(m) => m,
        _ => return None,
    };

    let texture = material
        .find_child_by_type(SectionType::Texture)
        .and_then(|texture| {
            let texture_data = match texture.get_child_struct_data()? {
                ClumpData::Texture(t) => t,
                _ => return None,
            };

            let names: Vec<&String> = texture
                .find_children_by_type(SectionType::String)
                .filter_map(|s| match &s.data {
                    ClumpData::String(s) => Some(s),
                    _ => None,
                })
                .collect();

            Some(Texture {
                filtering: texture_data.filtering,
                uv: texture_data.uv,
                name: names[0].to_string(),
                alpha_name: names[1].to_string(),
            })
        });

    Some(Material {
        color: material_data.color,
        is_textured: material_data.is_textured,
        lighting: material_data.lighting,
        texture,
    })
}
