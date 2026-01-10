//! Frame allocator for function call ABI.
//!
//! Provides LIFO frame allocation on a dedicated stack for passing large
//! parameters and return values via memory pointers.

use crate::naga_wasm_backend::output_layout;
use wasm_encoder::{Function, Instruction, ValType};

/// Emit WASM instructions to allocate a frame on the frame stack.
///
/// This implements the caller-side allocation sequence:
/// 1. Save current frame pointer (old_sp = global.get(FRAME_SP_GLOBAL))
/// 2. Align to required alignment (aligned = align_up(old_sp, align))
/// 3. Advance frame pointer (global.set(FRAME_SP_GLOBAL, aligned + size))
///
/// Returns the local indices for (old_sp, aligned_frame_base).
/// The caller must use `emit_free_frame` after the call completes.
///
/// # Arguments
/// - `func`: WASM function builder
/// - `size`: Total frame size in bytes
/// - `align`: Required alignment in bytes (must be power of 2)
/// - `old_sp_local`: Local index to store old SP value
/// - `aligned_local`: Local index to store aligned frame base
pub fn emit_alloc_frame(
    func: &mut Function,
    size: u32,
    align: u32,
    old_sp_local: u32,
    aligned_local: u32,
) {
    // Validate alignment is power of 2
    debug_assert!(
        align > 0 && (align & (align - 1)) == 0,
        "Alignment must be power of 2"
    );

    // 1. old_sp = global.get(FRAME_SP_GLOBAL)
    func.instruction(&Instruction::GlobalGet(output_layout::FRAME_SP_GLOBAL));
    func.instruction(&Instruction::LocalSet(old_sp_local));

    // 2. aligned = align_up(old_sp, align)
    //    aligned = (old_sp + align - 1) & ~(align - 1)
    func.instruction(&Instruction::LocalGet(old_sp_local));
    func.instruction(&Instruction::I32Const((align - 1) as i32));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::I32Const(!(align - 1) as i32));
    func.instruction(&Instruction::I32And);
    func.instruction(&Instruction::LocalSet(aligned_local));

    // 3. global.set(FRAME_SP_GLOBAL, aligned + size)
    func.instruction(&Instruction::LocalGet(aligned_local));
    func.instruction(&Instruction::I32Const(size as i32));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(output_layout::FRAME_SP_GLOBAL));
}

/// Emit WASM instructions to free a previously allocated frame.
///
/// This implements the caller-side deallocation sequence:
/// - Restore frame pointer (global.set(FRAME_SP_GLOBAL, old_sp))
///
/// Must be called after `emit_alloc_frame` and after any copy_out operations.
///
/// # Arguments
/// - `func`: WASM function builder
/// - `old_sp_local`: Local index containing saved SP value from `emit_alloc_frame`
pub fn emit_free_frame(func: &mut Function, old_sp_local: u32) {
    // Restore old frame pointer
    func.instruction(&Instruction::LocalGet(old_sp_local));
    func.instruction(&Instruction::GlobalSet(output_layout::FRAME_SP_GLOBAL));
}

/// Emit WASM instructions to store a value into the frame at a given offset.
///
/// # Arguments
/// - `func`: WASM function builder
/// - `aligned_local`: Local containing aligned frame base pointer
/// - `offset`: Offset within frame (bytes)
/// - `valtype`: Type of value to store (must match stack top)
/// - `value_local`: Local index containing the value to store
pub fn emit_frame_store(func: &mut Function, aligned_local: u32, offset: u32, valtype: ValType, value_local: u32) {
    // Compute address: aligned + offset
    func.instruction(&Instruction::LocalGet(aligned_local));
    if offset > 0 {
        func.instruction(&Instruction::I32Const(offset as i32));
        func.instruction(&Instruction::I32Add);
    }

    // Get value to store
    func.instruction(&Instruction::LocalGet(value_local));

    // Stack: [address, value]
    // Store instruction
    match valtype {
        ValType::I32 => {
            func.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                offset: 0,
                align: 2, // 4-byte alignment
                memory_index: 0,
            }));
        }
        ValType::I64 => {
            func.instruction(&Instruction::I64Store(wasm_encoder::MemArg {
                offset: 0,
                align: 3, // 8-byte alignment
                memory_index: 0,
            }));
        }
        ValType::F32 => {
            func.instruction(&Instruction::F32Store(wasm_encoder::MemArg {
                offset: 0,
                align: 2, // 4-byte alignment
                memory_index: 0,
            }));
        }
        ValType::F64 => {
            func.instruction(&Instruction::F64Store(wasm_encoder::MemArg {
                offset: 0,
                align: 3, // 8-byte alignment
                memory_index: 0,
            }));
        }
        _ => panic!("Unsupported value type for frame store"),
    }
}

/// Emit WASM instructions to load a value from the frame at a given offset.
///
/// # Arguments
/// - `func`: WASM function builder
/// - `aligned_local`: Local containing aligned frame base pointer
/// - `offset`: Offset within frame (bytes)
/// - `valtype`: Type of value to load
pub fn emit_frame_load(func: &mut Function, aligned_local: u32, offset: u32, valtype: ValType) {
    // Compute address: aligned + offset
    func.instruction(&Instruction::LocalGet(aligned_local));
    if offset > 0 {
        func.instruction(&Instruction::I32Const(offset as i32));
        func.instruction(&Instruction::I32Add);
    }

    // Load value
    match valtype {
        ValType::I32 => {
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: 0,
                align: 2, // 4-byte alignment
                memory_index: 0,
            }));
        }
        ValType::I64 => {
            func.instruction(&Instruction::I64Load(wasm_encoder::MemArg {
                offset: 0,
                align: 3, // 8-byte alignment
                memory_index: 0,
            }));
        }
        ValType::F32 => {
            func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                offset: 0,
                align: 2, // 4-byte alignment
                memory_index: 0,
            }));
        }
        ValType::F64 => {
            func.instruction(&Instruction::F64Load(wasm_encoder::MemArg {
                offset: 0,
                align: 3, // 8-byte alignment
                memory_index: 0,
            }));
        }
        _ => panic!("Unsupported value type for frame load"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_power_of_2() {
        // These should not panic in debug mode
        let mut func = Function::new(vec![]);
        emit_alloc_frame(&mut func, 16, 1, 0, 1);
        emit_alloc_frame(&mut func, 16, 2, 0, 1);
        emit_alloc_frame(&mut func, 16, 4, 0, 1);
        emit_alloc_frame(&mut func, 16, 8, 0, 1);
        emit_alloc_frame(&mut func, 16, 16, 0, 1);
    }

    #[test]
    #[should_panic(expected = "Alignment must be power of 2")]
    #[cfg(debug_assertions)]
    fn test_invalid_alignment() {
        let mut func = Function::new(vec![]);
        emit_alloc_frame(&mut func, 16, 3, 0, 1); // 3 is not power of 2
    }

    #[test]
    fn test_frame_alloc_free_sequence() {
        // Test that we generate valid instruction sequence
        let mut func = Function::new(vec![(1, ValType::I32), (1, ValType::I32)]);

        // Allocate
        emit_alloc_frame(&mut func, 32, 8, 0, 1);

        // Free
        emit_free_frame(&mut func, 0);

        // Should have generated instructions without panic
    }

    #[test]
    fn test_frame_store_load() {
        let mut func = Function::new(vec![(1, ValType::I32), (1, ValType::F32)]);

        // Store f32 from local 1
        emit_frame_store(&mut func, 0, 4, ValType::F32, 1);

        // Load f32
        emit_frame_load(&mut func, 0, 4, ValType::F32);

        // Should compile without errors
    }
}
