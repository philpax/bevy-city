use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};

use renderware_format as rwf;
pub use rwf::txd::Texture;

#[derive(Default)]
pub struct TxdLoader;

impl AssetLoader for TxdLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move { load_txd(bytes, load_context).await })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["txd"];
        EXTENSIONS
    }
}

#[derive(TypeUuid)]
#[uuid = "50233c25-f751-48e4-b805-90b597cdc9d8"]
pub struct Txd {
    pub textures: Vec<Texture>,
}

async fn load_txd<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> anyhow::Result<()> {
    let raw = rwf::raw::BinaryStreamFile::from_bytes(bytes)?;
    let textures = rwf::txd::Texture::from_raw(&raw);
    load_context.set_default_asset(LoadedAsset::new(Txd { textures }));

    Ok(())
}

#[derive(Default)]
pub struct TxdPlugin;
impl Plugin for TxdPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Txd>().init_asset_loader::<TxdLoader>();
    }
}
