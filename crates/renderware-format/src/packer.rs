use std::collections::HashMap;

use crate::{dff, txd};

use texture_packer as tp;
use tp::texture::{
    memory_rgba8_texture::{MemoryRGBA8Texture, RGBA8},
    Texture,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Frame {
    pub top_left: (f32, f32),
    pub bottom_right: (f32, f32),
}

#[derive(Debug, PartialEq)]
pub struct PackedTexture {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    pub frames: Vec<Frame>,
}

pub fn repack_model_textures(
    materials: &[dff::Material],
    material_indices: &[usize],
    textures: &[txd::Texture],
) -> PackedTexture {
    // Sort our materials so that the smallest is added to the packer first.
    let texture_data_by_name: HashMap<_, _> =
        textures.iter().map(|t| (t.name.clone(), t)).collect();
    let mut materials: Vec<_> = material_indices
        .iter()
        .copied()
        .map(|idx| {
            (
                idx as u16,
                material_to_texture_data(&texture_data_by_name, &materials[idx]),
            )
        })
        .collect();
    materials.sort_by_key(|m| -((m.1 .1 * m.1 .2) as i32));

    // Start packing!
    let mut packer = tp::TexturePacker::new_skyline(tp::TexturePackerConfig {
        max_width: 4096,
        max_height: 4096,
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

    PackedTexture {
        width: width.try_into().unwrap(),
        height: height.try_into().unwrap(),
        data: new_texture_data,
        frames: material_indices
            .iter()
            .map(|i| {
                let frame = packer.get_frame(&(*i as u16)).unwrap();
                let r = frame.frame;
                let (width, height) = (width as f32, height as f32);
                Frame {
                    top_left: (r.left() as f32 / width, r.top() as f32 / height),
                    bottom_right: (r.right() as f32 / width, r.bottom() as f32 / height),
                }
            })
            .collect(),
    }
}

fn material_to_texture_data(
    texture_data_by_name: &HashMap<String, &txd::Texture>,
    material: &dff::Material,
) -> (Vec<u8>, u32, u32) {
    let base_color = material.color;
    if let Some(texture) = &material.texture {
        if let Some(texture) = texture_data_by_name.get(&texture.name) {
            return texture_to_texture_data(base_color, texture);
        }
    }

    let width = 8;
    let height = 8;

    let data = std::iter::repeat(base_color.as_array())
        .take((width * height) as usize)
        .flatten()
        .collect::<Vec<_>>();

    (data, width, height)
}

fn texture_to_texture_data(base_color: txd::Color, texture: &txd::Texture) -> (Vec<u8>, u32, u32) {
    let base_color = base_color.as_array().map(remap_u8_to_f32);
    let mut buf: [u8; 4] = [0; 4];
    let data: Vec<_> = texture
        .data
        .chunks_exact(4)
        .flat_map(|col| {
            buf.copy_from_slice(col);
            let tex_color = buf.map(remap_u8_to_f32);
            [
                tex_color[0] * base_color[0],
                tex_color[1] * base_color[1],
                tex_color[2] * base_color[2],
                tex_color[3] * base_color[3],
            ]
            .map(remap_f32_to_u8)
        })
        .collect();
    (data, texture.width as _, texture.height as _)
}

fn remap_u8_to_f32(c: u8) -> f32 {
    c as f32 / 255.0
}

fn remap_f32_to_u8(c: f32) -> u8 {
    (c * 255.0) as u8
}
