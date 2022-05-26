#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] tangent: vec4<f32>;
#endif
#ifdef SKINNED
    [[location(4)]] joint_indices: vec4<u32>;
    [[location(5)]] joint_weights: vec4<f32>;
#endif
    [[location(6)]] material_id: u32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] world_tangent: vec4<f32>;
#endif
    [[location(4)]] material_id: u32;
};

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;
#ifdef SKINNED
[[group(2), binding(1)]]
var<uniform> joint_matrices: SkinnedMesh;
#import bevy_pbr::skinning
#endif

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
    out.world_position = model * vec4<f32>(vertex.position, 1.0);
    out.world_normal = skin_normals(model, vertex.normal);
#ifdef VERTEX_TANGENTS
    out.world_tangent = skin_tangents(model, vertex.tangent);
#endif
#else
    out.world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    out.world_normal = mat3x3<f32>(
        mesh.inverse_transpose_model[0].xyz,
        mesh.inverse_transpose_model[1].xyz,
        mesh.inverse_transpose_model[2].xyz
    ) * vertex.normal;
#ifdef VERTEX_TANGENTS
    out.world_tangent = vec4<f32>(
        mat3x3<f32>(
            mesh.model[0].xyz,
            mesh.model[1].xyz,
            mesh.model[2].xyz
        ) * vertex.tangent.xyz,
        vertex.tangent.w
    );
#endif
#endif

    out.uv = vertex.uv;
    out.clip_position = view.view_proj * out.world_position;
    out.material_id = vertex.material_id;
    return out;
}