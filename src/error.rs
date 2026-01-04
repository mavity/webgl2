//! Unified error handling for WebGL2 and WebGPU
//!
//! This module provides a centralized error storage and reporting system that:
//! - Supports WebGL's "sticky" error model (last error persists)
//! - Supports WebGPU's error scope stack with filter-based bubbling
//! - Provides safe FFI exports for error message retrieval

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

// Log level constants
const LOG_LEVEL_ERROR: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSource {
    WebGL,
    WebGPU(WebGPUErrorFilter),
    System,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebGPUErrorFilter {
    Validation,
    OutOfMemory,
    Internal,
}

#[derive(Debug, Clone)]
pub struct WasmError {
    pub code: u32,
    pub message: String,
    pub source: ErrorSource,
}

struct ErrorState {
    /// WebGL: Single sticky error slot.
    webgl_last_error: Option<WasmError>,
    
    /// WebGPU: Stack of error scopes. 
    /// Each scope is (WebGPUErrorFilter, Option<WasmError>) capturing the filter and the *first* error in that scope.
    /// If the stack is empty, errors are "uncaptured" (logged/evented).
    webgpu_scope_stack: Vec<(WebGPUErrorFilter, Option<WasmError>)>,
    
    /// FFI: Persistent buffer for the last retrieved error message to ensure safety.
    ffi_buffer: Option<CString>,
}

thread_local! {
    static STATE: RefCell<ErrorState> = RefCell::new(ErrorState {
        webgl_last_error: None,
        webgpu_scope_stack: Vec::new(),
        ffi_buffer: None,
    });
}

/// Set an error based on the source type
/// 
/// For WebGL: overwrites the sticky error slot
/// For WebGPU: bubbles down the scope stack to find a matching filter
/// For System: logs immediately
pub fn set_error(source: ErrorSource, code: u32, msg: impl ToString) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let error = WasmError {
            code,
            message: msg.to_string(),
            source,
        };

        match source {
            ErrorSource::WebGL => {
                // WebGL: Overwrite sticky error
                state.webgl_last_error = Some(error);
            }
            ErrorSource::WebGPU(error_filter) => {
                // WebGPU: Bubble down the stack
                let mut captured = false;
                
                // Iterate from top (most recent) to bottom
                for (scope_filter, scope_error) in state.webgpu_scope_stack.iter_mut().rev() {
                    if *scope_filter == error_filter {
                        // Found a matching scope
                        if scope_error.is_none() {
                            *scope_error = Some(error.clone());
                        }
                        captured = true;
                        break; // Stop bubbling once captured
                    }
                }

                if !captured {
                    // Uncaptured error: Log immediately or trigger global handler
                    crate::js_log(LOG_LEVEL_ERROR, &format!("Uncaptured WebGPU Error: {}", error.message));
                }
            }
            _ => {
                // System/Other: Log immediately
                crate::js_log(LOG_LEVEL_ERROR, &format!("System Error: {}", error.message));
            }
        }
    });
}

/// Push a new error scope onto the WebGPU stack
pub fn webgpu_push_error_scope(filter: WebGPUErrorFilter) {
    STATE.with(|s| {
        s.borrow_mut().webgpu_scope_stack.push((filter, None));
    });
}

/// Pop an error scope from the WebGPU stack, returning any captured error
pub fn webgpu_pop_error_scope() -> Option<WasmError> {
    STATE.with(|s| {
        s.borrow_mut().webgpu_scope_stack.pop().and_then(|(_filter, err)| err)
    })
}

/// Get the last WebGL error message (internal use)
pub fn get_last_webgl_error() -> Option<WasmError> {
    STATE.with(|s| {
        s.borrow().webgl_last_error.clone()
    })
}

/// Clear the last WebGL error
pub fn clear_webgl_error() {
    STATE.with(|s| {
        s.borrow_mut().webgl_last_error = None;
    });
}

/// FFI export: Get pointer to last WebGL error message
#[no_mangle]
pub extern "C" fn wasm_get_last_error_msg_ptr() -> *const c_char {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        
        // Get WebGL error for this specific API
        if let Some(err) = &state.webgl_last_error {
            let c_str = CString::new(err.message.clone())
                .unwrap_or_else(|_| CString::default());
            let ptr = c_str.as_ptr();
            state.ffi_buffer = Some(c_str); // Keep alive
            ptr
        } else {
            std::ptr::null()
        }
    })
}

/// FFI export: Get length of last WebGL error message
#[no_mangle]
pub extern "C" fn wasm_get_last_error_msg_len() -> usize {
    STATE.with(|s| {
        s.borrow().webgl_last_error.as_ref()
            .map(|e| e.message.len())
            .unwrap_or(0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to reset error state between tests
    fn reset_state() {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.webgl_last_error = None;
            state.webgpu_scope_stack.clear();
            state.ffi_buffer = None;
        });
    }

    #[test]
    fn test_webgl_sticky_behavior() {
        reset_state();
        
        // Set first error
        set_error(ErrorSource::WebGL, 1, "Error A");
        let err = get_last_webgl_error();
        assert!(err.is_some());
        assert_eq!(err.as_ref().unwrap().message, "Error A");
        
        // Set second error - should overwrite
        set_error(ErrorSource::WebGL, 2, "Error B");
        let err = get_last_webgl_error();
        assert!(err.is_some());
        assert_eq!(err.as_ref().unwrap().message, "Error B");
        
        // Clear error
        clear_webgl_error();
        assert!(get_last_webgl_error().is_none());
    }

    #[test]
    fn test_webgpu_scope_stack_basic() {
        reset_state();
        
        // Push a scope
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        
        // Set an error
        set_error(ErrorSource::WebGPU(WebGPUErrorFilter::Validation), 1, "Validation Failed");
        
        // Pop the scope
        let err = webgpu_pop_error_scope();
        assert!(err.is_some());
        assert_eq!(err.as_ref().unwrap().message, "Validation Failed");
    }

    #[test]
    fn test_webgpu_scope_stack_empty() {
        reset_state();
        
        // Push a scope but don't trigger any errors
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        
        // Pop should return None
        let err = webgpu_pop_error_scope();
        assert!(err.is_none());
    }

    #[test]
    fn test_webgpu_nested_scopes() {
        reset_state();
        
        // Push two scopes
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        
        // Set an error - should be captured by inner scope
        set_error(ErrorSource::WebGPU(WebGPUErrorFilter::Validation), 1, "Error in inner scope");
        
        // Pop inner scope
        let err = webgpu_pop_error_scope();
        assert!(err.is_some());
        assert_eq!(err.as_ref().unwrap().message, "Error in inner scope");
        
        // Pop outer scope - should be empty
        let err = webgpu_pop_error_scope();
        assert!(err.is_none());
    }

    #[test]
    fn test_webgpu_filter_bubbling() {
        reset_state();
        
        // Push two scopes with different filters
        webgpu_push_error_scope(WebGPUErrorFilter::OutOfMemory);
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        
        // Set an OutOfMemory error - should bubble past Validation to OutOfMemory
        set_error(ErrorSource::WebGPU(WebGPUErrorFilter::OutOfMemory), 1, "OOM Error");
        
        // Pop Validation scope - should be empty
        let err = webgpu_pop_error_scope();
        assert!(err.is_none());
        
        // Pop OutOfMemory scope - should have the error
        let err = webgpu_pop_error_scope();
        assert!(err.is_some());
        assert_eq!(err.as_ref().unwrap().message, "OOM Error");
    }

    #[test]
    fn test_webgpu_first_error_only() {
        reset_state();
        
        // Push a scope
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        
        // Set first error
        set_error(ErrorSource::WebGPU(WebGPUErrorFilter::Validation), 1, "First Error");
        
        // Set second error - should be ignored
        set_error(ErrorSource::WebGPU(WebGPUErrorFilter::Validation), 2, "Second Error");
        
        // Pop should return only the first error
        let err = webgpu_pop_error_scope();
        assert!(err.is_some());
        assert_eq!(err.as_ref().unwrap().message, "First Error");
    }
}
