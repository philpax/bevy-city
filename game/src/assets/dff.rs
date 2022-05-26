use std::path::Path;

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
use rwf::dff::Vec3;

#[derive(Default)]
pub struct DffLoader;

impl AssetLoader for DffLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move { Ok(load_dff(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dff"];
        EXTENSIONS
    }
}

pub struct Model {
    pub name: String,
    pub mesh: Mesh,
    pub materials: Vec<rwf::dff::Material>,
}

#[derive(TypeUuid)]
#[uuid = "7f24d251-ce34-4078-85b8-a8f99fc790db"]
pub struct Dff {
    pub models: Vec<Model>,
}

async fn load_dff<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> anyhow::Result<()> {
    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let models = rwf::dff::Model::from_raw(&raw);

    load_context.set_default_asset(LoadedAsset::new(Dff {
        models: models
            .iter()
            .map(|(_, model)| rwf_model_to_bevy_model(model, load_context.path()))
            .collect(),
    }));

    Ok(())
}

fn rwf_model_to_bevy_model(model: &rwf::dff::Model, path: &Path) -> Model {
    let mut mesh = Mesh::new(match model.topology {
        rwf::dff::Topology::TriangleList => PrimitiveTopology::TriangleList,
        rwf::dff::Topology::TriangleStrip => PrimitiveTopology::TriangleStrip,
    });

    let vertices = &model.vertices;
    set_position_data(
        &mut mesh,
        vertices
            .iter()
            .map(|v| v.position)
            .map(|Vec3 { x, y, z }| [x, z, -y])
            .collect(),
    );
    set_normal_data(
        &mut mesh,
        vertices.iter().map(|v| v.normal.as_array()).collect(),
    );
    set_uv_data(&mut mesh, vertices.iter().map(|v| v.uv).collect());
    mesh.set_indices(Some(Indices::U16(
        model
            .triangles
            .iter()
            .flat_map(|t| [t.vertex1, t.vertex2, t.vertex3])
            .collect(),
    )));

    let name = path
        .file_stem()
        .expect("failed to extract filestem")
        .to_string_lossy()
        .to_string();

    let materials = model.materials.clone();
    Model {
        name,
        mesh,
        materials,
    }
}

fn set_position_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data);
}

fn set_normal_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data);
}

fn set_uv_data(mesh: &mut Mesh, data: Vec<[f32; 2]>) {
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data);
}

#[derive(Default)]
pub struct DffPlugin;
impl Plugin for DffPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Dff>().init_asset_loader::<DffLoader>();
    }
}
