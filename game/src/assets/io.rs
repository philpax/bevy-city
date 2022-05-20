use bevy::{
    asset::{AssetIo, AssetIoError},
    prelude::*,
    utils::BoxedFuture,
};
use std::path::{Path, PathBuf};

struct ImgAssetIo {
    base: Box<dyn AssetIo>,
}

impl ImgAssetIo {
    fn new(base: Box<dyn AssetIo>) -> Self {
        Self { base }
    }
}

impl AssetIo for ImgAssetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        if path.parent() == Some(&Path::new("models/gta3")) {}
        self.base.load_path(path)
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        self.base.read_directory(path)
    }

    fn is_directory(&self, path: &Path) -> bool {
        self.base.is_directory(path)
    }

    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
        self.base.watch_path_for_changes(path)
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        self.base.watch_for_changes()
    }
}

pub struct ImgIoPlugin;

impl Plugin for ImgIoPlugin {
    fn build(&self, app: &mut App) {
        let task_pool = app.world.resource::<bevy::tasks::IoTaskPool>().0.clone();
        let asset_io = ImgAssetIo::new(bevy::asset::create_platform_default_asset_io(app));

        app.insert_resource(AssetServer::new(asset_io, task_pool));
    }
}
