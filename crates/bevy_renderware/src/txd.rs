use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_render::{
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::Image,
};
use bevy_utils::BoxedFuture;

use anyhow::Result;
use thiserror::Error;

use renderware_format as rwf;

#[derive(Default)]
pub struct Loader;

impl AssetLoader for Loader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move { Ok(load_txd(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["txd"];
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

async fn load_txd<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), RwError> {
    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let textures = rwf::txd::Texture::from_raw(&raw);

    for texture in textures {
        load_context.set_labeled_asset(
            &texture.name,
            LoadedAsset::new(Image::new(
                Extent3d {
                    width: texture.width as _,
                    height: texture.height as _,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                texture.data.clone(),
                TextureFormat::Rgba8Unorm,
            )),
        );
    }

    Ok(())
}
