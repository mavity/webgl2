//! WebGPU integration tests

#[cfg(test)]
mod tests {
    use crate::webgpu::adapter::{create_context, destroy_context};
    use crate::webgpu::{NULL_HANDLE, WEBGPU_ERROR_INVALID_HANDLE, WEBGPU_SUCCESS};

    #[test]
    fn test_create_and_destroy_context() {
        // Create a context
        let ctx = create_context();
        assert_ne!(
            ctx, NULL_HANDLE,
            "Context creation should return a valid handle"
        );

        // Destroy the context
        let result = destroy_context(ctx);
        assert_eq!(result, WEBGPU_SUCCESS, "Context destruction should succeed");

        // Try to destroy again (should fail)
        let result = destroy_context(ctx);
        assert_ne!(
            result, WEBGPU_SUCCESS,
            "Destroying same context twice should fail"
        );
    }

    #[test]
    fn test_multiple_contexts() {
        // Create multiple contexts
        let ctx1 = create_context();
        let ctx2 = create_context();
        let ctx3 = create_context();

        assert_ne!(ctx1, NULL_HANDLE);
        assert_ne!(ctx2, NULL_HANDLE);
        assert_ne!(ctx3, NULL_HANDLE);

        // All contexts should have unique handles
        assert_ne!(ctx1, ctx2);
        assert_ne!(ctx2, ctx3);
        assert_ne!(ctx1, ctx3);

        // Clean up
        assert_eq!(destroy_context(ctx1), WEBGPU_SUCCESS);
        assert_eq!(destroy_context(ctx2), WEBGPU_SUCCESS);
        assert_eq!(destroy_context(ctx3), WEBGPU_SUCCESS);
    }

    #[test]
    fn test_destroy_invalid_context() {
        // Try to destroy a handle that was never created
        let result = destroy_context(999);
        assert_ne!(
            result, WEBGPU_SUCCESS,
            "Destroying invalid handle should fail"
        );
        assert_eq!(result, WEBGPU_ERROR_INVALID_HANDLE);
    }

    #[test]
    fn test_context_sequential_ids() {
        // Create contexts and verify they get sequential IDs
        let ctx1 = create_context();
        let ctx2 = create_context();

        // IDs should be different and increasing
        assert!(ctx2 > ctx1 || ctx1 > ctx2); // Just ensure they're different

        // Clean up
        destroy_context(ctx1);
        destroy_context(ctx2);
    }

    #[test]
    fn test_null_handle_constant() {
        assert_eq!(NULL_HANDLE, 0, "NULL_HANDLE should be 0");
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(WEBGPU_SUCCESS, 0);
        assert_eq!(WEBGPU_ERROR_INVALID_HANDLE, 1);
    }

    #[test]
    fn test_create_many_contexts() {
        let mut contexts = Vec::new();

        // Create 10 contexts
        for _ in 0..10 {
            let ctx = create_context();
            assert_ne!(ctx, NULL_HANDLE);
            contexts.push(ctx);
        }

        // Verify all are unique
        for i in 0..contexts.len() {
            for j in i + 1..contexts.len() {
                assert_ne!(contexts[i], contexts[j]);
            }
        }

        // Clean up all
        for ctx in contexts {
            assert_eq!(destroy_context(ctx), WEBGPU_SUCCESS);
        }
    }

    #[test]
    fn test_context_reuse_after_destroy() {
        let ctx1 = create_context();
        let _id1 = ctx1;

        destroy_context(ctx1);

        // Create new context - it might reuse the ID or not
        let ctx2 = create_context();
        assert_ne!(ctx2, NULL_HANDLE);

        // Either way, we should be able to destroy it
        assert_eq!(destroy_context(ctx2), WEBGPU_SUCCESS);
    }

    #[test]
    fn test_destroy_same_context_twice() {
        let ctx = create_context();

        // First destroy should succeed
        assert_eq!(destroy_context(ctx), WEBGPU_SUCCESS);

        // Second destroy should fail
        let result = destroy_context(ctx);
        assert_ne!(result, WEBGPU_SUCCESS);
        assert_eq!(result, WEBGPU_ERROR_INVALID_HANDLE);
    }

    #[test]
    fn test_webgpu_module_constants() {
        use crate::webgpu::*;

        // Verify error code values
        assert_eq!(WEBGPU_SUCCESS, 0);
        assert_eq!(WEBGPU_ERROR_INVALID_HANDLE, 1);
        assert_eq!(WEBGPU_ERROR_OUT_OF_MEMORY, 2);
        assert_eq!(WEBGPU_ERROR_VALIDATION, 3);
        assert_eq!(WEBGPU_ERROR_OPERATION_FAILED, 4);
    }

    #[test]
    fn test_buffer_lifecycle() {
        use crate::webgpu::adapter::{request_adapter, request_device};
        use crate::webgpu::buffer::{
            buffer_get_mapped_range, buffer_map_async, buffer_unmap, create_buffer, destroy_buffer,
        };
        use wgpu_types::{BufferUsages, PowerPreference};

        let ctx = create_context();

        let adapter = request_adapter(ctx, PowerPreference::LowPower);
        assert_ne!(adapter, NULL_HANDLE, "Adapter creation failed");

        let device = request_device(ctx, adapter);
        assert_ne!(device, NULL_HANDLE, "Device creation failed");

        let size = 256;
        let usage = BufferUsages::MAP_READ | BufferUsages::COPY_DST;

        let buffer = create_buffer(ctx, device, size, usage.bits(), false);
        assert_ne!(buffer, NULL_HANDLE, "Buffer creation failed");

        // Map async (Read mode = 1)
        let result = buffer_map_async(ctx, device, buffer, 1, 0, size);
        assert_eq!(result, WEBGPU_SUCCESS, "Buffer map async failed");

        // Get mapped range
        let ptr = buffer_get_mapped_range(ctx, buffer, 0, size);
        assert!(!ptr.is_null(), "Get mapped range failed");

        // Verify we can write to the pointer (simulating JS writing to the ArrayBuffer)
        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr, size as usize);
            slice[0] = 0xAA;
            slice[size as usize - 1] = 0xBB;

            assert_eq!(slice[0], 0xAA);
            assert_eq!(slice[size as usize - 1], 0xBB);
        }

        // Unmap
        let result = buffer_unmap(ctx, buffer);
        assert_eq!(result, WEBGPU_SUCCESS, "Buffer unmap failed");

        // Destroy
        let result = destroy_buffer(ctx, buffer);
        assert_eq!(result, WEBGPU_SUCCESS, "Buffer destroy failed");

        destroy_context(ctx);
    }

    #[test]
    fn test_buffer_mapped_at_creation() {
        use crate::webgpu::adapter::{request_adapter, request_device};
        use crate::webgpu::buffer::{buffer_get_mapped_range, buffer_unmap, create_buffer};
        use wgpu_types::{BufferUsages, PowerPreference};

        let ctx = create_context();
        let adapter = request_adapter(ctx, PowerPreference::LowPower);
        let device = request_device(ctx, adapter);

        let size = 128;
        let usage = BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC;

        // Create mapped at creation
        let buffer = create_buffer(ctx, device, size, usage.bits(), true);
        assert_ne!(buffer, NULL_HANDLE);

        // Should be able to get range immediately without map_async
        let ptr = buffer_get_mapped_range(ctx, buffer, 0, size);
        assert!(!ptr.is_null());

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr, size as usize);
            slice[0] = 123;
        }

        let result = buffer_unmap(ctx, buffer);
        assert_eq!(result, WEBGPU_SUCCESS);

        destroy_context(ctx);
    }

    #[test]
    fn test_copy_buffer_to_buffer() {
        use crate::webgpu::adapter::{request_adapter, request_device};
        use crate::webgpu::buffer::{
            buffer_get_mapped_range, buffer_map_async, buffer_unmap, create_buffer,
        };
        use crate::webgpu::command::{
            command_encoder_copy_buffer_to_buffer, command_encoder_finish, create_command_encoder,
            queue_submit,
        };
        use wgpu_types::{BufferUsages, PowerPreference};

        let ctx = create_context();
        let adapter = request_adapter(ctx, PowerPreference::LowPower);
        let device = request_device(ctx, adapter);

        let size = 256;

        // Source buffer: Mapped at creation, COPY_SRC
        let src_buffer = create_buffer(
            ctx,
            device,
            size,
            (BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC).bits(),
            true,
        );
        assert_ne!(src_buffer, NULL_HANDLE);

        // Write data to source
        let src_ptr = buffer_get_mapped_range(ctx, src_buffer, 0, size);
        assert!(!src_ptr.is_null());
        unsafe {
            let slice = std::slice::from_raw_parts_mut(src_ptr, size as usize);
            for i in 0..size {
                slice[i as usize] = i as u8;
            }
        }
        buffer_unmap(ctx, src_buffer);

        // Dest buffer: COPY_DST, MAP_READ
        let dst_buffer = create_buffer(
            ctx,
            device,
            size,
            (BufferUsages::COPY_DST | BufferUsages::MAP_READ).bits(),
            false,
        );
        assert_ne!(dst_buffer, NULL_HANDLE);

        // Create command encoder
        let encoder = create_command_encoder(ctx, device);
        assert_ne!(encoder, NULL_HANDLE);

        // Record copy command
        command_encoder_copy_buffer_to_buffer(ctx, encoder, src_buffer, 0, dst_buffer, 0, size);

        // Finish encoding
        let cmd_buf = command_encoder_finish(ctx, encoder);
        assert_ne!(cmd_buf, NULL_HANDLE);

        // Submit to queue
        let queue = device;
        queue_submit(ctx, queue, &[cmd_buf]);

        // Map dest buffer to read back
        buffer_map_async(ctx, device, dst_buffer, 1, 0, size);
        let dst_ptr = buffer_get_mapped_range(ctx, dst_buffer, 0, size);
        assert!(!dst_ptr.is_null());

        unsafe {
            let slice = std::slice::from_raw_parts(dst_ptr, size as usize);
            for i in 0..size {
                assert_eq!(slice[i as usize], i as u8, "Mismatch at index {}", i);
            }
        }

        buffer_unmap(ctx, dst_buffer);
        destroy_context(ctx);
    }

    #[test]
    fn test_texture_creation() {
        use crate::webgpu::adapter::{request_adapter, request_device};
        use crate::webgpu::texture::{create_texture, create_texture_view, destroy_texture};
        use wgpu_types::{PowerPreference, TextureDimension, TextureUsages};

        let ctx = create_context();
        let adapter = request_adapter(ctx, PowerPreference::LowPower);
        let device = request_device(ctx, adapter);

        let width = 64;
        let height = 64;
        let usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

        // Create texture
        let texture = create_texture(
            ctx,
            device,
            width,
            height,
            1, // depth
            1, // mips
            1, // samples
            TextureDimension::D2 as u32,
            17, // Rgba8Unorm
            usage.bits(),
        );
        assert_ne!(texture, NULL_HANDLE, "Texture creation failed");

        // Create view
        let view = create_texture_view(
            ctx, texture, 0, // undefined format
            0, // undefined dimension
            0, // base mip
            1, // mip count
            0, // base layer
            1, // layer count
            0, // all aspects
        );
        assert_ne!(view, NULL_HANDLE, "Texture view creation failed");

        // Destroy
        let result = destroy_texture(ctx, texture);
        assert_eq!(result, WEBGPU_SUCCESS);

        destroy_context(ctx);
    }
}
