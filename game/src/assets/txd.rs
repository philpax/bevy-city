use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
    utils::BoxedFuture,
};

use renderware_format as rwf;

#[derive(Default)]
pub struct TxdLoader;

impl AssetLoader for TxdLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move { Ok(load_txd(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["txd"];
        EXTENSIONS
    }
}

async fn load_txd<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> anyhow::Result<()> {
    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let textures = rwf::txd::Texture::from_raw(&raw);

    for texture in textures {
        let mut image = Image::new(
            Extent3d {
                width: texture.width as _,
                height: texture.height as _,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture.data.clone(),
            TextureFormat::Rgba8Unorm,
        );

        let remap_addressing =
            |addressing: rwf::txd::TextureAddressing| -> bevy::render::render_resource::AddressMode {
                use bevy::render::render_resource::AddressMode;
                use rwf::txd::TextureAddressing;

                match addressing {
                    TextureAddressing::NoTiling => AddressMode::Repeat,
                    TextureAddressing::Wrap => AddressMode::Repeat,
                    TextureAddressing::Mirror => AddressMode::MirrorRepeat,
                    TextureAddressing::Clamp => AddressMode::ClampToEdge,
                    TextureAddressing::Border => AddressMode::ClampToBorder,
                }
            };

        image.sampler_descriptor.address_mode_u = remap_addressing(texture.uv.0);
        image.sampler_descriptor.address_mode_v = remap_addressing(texture.uv.1);
        load_context.set_labeled_asset(&texture.name, LoadedAsset::new(image));
    }

    Ok(())
}

#[derive(Default)]
pub struct TxdPlugin;
impl Plugin for TxdPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<TxdLoader>();
    }
}
