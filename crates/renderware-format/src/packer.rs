use std::collections::HashMap;

use crate::{
    dff::{self},
    txd,
};

use texture_packer as tp;
use tp::texture::{
    memory_rgba8_texture::{MemoryRGBA8Texture, RGBA8},
    Texture,
};

pub fn repack_model_textures(model: &dff::Model, textures: &[txd::Texture]) -> txd::Texture {
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

    // Finally, generate our texture, and return!
    let base_texture = &textures[0];
    

    txd::Texture {
        filtering: base_texture.filtering,
        uv: base_texture.uv,
        name: String::new(),
        mask_name: String::new(),
        width: width.try_into().unwrap(),
        height: height.try_into().unwrap(),
        data: new_texture_data,
    }
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
