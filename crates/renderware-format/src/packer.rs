use std::collections::{HashMap, HashSet};

use crate::{
    dff::{self, Vertex},
    txd,
};

use itertools::Itertools;
use texture_packer as tp;
use tp::texture::{
    memory_rgba8_texture::{MemoryRGBA8Texture, RGBA8},
    Texture,
};

pub fn repack_model(
    model: &dff::Model,
    textures: &[txd::Texture],
) -> (Vec<dff::Vertex>, Vec<dff::Triangle>, txd::Texture) {
    // Sort our materials so that the smallest is added to the packer first.
    let texture_data_by_name: HashMap<_, _> =
        textures.iter().map(|t| (t.name.clone(), t)).collect();
    let mut materials: Vec<_> = model
        .material_indices
        .iter()
        .copied()
        .map(|idx| {
            (
                idx as u16,
                material_to_texture_data(&texture_data_by_name, &model.materials[idx]),
            )
        })
        .collect();
    materials.sort_by_key(|m| -((m.1 .1 * m.1 .2) as i32));

    // Start packing!
    let mut packer = tp::TexturePacker::new_skyline(tp::TexturePackerConfig {
        allow_rotation: false,
        texture_padding: 0,
        texture_outlines: false,
        ..Default::default()
    });
    for (idx, (buf, width, height)) in materials {
        let mem_texture = MemoryRGBA8Texture::from_memory(&buf, width, height);
        packer.pack_own(idx, mem_texture).unwrap();
    }
    let packer = packer;

    // Copy our packed texture into a buffer.
    let width = packer.width();
    let height = packer.height();
    let mut new_texture_data = vec![];
    for y in 0..height {
        for x in 0..width {
            let p = packer.get(x, y).unwrap_or(RGBA8 {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            });
            new_texture_data.extend_from_slice(&[p.r, p.g, p.b, p.a]);
        }
    }

    // Build a UV remap table for each material ID.
    let remap_bounds_by_material_id: HashMap<_, _> = packer
        .get_frames()
        .iter()
        .map(|(material_id, frame)| {
            let tp::Rect { x, y, w, h } = frame.frame;
            let (x0, x1) = (x as f32, (x + w) as f32);
            let (y0, y1) = (y as f32, (y + h) as f32);

            let w = width as f32;
            let h = height as f32;
            (*material_id, ((x0 / w, x1 / w), (y0 / h, y1 / h)))
        })
        .collect();

    // Extract submeshes, grouped by material ID. These submeshes are then combined
    // to produce our final repacked model.
    let submeshes = generate_submeshes_by_material_id(
        &model.vertices,
        &model.triangles,
        &remap_bounds_by_material_id,
    );

    // Concatenate our submeshes.
    let mut vertices = vec![];
    let mut triangles = vec![];
    for submesh in submeshes {
        let base = vertices.len() as u16;
        vertices.extend_from_slice(&submesh.0);
        triangles.extend(submesh.1.iter().map(|t| dff::Triangle {
            vertex1: base + t.vertex1,
            vertex2: base + t.vertex2,
            vertex3: base + t.vertex3,
            material_id: t.material_id,
        }));
    }

    // Finally, generate our texture, and return!
    let base_texture = &textures[0];
    let texture = txd::Texture {
        filtering: base_texture.filtering,
        uv: base_texture.uv,
        name: String::new(),
        mask_name: String::new(),
        width: width.try_into().unwrap(),
        height: height.try_into().unwrap(),
        data: new_texture_data,
    };

    (vertices, triangles, texture)
}

type UvRemapBounds = ((f32, f32), (f32, f32));

fn generate_submeshes_by_material_id(
    vertices: &[dff::Vertex],
    triangles: &[dff::Triangle],
    remap_bounds_by_material_id: &HashMap<u16, UvRemapBounds>,
) -> Vec<(Vec<Vertex>, Vec<dff::Triangle>)> {
    let mut triangles = triangles.to_vec();
    triangles.sort_by_key(|t| t.material_id);
    triangles
        .iter()
        .group_by(|t| t.material_id)
        .into_iter()
        .map(|(material_id, triangles)| {
            let remap_bounds = remap_bounds_by_material_id.get(&material_id).copied();
            generate_submesh(vertices, triangles, remap_bounds)
        })
        .collect()
}

fn generate_submesh<'a>(
    vertices: &[dff::Vertex],
    triangles: impl Iterator<Item = &'a dff::Triangle>,
    remap_bounds: Option<UvRemapBounds>,
) -> (Vec<Vertex>, Vec<dff::Triangle>) {
    // thank you stack overflow
    // https://stackoverflow.com/a/3451607
    let remap =
        |value, low1, high1, low2, high2| low2 + (value - low1) * (high2 - low2) / (high1 - low1);
    let remap_01 = |value: f32, (low, high)| remap(value.clamp(0.0, 1.0), 0.0, 1.0, low, high);

    let triangles = triangles.collect_vec();
    let remap_bounds = remap_bounds.unwrap_or(((0.0, 1.0), (0.0, 1.0)));

    let our_indices = triangles
        .iter()
        .flat_map(|t| [t.vertex1, t.vertex2, t.vertex3])
        .collect::<HashSet<_>>()
        .into_iter()
        .collect_vec();

    let vertices = our_indices
        .iter()
        .map(|i| {
            let vertex = vertices[*i as usize];
            Vertex {
                uv: [
                    remap_01(vertex.uv[0], remap_bounds.0),
                    remap_01(vertex.uv[1], remap_bounds.1),
                ],
                ..vertex
            }
        })
        .collect_vec();

    let remap_index_table: HashMap<_, _> = our_indices
        .iter()
        .enumerate()
        .map(|(new_index, old_index)| (*old_index, new_index as u16))
        .collect();
    let remap_index = |idx| *remap_index_table.get(&idx).unwrap();

    let triangles = triangles
        .iter()
        .map(|t| dff::Triangle {
            vertex1: remap_index(t.vertex1),
            vertex2: remap_index(t.vertex2),
            vertex3: remap_index(t.vertex3),
            material_id: 0,
        })
        .collect_vec();

    (vertices, triangles)
}

fn material_to_texture_data(
    texture_data_by_name: &HashMap<String, &txd::Texture>,
    material: &dff::Material,
) -> (Vec<u8>, u32, u32) {
    let base_color = material.color;
    if let Some(texture) = &material.texture {
        let texture = texture_data_by_name.get(&texture.name).unwrap();
        let base_color = apply_to_vec4(base_color.as_array(), remap_c_to_f32);

        let mut buf: [u8; 4] = [0; 4];
        let data: Vec<_> = texture
            .data
            .chunks_exact(4)
            .flat_map(|col| {
                buf.copy_from_slice(col);
                let tex_color = apply_to_vec4(buf, remap_c_to_f32);
                apply_to_vec4(
                    [
                        tex_color[0] * base_color[0],
                        tex_color[1] * base_color[1],
                        tex_color[2] * base_color[2],
                        tex_color[3] * base_color[3],
                    ],
                    remap_f32_to_c,
                )
            })
            .collect();

        (data, texture.width as _, texture.height as _)
    } else {
        let width = 8;
        let height = 8;

        let data = std::iter::repeat(base_color.as_array())
            .take((width * height) as usize)
            .flatten()
            .collect::<Vec<_>>();

        (data, width, height)
    }
}

fn remap_c_to_f32(c: u8) -> f32 {
    c as f32 / 255.0
}

fn remap_f32_to_c(c: f32) -> u8 {
    (c * 255.0) as u8
}

fn apply_to_vec4<T: Copy, U>(v: [T; 4], f: impl Fn(T) -> U) -> [U; 4] {
    [f(v[0]), f(v[1]), f(v[2]), f(v[3])]
}
