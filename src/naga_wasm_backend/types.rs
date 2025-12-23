//! Type system mapping between Naga IR and WASM

use super::BackendError;
use naga::{ScalarKind, TypeInner, VectorSize};
use wasm_encoder::ValType;

/// Map a Naga scalar type to WASM value type
pub fn scalar_to_wasm(kind: ScalarKind, _width: u8) -> Result<ValType, BackendError> {
    match kind {
        ScalarKind::Sint => Ok(ValType::I32),
        ScalarKind::Uint => Ok(ValType::I32),
        ScalarKind::Float => Ok(ValType::F32),
        ScalarKind::Bool => Ok(ValType::I32), // Booleans as i32 (0 or 1)
        ScalarKind::AbstractInt => Ok(ValType::I32),
        ScalarKind::AbstractFloat => Ok(ValType::F32),
    }
}

/// Map a Naga type to WASM value types
pub fn naga_to_wasm_types(type_inner: &TypeInner) -> Result<Vec<ValType>, BackendError> {
    match type_inner {
        TypeInner::Scalar(scalar) => Ok(vec![scalar_to_wasm(scalar.kind, scalar.width)?]),
        TypeInner::Vector { size, scalar } => {
            let val_type = scalar_to_wasm(scalar.kind, scalar.width)?;
            let count = vector_component_count(*size) as usize;
            Ok(vec![val_type; count])
        }
        TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => {
            let val_type = scalar_to_wasm(scalar.kind, scalar.width)?;
            let count = (vector_component_count(*columns) * vector_component_count(*rows)) as usize;
            Ok(vec![val_type; count])
        }
        TypeInner::Pointer { .. } => Ok(vec![ValType::I32]),
        TypeInner::Image { .. } => Ok(vec![ValType::I32]),
        TypeInner::Sampler { .. } => Ok(vec![ValType::I32]),
        _ => Err(BackendError::UnsupportedFeature(format!(
            "Unsupported Naga type for WASM: {:?}",
            type_inner
        ))),
    }
}

/// Get the number of components in a type
pub fn component_count(type_inner: &TypeInner) -> u32 {
    match type_inner {
        TypeInner::Scalar(_) => 1,
        TypeInner::Vector { size, .. } => vector_component_count(*size),
        TypeInner::Matrix { columns, rows, .. } => {
            vector_component_count(*columns) * vector_component_count(*rows)
        }
        TypeInner::Array {
            size: naga::ArraySize::Constant(count),
            ..
        } => count.get(),
        TypeInner::Array { .. } => 1,
        _ => 1,
    }
}

/// Get the number of components in a vector
pub fn vector_component_count(size: VectorSize) -> u32 {
    match size {
        VectorSize::Bi => 2,
        VectorSize::Tri => 3,
        VectorSize::Quad => 4,
    }
}

/// Calculate the size in bytes of a Naga type
pub fn type_size(type_inner: &TypeInner) -> Result<u32, BackendError> {
    match type_inner {
        TypeInner::Scalar(scalar) => Ok(scalar.width as u32),
        TypeInner::Vector { size, scalar } => {
            let components = vector_component_count(*size);
            Ok(components * scalar.width as u32)
        }
        TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => {
            let col_count = vector_component_count(*columns);
            let row_count = vector_component_count(*rows);
            Ok(col_count * row_count * scalar.width as u32)
        }
        TypeInner::Array {
            base: _,
            size,
            stride,
        } => match size {
            naga::ArraySize::Constant(count) => Ok(count.get() * stride),
            naga::ArraySize::Dynamic => Err(BackendError::TypeConversion(
                "Dynamic arrays not yet supported".to_string(),
            )),
            naga::ArraySize::Pending(_) => Err(BackendError::TypeConversion(
                "Pending array sizes (overrides) not yet supported".to_string(),
            )),
        },
        TypeInner::Struct { span, .. } => Ok(*span),
        _ => Err(BackendError::TypeConversion(format!(
            "Unsupported type for size calculation: {:?}",
            type_inner
        ))),
    }
}

/// Represents how a type is stored in WASM
#[derive(Debug, Clone)]
pub enum WasmTypeLayout {
    /// Stored as WASM locals (for small types like scalars and small vectors)
    Locals(Vec<ValType>),
    /// Stored in linear memory (for large types like matrices and structs)
    Memory { offset: u32, size: u32 },
}

/// Determine the WASM storage layout for a Naga type
pub fn type_layout(type_inner: &TypeInner) -> Result<WasmTypeLayout, BackendError> {
    match type_inner {
        TypeInner::Scalar(scalar) => {
            let val_type = scalar_to_wasm(scalar.kind, scalar.width)?;
            Ok(WasmTypeLayout::Locals(vec![val_type]))
        }
        TypeInner::Vector { size, scalar } => {
            let val_type = scalar_to_wasm(scalar.kind, scalar.width)?;
            let count = vector_component_count(*size) as usize;
            Ok(WasmTypeLayout::Locals(vec![val_type; count]))
        }
        TypeInner::Matrix { .. } => {
            // Matrices are stored in memory for now (optimization: small matrices as locals)
            let size = type_size(type_inner)?;
            Ok(WasmTypeLayout::Memory { offset: 0, size })
        }
        TypeInner::Struct { .. } => {
            let size = type_size(type_inner)?;
            Ok(WasmTypeLayout::Memory { offset: 0, size })
        }
        _ => Err(BackendError::TypeConversion(format!(
            "Unsupported type layout: {:?}",
            type_inner
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use naga::Scalar;

    #[test]
    fn test_scalar_to_wasm() {
        assert_eq!(scalar_to_wasm(ScalarKind::Float, 4).unwrap(), ValType::F32);
        assert_eq!(scalar_to_wasm(ScalarKind::Sint, 4).unwrap(), ValType::I32);
        assert_eq!(scalar_to_wasm(ScalarKind::Bool, 1).unwrap(), ValType::I32);
    }

    #[test]
    fn test_vector_component_count() {
        assert_eq!(vector_component_count(VectorSize::Bi), 2);
        assert_eq!(vector_component_count(VectorSize::Tri), 3);
        assert_eq!(vector_component_count(VectorSize::Quad), 4);
    }

    #[test]
    fn test_type_size() {
        let float_type = TypeInner::Scalar(Scalar::F32);
        assert_eq!(type_size(&float_type).unwrap(), 4);

        let vec3_type = TypeInner::Vector {
            size: VectorSize::Tri,
            scalar: Scalar::F32,
        };
        assert_eq!(type_size(&vec3_type).unwrap(), 12);
    }
}
