use super::types::*;
use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::collections::HashMap;

/// Global registry: handle -> Context
///
/// Since WASM is single-threaded, we use a custom wrapper that bypasses Sync checking.
/// This is safe because WASM will never have multiple threads.
struct SyncRefCell<T>(RefCell<T>);

// SAFETY: WASM is single-threaded, so RefCell is safe to share across "threads"
// (there are none in practice).
unsafe impl<T> Sync for SyncRefCell<T> {}

pub(crate) fn get_registry() -> &'static RefCell<Registry> {
    static REGISTRY: std::sync::OnceLock<SyncRefCell<Registry>> = std::sync::OnceLock::new();
    &REGISTRY
        .get_or_init(|| {
            SyncRefCell(RefCell::new(Registry {
                contexts: HashMap::new(),
                next_context_handle: FIRST_HANDLE,
                allocations: HashMap::new(),
            }))
        })
        .0
}

pub(crate) struct Registry {
    pub(crate) contexts: HashMap<u32, Context>,
    pub(crate) next_context_handle: u32,
    /// Track allocations created via `wasm_alloc`: ptr -> size
    pub(crate) allocations: HashMap<u32, u32>,
}

impl Registry {
    fn allocate_context_handle(&mut self) -> u32 {
        let h = self.next_context_handle;
        self.next_context_handle = self.next_context_handle.saturating_add(1);
        if self.next_context_handle == 0 {
            self.next_context_handle = FIRST_HANDLE;
        }
        h
    }
}

// ============================================================================
// Public API (exported to WASM)
// ============================================================================

/// Set last error message (internal helper)
pub fn set_last_error(msg: &str) {
    crate::error::set_error(crate::error::ErrorSource::WebGL, 0, msg);
}

/// Get pointer to last error string (UTF-8)
pub fn last_error_ptr() -> *const u8 {
    crate::error::wasm_get_last_error_msg_ptr() as *const u8
}

/// Get length of last error string
pub fn last_error_len() -> u32 {
    crate::error::wasm_get_last_error_msg_len()
}

/// Clear last error
pub(crate) fn clear_last_error() {
    crate::error::clear_error();
}

// ============================================================================
// Context Lifecycle
// ============================================================================

/// Create a new WebGL2 context with flags. Flags bits:
/// bit0 = shader debug (enable shader debug stubs).
pub fn create_context_with_flags(flags: u32, width: u32, height: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let mut ctx = Context::new(width, height);

    // Determine debug mode from flags: only shader debug is relevant here
    let shader = (flags & 0x1) != 0;

    ctx.debug_shaders = shader;

    let handle = reg.allocate_context_handle();
    reg.contexts.insert(handle, ctx);
    handle
}

/// Destroy a context by handle, freeing all its resources.
/// Returns errno (0 on success).
pub fn destroy_context(handle: u32) -> u32 {
    clear_last_error();
    if handle == INVALID_HANDLE {
        set_last_error("invalid context handle");
        return ERR_INVALID_HANDLE;
    }
    let mut reg = get_registry().borrow_mut();
    if reg.contexts.remove(&handle).is_none() {
        set_last_error("context not found");
        return ERR_INVALID_HANDLE;
    }
    ERR_OK
}

// ============================================================================
// Memory Allocation
// ============================================================================

/// Allocate memory within WASM linear memory.
/// Returns pointer (0 on failure).
pub fn wasm_alloc(size: u32) -> u32 {
    clear_last_error();
    if size == 0 {
        return 0; // Valid: allocating 0 bytes is OK but we return 0 for simplicity
    }
    let layout = match Layout::from_size_align(size as usize, 8) {
        Ok(l) => l,
        Err(_) => {
            set_last_error("allocation layout error");
            return 0;
        }
    };
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        set_last_error("out of memory");
        return 0;
    }
    let ptr_u32 = ptr as u32;
    // Record allocation size so wasm_free can deallocate later.
    {
        let mut reg = get_registry().borrow_mut();
        reg.allocations.insert(ptr_u32, size);
    }
    ptr_u32
}

/// Free memory allocated by wasm_alloc.
/// Returns errno (0 on success).
pub fn wasm_free(ptr: u32) -> u32 {
    clear_last_error();
    if ptr == 0 {
        // Freeing null is a no-op
        return ERR_OK;
    }
    // Look up allocation size
    let mut reg = get_registry().borrow_mut();
    let size = match reg.allocations.remove(&ptr) {
        Some(s) => s,
        None => {
            set_last_error("invalid or unknown allocation");
            return ERR_INVALID_ARGS;
        }
    };

    let layout = match Layout::from_size_align(size as usize, 8) {
        Ok(l) => l,
        Err(_) => {
            set_last_error("invalid allocation layout");
            return ERR_INTERNAL;
        }
    };
    unsafe { dealloc(ptr as *mut u8, layout) };
    ERR_OK
}
