use super::{GTA_FRAGMENT_SHADER_HANDLE, GTA_VERTEX_SHADER_HANDLE};
use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    math::{Vec2, Vec4},
    pbr::{AlphaMode, MaterialPipeline, SpecializedMaterial},
    prelude::Mesh,
    reflect::TypeUuid,
    render::{
        color::Color,
        mesh::MeshVertexBufferLayout,
        prelude::Shader,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
        texture::Image,
    },
};

pub const SUBMATERIAL_MAX_COUNT: usize = 256;

/// A material with "standard" properties used in PBR lighting
/// Standard property values with pictures here
/// <https://google.github.io/filament/Material%20Properties.pdf>.
///
/// May be created directly from a [`Color`] or an [`Image`].
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "029788fe-34b8-4480-b0ee-35bb82b4528d"]
pub struct GtaMaterial {
    pub base_color_texture: Option<Handle<Image>>,
    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: Color,
    pub emissive_texture: Option<Handle<Image>>,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `roughness * roughness_texture_value`
    pub perceptual_roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `metallic * metallic_texture_value`
    pub metallic: f32,
    pub metallic_roughness_texture: Option<Handle<Image>>,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    pub normal_map_texture: Option<Handle<Image>>,
    /// Normal map textures authored for DirectX have their y-component flipped. Set this to flip
    /// it to right-handed conventions.
    pub flip_normal_map_y: bool,
    pub occlusion_texture: Option<Handle<Image>>,
    /// Support two-sided lighting by automatically flipping the normals for "back" faces
    /// within the PBR lighting shader.
    /// Defaults to false.
    /// This does not automatically configure backface culling, which can be done via
    /// `cull_mode`.
    pub double_sided: bool,
    /// Whether to cull the "front", "back" or neither side of a mesh
    /// defaults to `Face::Back`
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub alpha_mode: AlphaMode,
    pub materials: Vec<renderware_format::dff::Material>,
    pub frames: Option<Vec<renderware_format::packer::Frame>>,
}

impl Default for GtaMaterial {
    fn default() -> Self {
        GtaMaterial {
            base_color_texture: None,
            emissive: Color::BLACK,
            emissive_texture: None,
            // This is the minimum the roughness is clamped to in shader code
            // See <https://google.github.io/filament/Filament.html#materialsystem/parameterization/>
            // It's the minimum floating point value that won't be rounded down to 0 in the
            // calculations used. Although technically for 32-bit floats, 0.045 could be
            // used.
            perceptual_roughness: 0.089,
            // Few materials are purely dielectric or metallic
            // This is just a default for mostly-dielectric
            metallic: 0.01,
            metallic_roughness_texture: None,
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see
            // <https://google.github.io/filament/Material%20Properties.pdf>
            reflectance: 0.5,
            occlusion_texture: None,
            normal_map_texture: None,
            flip_normal_map_y: false,
            double_sided: false,
            cull_mode: Some(Face::Back),
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            materials: vec![],
            frames: None,
        }
    }
}

impl From<Handle<Image>> for GtaMaterial {
    fn from(texture: Handle<Image>) -> Self {
        GtaMaterial {
            base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct GtaMaterialFlags: u32 {
        const BASE_COLOR_TEXTURE         = (1 << 0);
        const EMISSIVE_TEXTURE           = (1 << 1);
        const METALLIC_ROUGHNESS_TEXTURE = (1 << 2);
        const OCCLUSION_TEXTURE          = (1 << 3);
        const DOUBLE_SIDED               = (1 << 4);
        const UNLIT                      = (1 << 5);
        const ALPHA_MODE_OPAQUE          = (1 << 6);
        const ALPHA_MODE_MASK            = (1 << 7);
        const ALPHA_MODE_BLEND           = (1 << 8);
        const TWO_COMPONENT_NORMAL_MAP   = (1 << 9);
        const FLIP_NORMAL_MAP_Y          = (1 << 10);
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

#[derive(Copy, Clone, Default, AsStd140)]
pub struct GtaMaterialSubmaterialData {
    pub color: Vec4,
    pub uv_top_left: Vec2,
    pub uv_bottom_right: Vec2,
}

/// The GPU representation of the uniform data of a [`GtaMaterial`].
#[derive(Clone, AsStd140)]
pub struct GtaMaterialUniformData {
    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: Vec4,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    pub roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    pub metallic: f32,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    pub flags: u32,
    /// When the alpha mode mask flag is set, any base color alpha above this cutoff means fully opaque,
    /// and any below means fully transparent.
    pub alpha_cutoff: f32,
    /// The number of submaterials.
    pub submaterial_count: u32,
    pub submaterials: [GtaMaterialSubmaterialData; SUBMATERIAL_MAX_COUNT],
}

/// The GPU representation of a [`GtaMaterial`].
#[derive(Debug, Clone)]
pub struct GpuGtaMaterial {
    /// A buffer containing the [`GtaMaterialUniformData`] of the material.
    pub buffer: Buffer,
    /// The bind group specifying how the [`GtaMaterialUniformData`] and
    /// all the textures of the material are bound.
    pub bind_group: BindGroup,
    pub has_normal_map: bool,
    pub flags: GtaMaterialFlags,
    pub base_color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
}

impl RenderAsset for GtaMaterial {
    type ExtractedAsset = GtaMaterial;
    type PreparedAsset = GpuGtaMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<MaterialPipeline<GtaMaterial>>,
        SRes<RenderAssets<Image>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, gta_pipeline, gpu_images): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let (base_color_texture_view, base_color_sampler) = if let Some(result) = gta_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.base_color_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let (emissive_texture_view, emissive_sampler) = if let Some(result) = gta_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.emissive_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let (metallic_roughness_texture_view, metallic_roughness_sampler) = if let Some(result) =
            gta_pipeline
                .mesh_pipeline
                .get_image_texture(gpu_images, &material.metallic_roughness_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let (normal_map_texture_view, normal_map_sampler) = if let Some(result) = gta_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.normal_map_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let (occlusion_texture_view, occlusion_sampler) = if let Some(result) = gta_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.occlusion_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let mut flags = GtaMaterialFlags::NONE;
        if material.base_color_texture.is_some() {
            flags |= GtaMaterialFlags::BASE_COLOR_TEXTURE;
        }
        if material.emissive_texture.is_some() {
            flags |= GtaMaterialFlags::EMISSIVE_TEXTURE;
        }
        if material.metallic_roughness_texture.is_some() {
            flags |= GtaMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }
        if material.occlusion_texture.is_some() {
            flags |= GtaMaterialFlags::OCCLUSION_TEXTURE;
        }
        if material.double_sided {
            flags |= GtaMaterialFlags::DOUBLE_SIDED;
        }
        if material.unlit {
            flags |= GtaMaterialFlags::UNLIT;
        }
        let has_normal_map = material.normal_map_texture.is_some();
        if has_normal_map {
            match gpu_images
                .get(material.normal_map_texture.as_ref().unwrap())
                .unwrap()
                .texture_format
            {
                // All 2-component unorm formats
                TextureFormat::Rg8Unorm
                | TextureFormat::Rg16Unorm
                | TextureFormat::Bc5RgUnorm
                | TextureFormat::EacRg11Unorm => {
                    flags |= GtaMaterialFlags::TWO_COMPONENT_NORMAL_MAP
                }
                _ => {}
            }
            if material.flip_normal_map_y {
                flags |= GtaMaterialFlags::FLIP_NORMAL_MAP_Y;
            }
        }
        // NOTE: 0.5 is from the glTF default - do we want this?
        let mut alpha_cutoff = 0.5;
        match material.alpha_mode {
            AlphaMode::Opaque => flags |= GtaMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= GtaMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= GtaMaterialFlags::ALPHA_MODE_BLEND,
        };

        let submaterials = &material.materials;
        assert!(submaterials.len() < SUBMATERIAL_MAX_COUNT);
        let mut value = GtaMaterialUniformData {
            emissive: material.emissive.into(),
            roughness: material.perceptual_roughness,
            metallic: material.metallic,
            reflectance: material.reflectance,
            flags: flags.bits(),
            alpha_cutoff,
            submaterial_count: submaterials.len() as u32,
            submaterials: [Default::default(); SUBMATERIAL_MAX_COUNT],
        };
        for (idx, submaterial) in submaterials.iter().enumerate() {
            let c = submaterial.color;
            let frame = material.frames.as_ref().map(|f| f[idx]);
            let (uv_top_left, uv_bottom_right) = match frame {
                Some(f) => (f.top_left.into(), f.bottom_right.into()),
                None => (Vec2::ZERO, Vec2::ZERO),
            };
            value.submaterials[idx] = GtaMaterialSubmaterialData {
                color: Color::rgba_u8(c.r, c.g, c.b, c.a).into(),
                uv_top_left,
                uv_bottom_right,
            };
        }
        let value_std140 = value.as_std140();
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("gta_material_uniform_buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: value_std140.as_bytes(),
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(base_color_texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(base_color_sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(emissive_texture_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(emissive_sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(metallic_roughness_texture_view),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(metallic_roughness_sampler),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::TextureView(occlusion_texture_view),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Sampler(occlusion_sampler),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::TextureView(normal_map_texture_view),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::Sampler(normal_map_sampler),
                },
            ],
            label: Some("gta_material_bind_group"),
            layout: &gta_pipeline.material_layout,
        });

        Ok(GpuGtaMaterial {
            buffer,
            bind_group,
            flags,
            has_normal_map,
            base_color_texture: material.base_color_texture,
            alpha_mode: material.alpha_mode,
            cull_mode: material.cull_mode,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GtaMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
}

impl SpecializedMaterial for GtaMaterial {
    type Key = GtaMaterialKey;

    fn key(render_asset: &<Self as RenderAsset>::PreparedAsset) -> Self::Key {
        GtaMaterialKey {
            normal_map: render_asset.has_normal_map,
            cull_mode: render_asset.cull_mode,
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.normal_map {
            descriptor
                .fragment
                .as_mut()
                .unwrap()
                .shader_defs
                .push(String::from("GTAMATERIAL_NORMAL_MAP"));
        }
        descriptor.primitive.cull_mode = key.cull_mode;
        if let Some(label) = &mut descriptor.label {
            *label = format!("gta_{}", *label).into();
        }

        let vertex_buffer_layout = {
            let mut vertex_attributes = vec![
                Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ];
            if layout.contains(Mesh::ATTRIBUTE_TANGENT) {
                vertex_attributes.push(Mesh::ATTRIBUTE_TANGENT.at_shader_location(3));
            }
            if layout.contains(Mesh::ATTRIBUTE_JOINT_INDEX)
                && layout.contains(Mesh::ATTRIBUTE_JOINT_WEIGHT)
            {
                vertex_attributes.push(Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(4));
                vertex_attributes.push(Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(5));
            };
            vertex_attributes.push(super::ATTRIBUTE_MATERIAL_ID.at_shader_location(6));

            layout.get_layout(&vertex_attributes)?
        };
        descriptor.vertex.buffers = vec![vertex_buffer_layout];

        Ok(())
    }

    fn vertex_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(GTA_VERTEX_SHADER_HANDLE.typed())
    }

    fn fragment_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(GTA_FRAGMENT_SHADER_HANDLE.typed())
    }

    #[inline]
    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(
        render_device: &RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            GtaMaterialUniformData::std140_size_static() as u64,
                        ),
                    },
                    count: None,
                },
                // Base Color Texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Base Color Texture Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Emissive Texture
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Emissive Texture Sampler
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Metallic Roughness Texture
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Metallic Roughness Texture Sampler
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Occlusion Texture
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Occlusion Texture Sampler
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal Map Texture
                BindGroupLayoutEntry {
                    binding: 9,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Normal Map Texture Sampler
                BindGroupLayoutEntry {
                    binding: 10,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("gta_material_layout"),
        })
    }

    #[inline]
    fn alpha_mode(render_asset: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        render_asset.alpha_mode
    }
}
