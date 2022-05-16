use bevy_app::prelude::*;
use bevy_asset::AddAsset;

pub mod dff;
pub mod txd;

/// Adds support for Rw file loading to Apps
#[derive(Default)]
pub struct RwPlugin;
impl Plugin for RwPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<dff::Dff>()
            .init_asset_loader::<dff::Loader>()
            .init_asset_loader::<txd::Loader>();
    }
}
