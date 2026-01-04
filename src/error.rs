use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSource {
    WebGL,
    WebGPU(WebGPUErrorFilter),
    System,
    Compilation,
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

    /// WebGPU: Temporary storage for the last popped error to be retrieved via FFI.
    webgpu_popped_error: Option<WasmError>,

    /// FFI: Persistent buffer for the last retrieved error message to ensure safety.
    ffi_buffer: Option<CString>,
}

thread_local! {
    static STATE: RefCell<ErrorState> = RefCell::new(ErrorState {
        webgl_last_error: None,
        webgpu_scope_stack: Vec::new(),
        webgpu_popped_error: None,
        ffi_buffer: None,
    });
}

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
                    crate::js_log(0, &format!("Uncaptured WebGPU Error: {}", error.message));
                    crate::js_dispatch_uncaptured_error(&error.message);
                }
            }
            _ => {
                // System/Other: Log immediately
                crate::js_log(0, &format!("System Error: {}", error.message));
            }
        }
    });
}

pub fn get_last_error_message() -> Option<String> {
    STATE.with(|s| {
        s.borrow()
            .webgl_last_error
            .as_ref()
            .map(|e| e.message.clone())
    })
}

pub fn get_last_error_code() -> u32 {
    STATE.with(|s| {
        s.borrow()
            .webgl_last_error
            .as_ref()
            .map(|e| e.code)
            .unwrap_or(0)
    })
}

pub fn clear_error() {
    STATE.with(|s| {
        s.borrow_mut().webgl_last_error = None;
    });
}

pub fn webgpu_push_error_scope(filter: WebGPUErrorFilter) {
    STATE.with(|s| {
        s.borrow_mut().webgpu_scope_stack.push((filter, None));
    });
}

pub fn webgpu_pop_error_scope() -> Option<WasmError> {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let error = state
            .webgpu_scope_stack
            .pop()
            .and_then(|(_filter, err)| err);
        state.webgpu_popped_error = error.clone();
        error
    })
}

#[no_mangle]
pub extern "C" fn wasm_get_last_error_code() -> u32 {
    get_last_error_code()
}

#[no_mangle]
pub extern "C" fn wasm_get_webgpu_error_msg_ptr() -> *const c_char {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(err) = &state.webgpu_popped_error {
            let c_str = CString::new(err.message.clone()).unwrap_or_default();
            let ptr = c_str.as_ptr();
            state.ffi_buffer = Some(c_str); // Keep alive
            ptr
        } else {
            std::ptr::null()
        }
    })
}

#[no_mangle]
pub extern "C" fn wasm_get_last_error_msg_ptr() -> *const c_char {
    STATE.with(|s| {
        let mut state = s.borrow_mut();

        // Prioritize WebGL error for this specific API
        if let Some(err) = &state.webgl_last_error {
            let c_str = CString::new(err.message.clone()).unwrap_or_default();
            let ptr = c_str.as_ptr();
            state.ffi_buffer = Some(c_str); // Keep alive
            ptr
        } else {
            std::ptr::null()
        }
    })
}

#[no_mangle]
pub extern "C" fn wasm_get_last_error_msg_len() -> u32 {
    STATE.with(|s| {
        s.borrow()
            .webgl_last_error
            .as_ref()
            .map(|e| e.message.len() as u32)
            .unwrap_or(0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webgl_sticky_behavior() {
        set_error(ErrorSource::WebGL, 1, "Error A");
        assert_eq!(get_last_error_message(), Some("Error A".to_string()));

        set_error(ErrorSource::WebGL, 2, "Error B");
        assert_eq!(get_last_error_message(), Some("Error B".to_string()));

        clear_error();
        assert_eq!(get_last_error_message(), None);
    }

    #[test]
    fn test_webgpu_scope_stack_basic() {
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        set_error(
            ErrorSource::WebGPU(WebGPUErrorFilter::Validation),
            1,
            "Fail",
        );

        let err = webgpu_pop_error_scope();
        assert!(err.is_some());
        assert_eq!(err.unwrap().message, "Fail");
    }

    #[test]
    fn test_webgpu_scope_stack_empty() {
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);
        let err = webgpu_pop_error_scope();
        assert!(err.is_none());
    }

    #[test]
    fn test_webgpu_nested_scopes() {
        webgpu_push_error_scope(WebGPUErrorFilter::Validation); // Scope A
        webgpu_push_error_scope(WebGPUErrorFilter::Validation); // Scope B

        set_error(
            ErrorSource::WebGPU(WebGPUErrorFilter::Validation),
            1,
            "Fail",
        );

        let err_b = webgpu_pop_error_scope();
        assert!(err_b.is_some());
        assert_eq!(err_b.unwrap().message, "Fail");

        let err_a = webgpu_pop_error_scope();
        assert!(err_a.is_none());
    }

    #[test]
    fn test_webgpu_filter_logic_bubbling() {
        webgpu_push_error_scope(WebGPUErrorFilter::OutOfMemory);
        webgpu_push_error_scope(WebGPUErrorFilter::Validation);

        set_error(
            ErrorSource::WebGPU(WebGPUErrorFilter::OutOfMemory),
            1,
            "OOM",
        );

        let err_val = webgpu_pop_error_scope(); // Validation scope
        assert!(err_val.is_none());

        let err_oom = webgpu_pop_error_scope(); // OOM scope
        assert!(err_oom.is_some());
        assert_eq!(err_oom.unwrap().message, "OOM");
    }
}
