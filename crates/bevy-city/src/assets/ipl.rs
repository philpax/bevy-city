use bevy::{
    app::prelude::*,
    asset::{AddAsset, AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

#[derive(Debug, TypeUuid, PartialEq)]
#[uuid = "eef31d55-f995-4073-87a0-3c50e7fabef7"]
pub struct Ipl {
    pub instances: Vec<(String, [f32; 3])>,
}

impl Ipl {
    pub fn parse(data: &str) -> Self {
        let sections = super::common::categorise_lines(data);

        let instances: Vec<_> = sections
            .get("inst")
            .expect("no inst")
            .iter()
            .map(|line| {
                let segments: Vec<_> = super::common::split_line(line);
                (
                    segments[1].to_string(),
                    [
                        segments[3].parse::<f32>().unwrap(),
                        segments[5].parse::<f32>().unwrap(),
                        -segments[4].parse::<f32>().unwrap(),
                    ],
                )
            })
            .collect();

        Ipl { instances }
    }
}

#[derive(Default)]
pub struct IplLoader;

impl AssetLoader for IplLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let value = Ipl::parse(std::str::from_utf8(bytes).unwrap());
            load_context.set_default_asset(LoadedAsset::new(value));
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

mod tests {
    pub use super::*;

    #[test]
    fn can_parse_downtown_subset() {
        const TEST_DATA: &str = r"
# IPL generated from Max file downtown.max
inst
1860, doontoon03, 0, -445.4862671, 1280.132813, 42.78390503, 1, 1, 1, 0, 0, 0, 1
1861, doontoon04, 0, -303.8299866, 1394.506836, 6.610000134, 1, 1, 1, 0, 0, 0, 1
1862, doontoon09, 0, -798.4454346, 1039.305176, 12.29159546, 1, 1, 1, 0, 0, 0, 1
end
cull
end
pick
end
path
end
";

        let test_data = TEST_DATA.trim();
        let ipl = Ipl::parse(test_data);
        assert_eq!(
            ipl,
            Ipl {
                instances: vec![
                    ("doontoon03".to_owned(), [-445.48627, 42.783905, -1280.1328]),
                    ("doontoon04".to_owned(), [-303.83, 6.61, -1394.5068]),
                    ("doontoon09".to_owned(), [-798.44543, 12.291595, -1039.3052]),
                ],
            },
        );
    }
}
