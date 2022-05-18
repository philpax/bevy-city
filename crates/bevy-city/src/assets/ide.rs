use bevy::{
    app::prelude::*,
    asset::{AddAsset, AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

use bitflags::bitflags;

bitflags! {
    // Vice City only. They're different per game...
    // https://gtamods.com/wiki/Item_Definition#IDE_Flags
    pub struct ObjectFlagsVC: u32 {
        // Identifies objects to draw "wet reflections" on them.
        const IS_ROAD = 0x1;
        // Do not fade the object when it is being loaded into or out of view.
        const DO_NOT_FADE = 0x2;
        // Model is transparent. Render this object after all opaque objects, allowing transparencies of other objects to be visible through this object.
        const DRAW_LAST = 0x4;
        // Render with additive blending. Previous flag will be enabled automatically.
        const ADDITIVE = 0x8;
        // Don't use static lighting, we want dynamic if it's possible.
        const IGNORE_LIGHTING = 0x20;
        // Model is a shadow. Disable writing to z-buffer when rendering it, allowing transparencies of other objects, shadows, and lights to be visible through this object.
        const NO_ZBUFFER_WRITE = 0x40;
        // Do not draw dynamic shadows on this object.
        const DONT_RECEIVE_SHADOWS = 0x80;
        // Ignore draw distance for this object (sets its "level" (island id) to 0).
        const IGNORE_DRAW_DISTANCE = 0x100;
        // Breakable glass type 1: glass object changes its textures when breaking.
        const IS_GLASS_TYPE_1 = 0x200;
        // Breakable glass type 2: glass object doesn't change its textures when breaking.
        const IS_GLASS_TYPE_2 = 0x400;
    }
}

#[derive(Debug, PartialEq)]
pub struct Object {
    pub id: u32,
    pub model_name: String,
    pub texture_name: String,
    pub mesh_count: Option<u16>,
    pub draw_distance: f32,
    pub flags: ObjectFlagsVC,
}

#[derive(Debug, PartialEq)]
pub struct Weapon {
    pub id: u32,
    pub model_name: String,
    pub texture_name: String,
    pub animation_name: String,
    pub draw_distance: f32,
}

#[derive(Debug, TypeUuid, PartialEq)]
#[uuid = "b80d2ec1-3e87-49eb-8e8e-c24658383203"]
pub struct Ide {
    pub objects: Vec<Object>,
    pub weapons: Vec<Weapon>,
}

impl Ide {
    pub fn parse(data: &str) -> Self {
        let sections = super::common::categorise_lines(data);
        let section_iter = |section| {
            sections
                .get(section)
                .map(|v| v.as_slice())
                .unwrap_or_default()
                .iter()
        };

        let objects: Vec<_> = section_iter("objs")
            .map(|line| {
                let segments: Vec<_> = super::common::split_line(line);
                let (id, model_name, texture_name) = (
                    segments[0].parse().unwrap(),
                    segments[1].to_string(),
                    segments[2].to_string(),
                );
                let flags =
                    ObjectFlagsVC::from_bits(segments.last().unwrap().parse().unwrap()).unwrap();

                let (mesh_count, draw_distance) = match segments.len() {
                    5 => (None, segments[3].parse().unwrap()),
                    6 | 7 | 8 => (
                        Some(segments[3].parse().unwrap()),
                        segments[4].parse().unwrap(),
                    ),
                    _ => panic!("unexpected length of ide segments"),
                };

                Object {
                    id,
                    model_name,
                    texture_name,
                    mesh_count,
                    draw_distance,
                    flags,
                }
            })
            .collect();

        let weapons: Vec<_> = section_iter("weap")
            .map(|line| {
                let segments: Vec<_> = super::common::split_line(line);
                let (id, model_name, texture_name, animation_name) = (
                    segments[0].parse().unwrap(),
                    segments[1].to_string(),
                    segments[2].to_string(),
                    segments[3].to_string(),
                );
                let draw_distance = segments[5].parse().unwrap();

                Weapon {
                    id,
                    model_name,
                    texture_name,
                    animation_name,
                    draw_distance,
                }
            })
            .collect();

        Ide { objects, weapons }
    }
}

#[derive(Default)]
pub struct IdeLoader;

impl AssetLoader for IdeLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let value = Ide::parse(std::str::from_utf8(bytes).unwrap());
            load_context.set_default_asset(LoadedAsset::new(value));
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

mod tests {
    pub use super::*;

    #[test]
    fn can_parse_club_subset() {
        const TEST_DATA: &str = r#"
objs
4720, clubceilingdome, mainclub2, 1, 100, 160
4721, cl_main_room, hi_cutmaincl, 1, 100, 160
4722, cl_recessedlights1, mainclub2, 1, 100, 164
end
tobj
end
path
end
2dfx
4721, 9.89917, -4.43922, -2.89738, 184, 255, 0, 120, 3, 1, 0.035553, -0.999368, -6.81368e-005, 0.035553, -0.999368, -6.81368e-005
4721, 9.39572, -4.45834, -2.89738, 184, 255, 0, 120, 3, 1, 0.035553, -0.999368, -6.81368e-005, 0.035553, -0.999368, -6.81368e-005
4722, -2.76812, 16.3968, -0.43701, 94, 50, 50, 200, 0, "coronastar", "shad_exp", 100, 0, 0.5, 0, 40, 0, 0, 0, 0
end
"#;

        let test_data = TEST_DATA.trim();
        type F = ObjectFlagsVC;
        assert_eq!(
            Ide::parse(test_data),
            Ide {
                objects: vec![
                    Object {
                        id: 4720,
                        model_name: "clubceilingdome".to_string(),
                        texture_name: "mainclub2".to_string(),
                        mesh_count: Some(1),
                        draw_distance: 100.0,
                        flags: F::IGNORE_LIGHTING | F::DONT_RECEIVE_SHADOWS,
                    },
                    Object {
                        id: 4721,
                        model_name: "cl_main_room".to_string(),
                        texture_name: "hi_cutmaincl".to_string(),
                        mesh_count: Some(1),
                        draw_distance: 100.0,
                        flags: F::IGNORE_LIGHTING | F::DONT_RECEIVE_SHADOWS,
                    },
                    Object {
                        id: 4722,
                        model_name: "cl_recessedlights1".to_string(),
                        texture_name: "mainclub2".to_string(),
                        mesh_count: Some(1),
                        draw_distance: 100.0,
                        flags: F::DRAW_LAST | F::IGNORE_LIGHTING | F::DONT_RECEIVE_SHADOWS,
                    },
                ],
                weapons: vec![],
            }
        );
    }

    #[test]
    fn can_parse_default_subset() {
        const TEST_DATA: &str = r#"
peds
9, HFYST, HFYST, CIVFEMALE, STAT_STREET_GIRL, sexywoman, 013, null, 6,1		
83, CBa, CBa, GANG1, STAT_GANG1, gang1, 0, null, 6,6
97, vice1, vice1, COP, STAT_COP, man, 0, null, 9,9
106, WFYG2, WFYG2, CIVFEMALE, STAT_SENSIBLE_GIRL, woman, 0, null, 9,9
109, special01, generic, CIVMALE, STAT_STD_MISSION, man, 0, null, 9,9
end

cars
130,	landstal, 	landstal, 	car, 	LANDSTAL, 	LANDSTK, 		null,	normal, 	10,	7,	0,		254, 0.8
165, 	chopper, 	chopper, 	heli,	HELI, 		HELI,	 		null,	ignore,		10,	7,	0
180, 	airtrain, 	airtrain, 	plane,	AIRTRAIN,	AEROPL, 		null,	ignore,		10,	7,	0,		257
198,	sanchez,	sanchez,	bike,	DIRTBIKE,	SANCHEZ,		biked,	motorbike,	10,	7,	0,		23, 0.66
223, 	jetmax, 	jetmax, 	boat,	CUPBOAT,	CUBJET, 		null,	ignore,		10,	7, 	0
end

objs
237, wheel_rim, generic, 2, 20, 70, 0
end

weap
265, hammer, hammer, baseball, 1, 50, 0
end

hier
295, cutobj01, generic
end
"#;

        let test_data = TEST_DATA.trim();
        assert_eq!(
            Ide::parse(test_data),
            Ide {
                objects: vec![Object {
                    id: 237,
                    model_name: "wheel_rim".to_string(),
                    texture_name: "generic".to_string(),
                    mesh_count: Some(2),
                    draw_distance: 20.0,
                    flags: ObjectFlagsVC::empty(),
                }],
                weapons: vec![Weapon {
                    id: 265,
                    model_name: "hammer".to_string(),
                    texture_name: "hammer".to_string(),
                    animation_name: "baseball".to_string(),
                    draw_distance: 50.0,
                }],
            }
        );
    }
}
