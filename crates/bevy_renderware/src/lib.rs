use bevy_app::prelude::*;
use bevy_asset::{AddAsset, AssetLoader, LoadContext, LoadedAsset};
use bevy_render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use bevy_utils::BoxedFuture;

use anyhow::Result;
use thiserror::Error;

use renderware_format as rwf;

#[derive(Default)]
pub struct DffLoader;

impl AssetLoader for DffLoader {
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

async fn load_dff<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), RwError> {
    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let model = rwf::dff::Model::from_raw(&raw).swap_remove(0);
    let vertices = model.vertices;

    let mut mesh = Mesh::new(match model.topology {
        rwf::dff::Topology::TriangleList => PrimitiveTopology::TriangleList,
        rwf::dff::Topology::TriangleStrip => PrimitiveTopology::TriangleStrip,
    });
    set_position_data(
        &mut mesh,
        vertices
            .iter()
            .map(|v| v.position)
                .map(|[x, y, z]| [x, z, -y])
            .collect(),
    );
    set_normal_data(&mut mesh, vertices.iter().map(|v| v.normal).collect());
    set_uv_data(&mut mesh, vertices.iter().map(|v| v.uv).collect());
    mesh.set_indices(Some(Indices::U16(model.indices.clone())));

    load_context.set_default_asset(LoadedAsset::new(mesh));
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

/// Adds support for Rw file loading to Apps
#[derive(Default)]
pub struct RwPlugin;
impl Plugin for RwPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<DffLoader>();
    }
}
