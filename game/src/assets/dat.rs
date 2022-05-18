use bevy::{
    app::prelude::*,
    asset::{AddAsset, AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

#[derive(Debug, TypeUuid, PartialEq)]
#[uuid = "95f9b96b-326e-4479-8341-0b45c83ead25"]
pub struct Dat(pub String);

#[derive(Default)]
pub struct DatLoader;

impl AssetLoader for DatLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let value = Dat(std::str::from_utf8(bytes).unwrap().to_owned());
            load_context.set_default_asset(LoadedAsset::new(value));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dat"];
        EXTENSIONS
    }
}

/// Adds support for Dat file loading to Apps
#[derive(Default)]
pub struct DatPlugin;
impl Plugin for DatPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Dat>().init_asset_loader::<DatLoader>();
    }
}
