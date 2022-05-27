use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
    utils::BoxedFuture,
};

use renderware_format as rwf;

#[derive(Default)]
pub struct DffLoader;

impl AssetLoader for DffLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move { load_dff(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dff"];
        EXTENSIONS
    }
}

pub struct Model {
    pub mesh: Mesh,
    pub transform: Transform,
    pub materials: Vec<rwf::dff::Material>,
    pub material_indices: Vec<usize>,
}

#[derive(TypeUuid)]
#[uuid = "7f24d251-ce34-4078-85b8-a8f99fc790db"]
pub struct Dff {
    pub name: String,
    pub models: Vec<Model>,
}

async fn load_dff<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> anyhow::Result<()> {
    let name = load_context
        .path()
        .file_stem()
        .expect("failed to extract filestem")
        .to_string_lossy()
        .to_string();

    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let models = rwf::dff::Model::from_raw(&raw)
        .into_iter()
        .map(rwf_model_to_bevy_model)
        .collect();

    load_context.set_default_asset(LoadedAsset::new(Dff { name, models }));

    Ok(())
}

fn rwf_model_to_bevy_model((transform, model): (rwf::dff::Transform, rwf::dff::Model)) -> Model {
    let mesh = {
        let mut mesh = Mesh::new(match model.topology {
            rwf::dff::Topology::TriangleList => PrimitiveTopology::TriangleList,
            rwf::dff::Topology::TriangleStrip => PrimitiveTopology::TriangleStrip,
        });

        let mut positions = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        let mut material_ids = vec![];
        for vertex in &model.vertices {
            let rwf::dff::Vec3 { x, y, z } = vertex.position;
            positions.push([x, z, -y]);
            normals.push(vertex.normal.as_array());
            uvs.push(vertex.uv);
            material_ids.push(vertex.material_id as u32);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(crate::render::ATTRIBUTE_MATERIAL_ID, material_ids);
        mesh.set_indices(Some(Indices::U16(model.indices)));
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
        mesh
    };
    let transform = Transform {
        translation: transform.translation.as_array().into(),
        rotation: Quat::from_mat3(&Mat3::from_cols_array(&transform.rotation.0)),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let materials = model.materials;
    let material_indices = model.material_indices;
    Model {
        mesh,
        transform,
        materials,
        material_indices,
    }
}

#[derive(Default)]
pub struct DffPlugin;
impl Plugin for DffPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Dff>().init_asset_loader::<DffLoader>();
    }
}
