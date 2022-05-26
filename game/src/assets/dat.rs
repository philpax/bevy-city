use bevy::{
    app::prelude::*,
    asset::{AddAsset, AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use vice_city_formats::dat::GtaVcDat;

#[derive(Debug, TypeUuid, PartialEq)]
#[uuid = "95f9b96b-326e-4479-8341-0b45c83ead25"]
pub enum Dat {
    GtaVcDat(GtaVcDat),
}

#[derive(Default)]
pub struct DatLoader;

impl AssetLoader for DatLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            if load_context
                .path()
                .file_name()
                .map(|s| s != "gta_vc.dat")
                .unwrap_or(true)
            {
                panic!("unsupported dat `{:?}`!", load_context.path());
            }
            let contents = std::str::from_utf8(bytes).unwrap().to_owned();
            load_context
                .set_default_asset(LoadedAsset::new(Dat::GtaVcDat(GtaVcDat::parse(&contents))));
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
