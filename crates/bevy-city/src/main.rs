use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use maps::ipl_parser::Ipl;

pub mod maps;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_renderware::RwPlugin)
        .add_plugin(maps::ipl_parser::IplPlugin)
        .add_plugin(EditorPlugin)
        .add_startup_system(setup)
        // .add_system(load_map)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let downtown_handle: Handle<Ipl> = asset_server.load("data/maps/downtown/downtown.ipl");
    commands.insert_resource(downtown_handle);

    commands.spawn_bundle(PbrBundle {
        mesh: asset_server.load("models/gta3/uzi.dff"),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn load_map(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut ev_asset: EventReader<AssetEvent<Ipl>>,
    assets: Res<Assets<Ipl>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let ipl = assets.get(handle).unwrap();
                let avg_position = ipl
                    .instances
                    .iter()
                    .map(|(_, pos)| Vec3::new(pos[0], pos[2], pos[1]))
                    .reduce(|acc, pos| acc + pos)
                    .unwrap_or_default()
                    / (ipl.instances.len() as f32);

                for (name, position) in &ipl.instances {
                    let path = format!("models/gta3/{name}.dff");
                    let model_handle = asset_server.load(&path);

                    commands.spawn_bundle(PbrBundle {
                        mesh: model_handle,
                        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                        transform: Transform::from_translation(
                            Vec3::new(position[0], position[2], position[1]) - avg_position,
                        ),
                        ..default()
                    });
                }
            }
            AssetEvent::Modified { handle: _handle } => {
                panic!("you aren't meant to modify the IPLs during gameplay!");
            }
            AssetEvent::Removed { handle: _handle } => {}
        }
    }
}
