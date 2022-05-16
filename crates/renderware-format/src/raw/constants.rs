use bitflags::bitflags;
use num_derive::FromPrimitive;

bitflags! {
    pub struct GeometryFormat: u32 {
        // Is triangle strip (if disabled it will be an triangle list)
        const TRI_STRIP = 0x00000001;
        // Vertex translation
        const POSITIONS = 0x00000002;
        // Texture coordinates
        const TEXTURED = 0x00000004;
        // Vertex colors
        const PRELIT = 0x00000008;
        // Store normals
        const NORMALS = 0x00000010;
        // Geometry is lit (dynamic and static)
        const LIGHT = 0x00000020;
        // Modulate material color
        const MODULATE_MATERIAL_COLOR = 0x00000040;
        // Texture coordinates 2
        const TEXTURED2 = 0x00000080;
        // Native Geometry
        const NATIVE = 0x01000000;
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive)]
pub enum SectionType {
    Struct = 0x00000001,
    String = 0x00000002,
    Extension = 0x00000003,
    Camera = 0x00000005,
    Texture = 0x00000006,
    Material = 0x00000007,
    MaterialList = 0x00000008,
    AtomicSection = 0x00000009,
    PlaneSection = 0x0000000A,
    World = 0x0000000B,
    Spline = 0x0000000C,
    Matrix = 0x0000000D,
    FrameList = 0x0000000E,
    Geometry = 0x0000000F,
    Clump = 0x00000010,
    Light = 0x00000012,
    UnicodeString = 0x00000013,
    Atomic = 0x00000014,
    Raster = 0x00000015,
    TextureDictionary = 0x00000016,
    AnimationDatabase = 0x00000017,
    Image = 0x00000018,
    SkinAnimation = 0x00000019,
    GeometryList = 0x0000001A,
    AnimAnimation = 0x0000001B,
    Team = 0x0000001C,
    Crowd = 0x0000001D,
    DeltaMorphAnimation = 0x0000001E,
    RightToRender = 0x0000001f,
    MultiTextureEffectNative = 0x00000020,
    MultiTextureEffectDictionary = 0x00000021,
    TeamDictionary = 0x00000022,
    PlatformIndependentTextureDictionary = 0x00000023,
    TableofContents = 0x00000024,
    ParticleStandardGlobalData = 0x00000025,
    AltPipe = 0x00000026,
    PlatformIndependentPeds = 0x00000027,
    PatchMesh = 0x00000028,
    ChunkGroupStart = 0x00000029,
    ChunkGroupEnd = 0x0000002A,
    UVAnimationDictionary = 0x0000002B,
    CollTree = 0x0000002C,
    MetricsPLG = 0x00000101,
    SplinePLG = 0x00000102,
    StereoPLG = 0x00000103,
    VRMLPLG = 0x00000104,
    MorphPLG = 0x00000105,
    PVSPLG = 0x00000106,
    MemoryLeakPLG = 0x00000107,
    AnimationPLG = 0x00000108,
    GlossPLG = 0x00000109,
    LogoPLG = 0x0000010a,
    MemoryInfoPLG = 0x0000010b,
    RandomPLG = 0x0000010c,
    PNGImagePLG = 0x0000010d,
    BonePLG = 0x0000010e,
    VRMLAnimPLG = 0x0000010f,
    SkyMipmapVal = 0x00000110,
    MRMPLG = 0x00000111,
    LODAtomicPLG = 0x00000112,
    MEPLG = 0x00000113,
    LightmapPLG = 0x00000114,
    RefinePLG = 0x00000115,
    SkinPLG = 0x00000116,
    LabelPLG = 0x00000117,
    ParticlesPLG = 0x00000118,
    GeomTXPLG = 0x00000119,
    SynthCorePLG = 0x0000011a,
    STQPPPLG = 0x0000011b,
    PartPPPLG = 0x0000011c,
    CollisionPLG = 0x0000011d,
    HAnimPLG = 0x0000011e,
    UserDataPLG = 0x0000011f,
    MaterialEffectsPLG = 0x00000120,
    ParticleSystemPLG = 0x00000121,
    DeltaMorphPLG = 0x00000122,
    PatchPLG = 0x00000123,
    TeamPLG = 0x00000124,
    CrowdPPPLG = 0x00000125,
    MipSplitPLG = 0x00000126,
    AnisotropyPLG = 0x00000127,
    GCNMaterialPLG = 0x00000129,
    GeometricPVSPLG = 0x0000012a,
    XBOXMaterialPLG = 0x0000012b,
    MultiTexturePLG = 0x0000012c,
    ChainPLG = 0x0000012d,
    ToonPLG = 0x0000012e,
    PTankPLG = 0x0000012f,
    ParticleStandardPLG = 0x00000130,
    PDSPLG = 0x00000131,
    PrtAdvPLG = 0x00000132,
    NormalMapPLG = 0x00000133,
    ADCPLG = 0x00000134,
    UVAnimationPLG = 0x00000135,
    CharacterSetPLG = 0x00000180,
    NOHSWorldPLG = 0x00000181,
    ImportUtilPLG = 0x00000182,
    SlerpPLG = 0x00000183,
    OptimPLG = 0x00000184,
    TLWorldPLG = 0x00000185,
    DatabasePLG = 0x00000186,
    RaytracePLG = 0x00000187,
    RayPLG = 0x00000188,
    LibraryPLG = 0x00000189,
    _2DPLG = 0x00000190,
    TileRenderPLG = 0x00000191,
    JPEGImagePLG = 0x00000192,
    TGAImagePLG = 0x00000193,
    GIFImagePLG = 0x00000194,
    QuatPLG = 0x00000195,
    SplinePVSPLG = 0x00000196,
    MipmapPLG = 0x00000197,
    MipmapKPLG = 0x00000198,
    _2DFont = 0x00000199,
    IntersectionPLG = 0x0000019a,
    TIFFImagePLG = 0x0000019b,
    PickPLG = 0x0000019c,
    BMPImagePLG = 0x0000019d,
    RASImagePLG = 0x0000019e,
    SkinFXPLG = 0x0000019f,
    VCATPLG = 0x000001a0,
    _2DPath = 0x000001a1,
    _2DBrush = 0x000001a2,
    _2DObject = 0x000001a3,
    _2DShape = 0x000001a4,
    _2DScene = 0x000001a5,
    _2DPickRegion = 0x000001a6,
    _2DObjectString = 0x000001a7,
    _2DAnimationPLG = 0x000001a8,
    _2DAnimation = 0x000001a9,
    _2DKeyframe = 0x000001b0,
    _2DMaestro = 0x000001b1,
    Barycentric = 0x000001b2,
    PlatformIndependentTextureDictionaryTK = 0x000001b3,
    TOCTK = 0x000001b4,
    TPLTK = 0x000001b5,
    AltPipeTK = 0x000001b6,
    AnimationTK = 0x000001b7,
    SkinSplitTookit = 0x000001b8,
    CompressedKeyTK = 0x000001b9,
    GeometryConditioningPLG = 0x000001ba,
    WingPLG = 0x000001bb,
    GenericPipelineTK = 0x000001bc,
    LightmapConversionTK = 0x000001bd,
    FilesystemPLG = 0x000001be,
    DictionaryTK = 0x000001bf,
    UVAnimationLinear = 0x000001c0,
    UVAnimationParameter = 0x000001c1,
    BinMeshPLG = 0x0000050E,
    NativeDataPLG = 0x00000510,
    ZModelerLock = 0x0000F21E,
    AtomicVisibilityDistance = 0x0253F200,
    ClumpVisibilityDistance = 0x0253F201,
    FrameVisibilityDistance = 0x0253F202,
    PipelineSet = 0x0253F2F3,
    TexDictionaryLink = 0x0253F2F5,
    SpecularMaterial = 0x0253F2F6,
    _2dEffect = 0x0253F2F8,
    ExtraVertColour = 0x0253F2F9,
    CollisionModel = 0x0253F2FA,
    GTAHAnim = 0x0253F2FB,
    ReflectionMaterial = 0x0253F2FC,
    Breakable = 0x0253F2FD,
    NodeName = 0x0253F2FE,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive)]
pub enum TextureFiltering {
    // filtering is disabled
    NaFilterMode = 0,
    // Point sampled
    Nearest = 1,
    // Bilinear
    Linear = 2,
    // Point sampled per pixel mip map
    MipNearest = 3,
    // Bilinear per pixel mipmap
    MipLinear = 4,
    // MipMap interp point sampled
    LinearMipNearest = 5,
    // Trilinear
    LinearMipLinear = 6,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive)]
pub enum TextureAddressing {
    // no tiling
    NoTiling = 0,
    // tile in U or V direction
    Wrap = 1,
    // mirror in U or V direction
    Mirror = 2,
    Clamp = 3,
    Border = 4,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RasterFormatScheme {
    // 1 bit alpha, RGB 5 bits each; also used for DXT1 with alpha
    _1555,
    // 5 bits red, 6 bits green, 5 bits blue; also used for DXT1 without alpha
    _565,
    // RGBA 4 bits each; also used for DXT3
    _4444,
    // gray scale, D3DFMT_L8
    LUM8,
    // RGBA 8 bits each
    _8888,
    // RGB 8 bits each, D3DFMT_X8R8G8B8
    _888,
    // RGB 5 bits each - rare, use 565 instead, D3DFMT_X1R5G5B5
    _555,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct RasterFormat(u32);
impl RasterFormat {
    // 1 bit alpha, RGB 5 bits each; also used for DXT1 with alpha
    const _1555: u32 = 0x0100;
    // 5 bits red, 6 bits green, 5 bits blue; also used for DXT1 without alpha
    const _565: u32 = 0x0200;
    // RGBA 4 bits each; also used for DXT3
    const _4444: u32 = 0x0300;
    // gray scale, D3DFMT_L8
    const LUM8: u32 = 0x0400;
    // RGBA 8 bits each
    const _8888: u32 = 0x0500;
    // RGB 8 bits each, D3DFMT_X8R8G8B8
    const _888: u32 = 0x0600;
    // RGB 5 bits each - rare, use 565 instead, D3DFMT_X1R5G5B5
    const _555: u32 = 0x0A00;
    #[cfg(feature = "san_andreas_support")]
    // RW generates mipmaps, see special section below
    const EXT_AUTO_MIPMAP: u32 = 0x1000;
    // 2^8 = 256 palette colors
    const EXT_PAL8: u32 = 0x2000;
    // 2^4 = 16 palette colors
    const EXT_PAL4: u32 = 0x4000;
    #[cfg(feature = "san_andreas_support")]
    // mipmaps included
    const EXT_MIPMAP: u32 = 0x8000;

    pub fn new(bits: u32) -> RasterFormat {
        Self(bits)
    }

    pub fn scheme(&self) -> RasterFormatScheme {
        if (self.0 & Self::_1555) != 0 {
            RasterFormatScheme::_1555
        } else if (self.0 & Self::_565) != 0 {
            RasterFormatScheme::_565
        } else if (self.0 & Self::_4444) != 0 {
            RasterFormatScheme::_4444
        } else if (self.0 & Self::LUM8) != 0 {
            RasterFormatScheme::LUM8
        } else if (self.0 & Self::_8888) != 0 {
            RasterFormatScheme::_8888
        } else if (self.0 & Self::_888) != 0 {
            RasterFormatScheme::_888
        } else if (self.0 & Self::_555) != 0 {
            RasterFormatScheme::_555
        } else {
            panic!("unexpected scheme")
        }
    }

    #[cfg(feature = "san_andreas_support")]
    pub fn auto_mipmap(&self) -> bool {
        (self.0 | Self::EXT_AUTO_MIPMAP) != 0
    }

    #[cfg(feature = "san_andreas_support")]
    pub fn mipmap_included(&self) -> bool {
        (self.0 | Self::EXT_MIPMAP) != 0
    }

    pub fn palette_color_count(&self) -> u16 {
        if (self.0 & Self::EXT_PAL8) != 0 {
            256
        } else if (self.0 & Self::EXT_PAL4) != 0 {
            16
        } else {
            0
        }
    }
}
impl std::fmt::Debug for RasterFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("RasterFormat");
        ds.field("scheme", &self.scheme())
            .field("palette_color_count", &self.palette_color_count());

        #[cfg(feature = "san_andreas_support")]
        {
            ds.field("auto_mipmap", &self.auto_mipmap())
                .field("mipmap_included", &self.mipmap_included())
        }

        ds.finish()
    }
}
