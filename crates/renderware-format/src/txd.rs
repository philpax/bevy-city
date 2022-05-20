use crate::raw::{constants::SectionType, BinaryStreamFile, ClumpData};

pub use crate::raw::{
    constants::{TextureAddressing, TextureFiltering},
    Color,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Texture {
    pub filtering: TextureFiltering,
    pub uv: (TextureAddressing, TextureAddressing),

    pub name: String,
    pub mask_name: String,

    pub width: u16,
    pub height: u16,

    pub data: Vec<u8>,
}

impl Texture {
    pub fn from_raw(raw: &BinaryStreamFile) -> Vec<Texture> {
        let main = &raw.sections[0];
        main.find_children_by_type(SectionType::Raster)
            .filter_map(|r| match r.get_child_struct_data()? {
                ClumpData::Raster(r) => Some(Texture {
                    filtering: r.filtering,
                    uv: r.uv,
                    name: r.name.clone(),
                    mask_name: r.mask_name.clone(),
                    width: r.width,
                    height: r.height,
                    data: decompress_dxt(
                        &r.data,
                        r.width as usize,
                        r.height as usize,
                        r.compression,
                    ),
                }),
                _ => None,
            })
            .collect()
    }

    pub fn as_colors(&self) -> impl Iterator<Item = Color> + '_ {
        self.data
            .chunks_exact(4)
            .map(|c| Color::new(c[0], c[1], c[2], c[3]))
    }
}

fn decompress_dxt(data: &[u8], width: usize, height: usize, compression: u8) -> Vec<u8> {
    let mut uncompressed = vec![0u8; width * height * 4];
    match compression {
        1 => {
            squish::Format::Bc1.decompress(data, width, height, &mut uncompressed);
        }
        3 => {
            squish::Format::Bc3.decompress(data, width, height, &mut uncompressed);
        }
        _ => unimplemented!(),
    }

    uncompressed
}
