//! WebGPU Texture management

use super::adapter::with_context;
use wgpu_types as wgt;

/// Create a new texture
pub fn create_texture(
    ctx_handle: u32,
    device_handle: u32,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    mip_level_count: u32,
    sample_count: u32,
    dimension: u32, // wgt::TextureDimension
    format: u32,    // wgt::TextureFormat
    usage: u32,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let dimension = match dimension {
            0 => wgt::TextureDimension::D1,
            1 => wgt::TextureDimension::D2,
            2 => wgt::TextureDimension::D3,
            _ => return super::NULL_HANDLE,
        };

        // TODO: Map integer format to wgt::TextureFormat
        // For now, assume Rgba8Unorm (17) if format is 17, else panic or default
        let format = wgt::TextureFormat::Rgba8Unorm; 

        let desc = wgt::TextureDescriptor {
            label: None,
            size: wgt::Extent3d {
                width,
                height,
                depth_or_array_layers,
            },
            mip_level_count,
            sample_count,
            dimension,
            format,
            usage: wgt::TextureUsages::from_bits_truncate(usage),
            view_formats: vec![],
        };

        let (texture_id, error) = ctx.global.device_create_texture(device_id, &desc, None);
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to create texture: {:?}", e));
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_texture_id;
        ctx.next_texture_id += 1;
        ctx.textures.insert(handle, texture_id);

        handle
    })
}

/// Create a texture view
pub fn create_texture_view(
    ctx_handle: u32,
    texture_handle: u32,
    format: u32, // 0 = undefined/inherit
    dimension: u32, // 0 = undefined/inherit
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
    aspect: u32, // wgt::TextureAspect
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let texture_id = match ctx.textures.get(&texture_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let format = if format == 0 {
            None
        } else {
            Some(wgt::TextureFormat::Rgba8Unorm) // TODO: Map format
        };

        let dimension = match dimension {
            1 => Some(wgt::TextureViewDimension::D1),
            2 => Some(wgt::TextureViewDimension::D2),
            3 => Some(wgt::TextureViewDimension::D2Array),
            4 => Some(wgt::TextureViewDimension::Cube),
            5 => Some(wgt::TextureViewDimension::CubeArray),
            6 => Some(wgt::TextureViewDimension::D3),
            _ => None,
        };

        let aspect = match aspect {
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
                base_mip_level,
                mip_level_count: if mip_level_count == 0 { None } else { Some(mip_level_count) },
                base_array_layer,
                array_layer_count: if array_layer_count == 0 { None } else { Some(array_layer_count) },
            },
        };

        let (view_id, error) = ctx.global.texture_create_view(texture_id, &desc, None);
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to create texture view: {:?}", e));
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_texture_view_id;
        ctx.next_texture_view_id += 1;
        ctx.texture_views.insert(handle, view_id);

        handle
    })
}

/// Create a sampler
pub fn create_sampler(
    ctx_handle: u32,
    device_handle: u32,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let desc = wgpu_core::resource::SamplerDescriptor {
            label: None,
            address_modes: [
                wgt::AddressMode::ClampToEdge,
                wgt::AddressMode::ClampToEdge,
                wgt::AddressMode::ClampToEdge,
            ],
            mag_filter: wgt::FilterMode::Linear,
            min_filter: wgt::FilterMode::Linear,
            mipmap_filter: wgt::MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        };

        let (sampler_id, error) = ctx
            .global
            .device_create_sampler(device_id, &desc, None);
            
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to create sampler: {:?}", e));
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
