#![allow(clippy::too_many_arguments)]

use std::{collections::HashMap, path::PathBuf};

use bevy::{
    prelude::*,
    render::{render_resource::WgpuFeatures, settings::WgpuSettings},
};

use bevy_atmosphere::*;
use bevy_editor_pls::{
    editor_window::{EditorWindow, EditorWindowContext},
    prelude::*,
};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_renderware::dff::Dff;

use clap::Parser;

pub mod assets;
use assets::{Dat, Ide, Ipl};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// If provided, render this asset by itself
    #[clap(short, long)]
    path: Option<PathBuf>,
}

struct DesiredAssetRenderPath(PathBuf);
struct DesiredAssetMeshes(Vec<(Handle<Dff>, Transform, bool)>);
struct GlobalDat(Handle<Dat>);
#[derive(PartialEq, Eq)]
enum LoadedIdes {
    Unloaded,
    Unprocessed(Vec<Handle<Ide>>),
    Processed,
}
enum PendingIpls {
    NoneRequested,
    Unloaded,
    Loaded(Vec<Handle<Ipl>>),
}
struct ModelTextureMap(HashMap<String, String>);
struct GameTime(f32);
#[derive(Component)]
struct Sun;

const EXTERIOR_MAP_SIZE: f32 = 10_000.0;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let mut app = App::new();

    // Preliminary setup
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_renderware::RwPlugin)
        .add_plugins(assets::ViceCityPluginGroup)
        .add_plugin(EditorPlugin)
        .insert_resource(DesiredAssetMeshes(vec![]))
        .insert_resource(LoadedIdes::Unloaded)
        .insert_resource(ModelTextureMap(HashMap::new()));

    // Loading systems
    app.add_startup_system(load_vice_city_dat)
        .add_system(handle_dat_events)
        .add_system(handle_ipl_events)
        .add_system(process_pending_desired_meshes)
        .add_system(process_pending_ides);

    // Primary behaviour
    if let Some(path) = args.path {
        let path = DesiredAssetRenderPath(path.strip_prefix("assets/")?.into());
        app.insert_resource(PendingIpls::NoneRequested)
            .insert_resource(path)
            .add_startup_system(asset_viewer);
    } else {
        app.insert_resource(PendingIpls::Unloaded)
            .add_startup_system(load_maps)
            // Gameplay
            .add_plugin(NoCameraPlayerPlugin)
            .insert_resource(MovementSettings {
                sensitivity: 0.00015,
                speed: 120.0,
            })
            .insert_resource(GameTime(12.0))
            .add_system(update_game_time)
            .add_editor_window::<TimeEditorWindow>()
            // Bevy atmosphere
            .insert_resource(AtmosphereMat::default())
            .add_plugin(AtmospherePlugin {
                dynamic: true,
                sky_radius: EXTERIOR_MAP_SIZE,
            })
            .add_system(daylight_cycle);
    };

    app.run();

    Ok(())
}

fn load_vice_city_dat(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GlobalDat(asset_server.load("data/gta_vc.dat")));
}

fn asset_viewer(
    mut commands: Commands,
    mut desired_asset_meshes: ResMut<DesiredAssetMeshes>,
    asset_server: Res<AssetServer>,
    desired_asset_render_path: Res<DesiredAssetRenderPath>,
) {
    desired_asset_meshes.0.push((
        asset_server.load(desired_asset_render_path.0.as_path()),
        Transform::from_xyz(0.0, 0.5, 0.0),
        false,
    ));

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
        transform: Transform::from_xyz(-1.0, 1.0, -1.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        ..default()
    });
}

fn load_maps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: EXTERIOR_MAP_SIZE * 2.0,
        })),
        material: materials.add(Color::rgba_u8(78, 156, 181, 255).into()),
        ..default()
    });

    commands
        .spawn_bundle(DirectionalLightBundle { ..default() })
        .insert(Sun);

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::ONE * 1000.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(FlyCam);
}

fn handle_dat_events(
    mut ev_asset: EventReader<AssetEvent<Dat>>,
    mut loaded_ides: ResMut<LoadedIdes>,
    mut pending_ipls: ResMut<PendingIpls>,
    global_dat: Res<GlobalDat>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Dat>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } if *handle == global_dat.0 => {
                let mut ides = vec![asset_server.load("data/default.ide")];
                let mut ipls = vec![];

                let dat = assets.get(handle).unwrap();
                for (filetype, path) in dat
                    .0
                    .lines()
                    .filter(|l| !(l.trim().is_empty() || l.starts_with('#')))
                    .filter_map(|l| l.split_once(' '))
                {
                    if !path.starts_with("DATA\\MAPS") {
                        continue;
                    }

                    let path = path
                        .replace('\\', "/")
                        .replace("DATA/MAPS", "data/maps")
                        .replace(".IDE", ".ide")
                        .replace(".IPL", ".ipl")
                        // hack: fix the case on some map IDEs...
                        .replace("haitin/haitin.ide", "haitiN/haitiN.ide")
                        .replace("oceandn/oceandn", "oceandn/oceandN")
                        // hack: fix the case on some map IDLs...
                        .replace("club.ipl", "CLUB.ipl")
                        .replace("haitin/haitin.ipl", "haitiN/haitin.ipl");

                    // hack: remove some ipls we don't care for
                    if path.ends_with("islandsf.ipl") {
                        continue;
                    }

                    match filetype {
                        "IDE" => ides.push(asset_server.load(&path)),
                        "IPL" => ipls.push(path),
                        _ => {}
                    }
                }

                *loaded_ides = LoadedIdes::Unprocessed(ides);
                if let PendingIpls::Unloaded = *pending_ipls {
                    *pending_ipls = PendingIpls::Loaded(
                        ipls.into_iter().map(|p| asset_server.load(&p)).collect(),
                    );
                }
            }
            AssetEvent::Created { .. } => {}
            AssetEvent::Modified { handle: _handle } => {
                panic!("you aren't meant to modify the DATs during gameplay!");
            }
            AssetEvent::Removed { handle: _handle } => {}
        }
    }
}

fn handle_ipl_events(
    mut ev_asset: EventReader<AssetEvent<Ipl>>,
    mut desired_asset_meshes: ResMut<DesiredAssetMeshes>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<Ipl>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let ipl = assets.get(handle).unwrap();

                for instance in &ipl.instances {
                    if instance.interior != 0 {
                        // We don't support interiors right now!
                        continue;
                    }

                    let name = &instance.model_name;
                    if name.len() > 3 && name[..3].eq_ignore_ascii_case("lod") {
                        continue;
                    }

                    let model_handle = asset_server.load(&format!("models/gta3/{name}.dff"));
                    let (translation, _rotation, scale) =
                        (instance.position, instance.rotation, instance.scale);

                    // HACK(philpax): fix this at some point. I believe the parsed
                    // representation has the elements in the wrong order.
                    let rotation = default();

                    desired_asset_meshes.0.push((
                        model_handle,
                        Transform {
                            translation,
                            rotation,
                            scale,
                        },
                        false,
                    ));
                }
            }
            AssetEvent::Modified { handle: _handle } => {
                panic!("you aren't meant to modify the IPLs during gameplay!");
            }
            AssetEvent::Removed { handle: _handle } => {}
        }
    }
}

fn process_pending_desired_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut desired_asset_meshes: ResMut<DesiredAssetMeshes>,
    loaded_ides: Res<LoadedIdes>,
    model_texture_map: Res<ModelTextureMap>,
    asset_server: Res<AssetServer>,
    asset_meshes: Res<Assets<Dff>>,
) {
    if *loaded_ides != LoadedIdes::Processed {
        return;
    }

    for (handle, transform, spawned) in &mut desired_asset_meshes.0 {
        if *spawned {
            continue;
        }

        if let Some(dff) = asset_meshes.get(handle.clone()) {
            let mesh = meshes.add(dff.mesh.clone());
            let dff_material = dff.materials.get(0);

            let (base_color, base_color_texture) =
                match (dff_material, model_texture_map.0.get(&dff.name)) {
                    (Some(material), Some(name)) => (
                        Color::rgba_u8(
                            material.color.r,
                            material.color.g,
                            material.color.b,
                            material.color.a,
                        ),
                        Some(asset_server.load(&format!(
                            "models/gta3/{}.txd#{}",
                            name.replace("generic", "Generic"),
                            material.texture.name
                        ))),
                    ),
                    _ => (Color::WHITE, None),
                };

            commands.spawn_bundle(PbrBundle {
                mesh,
                material: materials.add(StandardMaterial {
                    base_color,
                    base_color_texture,
                    unlit: true,
                    ..default()
                }),
                transform: *transform,
                ..default()
            });

            *spawned = true;
        }
    }
}

fn process_pending_ides(
    mut loaded_ides: ResMut<LoadedIdes>,
    mut model_texture_map: ResMut<ModelTextureMap>,
    assets_ide: Res<Assets<Ide>>,
) {
    if let LoadedIdes::Unprocessed(ides) = &mut *loaded_ides {
        ides.retain(|ide| match assets_ide.get(ide.clone()) {
            Some(ide) => {
                let mtm = &mut model_texture_map.0;
                for object in ide.objects.iter() {
                    mtm.insert(object.model_name.clone(), object.texture_name.clone());
                }
                for weapon in ide.weapons.iter() {
                    mtm.insert(weapon.model_name.clone(), weapon.texture_name.clone());
                }

                false
            }
            None => true,
        });

        if ides.is_empty() {
            *loaded_ides = LoadedIdes::Processed;
        }
    }
}

fn update_game_time(mut game_time: ResMut<GameTime>, time: Res<Time>) {
    game_time.0 = (game_time.0 + (time.delta_seconds() / 60.0)) % 24.0;
}

pub struct TimeEditorWindow;
impl EditorWindow for TimeEditorWindow {
    type State = ();
    const NAME: &'static str = "Time";

    fn ui(world: &mut World, _cx: EditorWindowContext, ui: &mut bevy_editor_pls::egui::Ui) {
        if let Some(mut time) = world.get_resource_mut::<GameTime>() {
            ui.add(
                bevy_editor_pls::egui::widgets::Slider::new(&mut time.0, 0.0..=24.0)
                    .text("Current time"),
            );
        }
    }
}

fn daylight_cycle(
    mut sky_mat: ResMut<AtmosphereMat>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    time: Res<GameTime>,
) {
    let mut pos = sky_mat.sun_position;
    // The sky is directly overhead when t = pi/2.
    // pi/2/2pi = 1/4 * 24 = 6
    // That is, the sun will be overhead when the original time is 6h.
    // We offset the time to make 12h the overhead time.
    let t = (time.0 - 6.0) / 24.0 * std::f32::consts::TAU;
    pos.y = t.sin();
    pos.z = t.cos();
    sky_mat.sun_position = pos;

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
        directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
    }
}
