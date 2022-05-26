use bevy::{
    asset::load_internal_asset,
    prelude::*,
    reflect::TypeUuid,
    render::{mesh::MeshVertexAttribute, render_resource::VertexFormat},
};

pub mod gta_material;
pub use gta_material::*;

pub const GTA_VERTEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12055104379192973046);

pub const GTA_FRAGMENT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5314570609969361564);

pub const GTA_COMMON_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 14824254865876030762);

pub const ATTRIBUTE_MATERIAL_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("MaterialId", 2708715425, VertexFormat::Uint32);

pub type GtaBundle = MaterialMeshBundle<GtaMaterial>;

#[derive(Default)]
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        use bevy::asset as bevy_asset;

        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            GTA_COMMON_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("gta_common.wgsl")),
        );

        load_internal_asset!(
            app,
            GTA_VERTEX_SHADER_HANDLE,
            "gta_vertex.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            GTA_FRAGMENT_SHADER_HANDLE,
            "gta_fragment.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(MaterialPlugin::<GtaMaterial>::default());

        app.world
            .resource_mut::<Assets<GtaMaterial>>()
            .set_untracked(
                Handle::<GtaMaterial>::default(),
                GtaMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    ..default()
                },
            );
    }
}
