use bevy::{
    app::prelude::*,
    asset::{AddAsset, AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

pub use vice_city_formats as vcf;

#[derive(Debug, TypeUuid, PartialEq)]
#[uuid = "b80d2ec1-3e87-49eb-8e8e-c24658383203"]
pub struct Ide(pub vcf::Ide);

#[derive(Default)]
pub struct IdeLoader;

impl AssetLoader for IdeLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let value = vcf::Ide::parse(std::str::from_utf8(bytes).unwrap());
            load_context.set_default_asset(LoadedAsset::new(Ide(value)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ide"];
        EXTENSIONS
    }
}

/// Adds support for Ide file loading to Apps
#[derive(Default)]
pub struct IdePlugin;
impl Plugin for IdePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Ide>().init_asset_loader::<IdeLoader>();
    }
}
