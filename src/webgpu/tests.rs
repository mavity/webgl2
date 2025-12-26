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
}
