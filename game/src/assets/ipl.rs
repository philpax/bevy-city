use bevy::{
    app::prelude::*,
    asset::{AddAsset, AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

use vice_city_formats as vcf;

#[derive(Debug, TypeUuid, PartialEq)]
#[uuid = "eef31d55-f995-4073-87a0-3c50e7fabef7"]
pub struct Ipl(pub vcf::Ipl);

#[derive(Default)]
pub struct IplLoader;

impl AssetLoader for IplLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let value = vcf::Ipl::parse(std::str::from_utf8(bytes).unwrap());
            load_context.set_default_asset(LoadedAsset::new(Ipl(value)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ipl"];
        EXTENSIONS
    }
}

/// Adds support for Ipl file loading to Apps
#[derive(Default)]
pub struct IplPlugin;
impl Plugin for IplPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Ipl>().init_asset_loader::<IplLoader>();
    }
}
