//! WebGPU Texture management

use super::adapter::with_context;
use wgpu_types as wgt;

pub struct TextureConfig {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: u32,
    pub format: u32,
    pub usage: u32,
}

/// Create a new texture
pub fn create_texture(ctx_handle: u32, device_handle: u32, config: TextureConfig) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let dimension = match config.dimension {
            0 => wgt::TextureDimension::D1,
            1 => wgt::TextureDimension::D2,
            2 => wgt::TextureDimension::D3,
            _ => return super::NULL_HANDLE,
        };

        // TODO: Map integer format to wgt::TextureFormat
        let format = match config.format {
            0 => wgt::TextureFormat::R8Unorm,
            1 => wgt::TextureFormat::R8Snorm,
            2 => wgt::TextureFormat::R8Uint,
            3 => wgt::TextureFormat::R8Sint,
            12 => wgt::TextureFormat::R16Float,
            17 => wgt::TextureFormat::Rgba8Unorm,
            18 => wgt::TextureFormat::Rgba8UnormSrgb,
            19 => wgt::TextureFormat::Bgra8Unorm,
            20 => wgt::TextureFormat::Bgra8UnormSrgb,
            24 => wgt::TextureFormat::Rgba16Float,
            35 => wgt::TextureFormat::R32Float,
            38 => wgt::TextureFormat::Depth32Float,
            39 => wgt::TextureFormat::Depth24Plus,
            40 => wgt::TextureFormat::Depth24PlusStencil8,
            _ => wgt::TextureFormat::Rgba8Unorm, // Default
        };

        let desc = wgt::TextureDescriptor {
            label: None,
            size: wgt::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: config.depth_or_array_layers,
            },
            mip_level_count: config.mip_level_count,
            sample_count: config.sample_count,
            dimension,
            format,
            usage: wgt::TextureUsages::from_bits_truncate(config.usage),
            view_formats: vec![],
        };

        let (texture_id, error) = ctx.global.device_create_texture(device_id, &desc, None);
        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e,
            );
            // TODO: Return poisoned handle
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_texture_id;
        ctx.next_texture_id += 1;
        ctx.textures.insert(handle, texture_id);

        handle
    })
}

pub struct TextureViewConfig {
    pub format: u32,
    pub dimension: u32,
    pub base_mip_level: u32,
    pub mip_level_count: u32,
    pub base_array_layer: u32,
    pub array_layer_count: u32,
    pub aspect: u32,
}

pub struct SamplerConfig {
    pub address_mode_u: u32,
    pub address_mode_v: u32,
    pub address_mode_w: u32,
    pub mag_filter: u32,
    pub min_filter: u32,
    pub mipmap_filter: u32,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: u32,
    pub max_anisotropy: u16,
}

/// Create a texture view
pub fn create_texture_view(ctx_handle: u32, texture_handle: u32, config: TextureViewConfig) -> u32 {
    with_context(ctx_handle, |ctx| {
        let texture_id = match ctx.textures.get(&texture_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let format = if config.format == 0 {
            None
        } else {
            Some(wgt::TextureFormat::Rgba8Unorm) // TODO: Map format
        };

        let dimension = match config.dimension {
            1 => Some(wgt::TextureViewDimension::D1),
            2 => Some(wgt::TextureViewDimension::D2),
            3 => Some(wgt::TextureViewDimension::D2Array),
            4 => Some(wgt::TextureViewDimension::Cube),
            5 => Some(wgt::TextureViewDimension::CubeArray),
            6 => Some(wgt::TextureViewDimension::D3),
            _ => None,
        };

        let aspect = match config.aspect {
            0 => wgt::TextureAspect::All,
            1 => wgt::TextureAspect::StencilOnly,
            2 => wgt::TextureAspect::DepthOnly,
            _ => wgt::TextureAspect::All,
        };

        let desc = wgpu_core::resource::TextureViewDescriptor {
            label: None,
            format,
            dimension,
            usage: None, // Inherit from texture
            range: wgt::ImageSubresourceRange {
                aspect,
                base_mip_level: config.base_mip_level,
                mip_level_count: if config.mip_level_count == 0 {
                    None
                } else {
                    Some(config.mip_level_count)
                },
                base_array_layer: config.base_array_layer,
                array_layer_count: if config.array_layer_count == 0 {
                    None
                } else {
                    Some(config.array_layer_count)
                },
            },
        };

        let (view_id, error) = ctx.global.texture_create_view(texture_id, &desc, None);
        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e,
            );
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_texture_view_id;
        ctx.next_texture_view_id += 1;
        ctx.texture_views.insert(handle, view_id);

        handle
    })
}

/// Create a sampler
pub fn create_sampler(ctx_handle: u32, device_handle: u32, config: SamplerConfig) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let address_mode = |mode| match mode {
            0 => wgt::AddressMode::ClampToEdge,
            1 => wgt::AddressMode::Repeat,
            2 => wgt::AddressMode::MirrorRepeat,
            _ => wgt::AddressMode::ClampToEdge,
        };

        let filter_mode = |mode| match mode {
            0 => wgt::FilterMode::Nearest,
            1 => wgt::FilterMode::Linear,
            _ => wgt::FilterMode::Nearest,
        };

        let mip_filter_mode = |mode| match mode {
            0 => wgt::MipmapFilterMode::Nearest,
            1 => wgt::MipmapFilterMode::Linear,
            _ => wgt::MipmapFilterMode::Nearest,
        };

        let compare = if config.compare == 0 {
            None
        } else {
            Some(match config.compare - 1 {
                0 => wgt::CompareFunction::Never,
                1 => wgt::CompareFunction::Less,
                2 => wgt::CompareFunction::Equal,
                3 => wgt::CompareFunction::LessEqual,
                4 => wgt::CompareFunction::Greater,
                5 => wgt::CompareFunction::NotEqual,
                6 => wgt::CompareFunction::GreaterEqual,
                7 => wgt::CompareFunction::Always,
                _ => wgt::CompareFunction::Never,
            })
        };

        let desc = wgpu_core::resource::SamplerDescriptor {
            label: None,
            address_modes: [
                address_mode(config.address_mode_u),
                address_mode(config.address_mode_v),
                address_mode(config.address_mode_w),
            ],
            mag_filter: filter_mode(config.mag_filter),
            min_filter: filter_mode(config.min_filter),
            mipmap_filter: mip_filter_mode(config.mipmap_filter),
            lod_min_clamp: config.lod_min_clamp,
            lod_max_clamp: config.lod_max_clamp,
            compare,
            anisotropy_clamp: config.max_anisotropy,
            border_color: None,
        };

        let (sampler_id, error) = ctx.global.device_create_sampler(device_id, &desc, None);

        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e,
            );
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_sampler_id;
        ctx.next_sampler_id += 1;
        ctx.samplers.insert(handle, sampler_id);

        handle
    })
}

/// Destroy a texture
pub fn destroy_texture(ctx_handle: u32, texture_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        if let Some(id) = ctx.textures.remove(&texture_handle) {
            ctx.global.texture_destroy(id);
            super::WEBGPU_SUCCESS
        } else {
            super::WEBGPU_ERROR_INVALID_HANDLE
        }
    })
}
