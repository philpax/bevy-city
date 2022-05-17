use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_reflect::TypeUuid;
use bevy_render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use bevy_utils::BoxedFuture;

use anyhow::Result;
use thiserror::Error;

use renderware_format as rwf;
use rwf::dff::Vec3;

#[derive(Default)]
pub struct Loader;

impl AssetLoader for Loader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move { Ok(load_dff(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dff"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum RwError {
    #[error("Invalid RW file: {0}")]
    Rw(#[from] rwf::raw::Error),
    #[error("Unknown vertex format")]
    UnknownVertexFormat,
}

#[derive(TypeUuid)]
#[uuid = "7f24d251-ce34-4078-85b8-a8f99fc790db"]
pub struct Dff {
    pub name: String,
    pub mesh: Mesh,
    pub materials: Vec<rwf::dff::Material>,
}

async fn load_dff<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), RwError> {
    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let models = rwf::dff::Model::from_raw(&raw);
    if let Some((_transform, model)) = models
        .iter()
        .max_by(|(_, x), (_, y)| x.indices.len().cmp(&y.indices.len()))
    {
        let vertices = &model.vertices;

        let mut mesh = Mesh::new(match model.topology {
            rwf::dff::Topology::TriangleList => PrimitiveTopology::TriangleList,
            rwf::dff::Topology::TriangleStrip => PrimitiveTopology::TriangleStrip,
        });
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
        mesh.set_indices(Some(Indices::U16(model.indices.clone())));

        let name = load_context
            .path()
            .file_stem()
            .expect("failed to extract filestem")
            .to_string_lossy()
            .to_string();

        let materials = model.materials.clone();
        load_context.set_default_asset(LoadedAsset::new(Dff {
            name,
            mesh,
            materials,
        }));
    }

    Ok(())
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
