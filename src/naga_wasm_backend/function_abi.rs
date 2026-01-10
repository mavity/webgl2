//! Function ABI: deterministic classification and layout for function signatures.
//!
//! This module provides a pure, testable facility to classify parameters and results
//! as either Flattened (passed as WASM scalar values) or Frame (passed via memory pointer).

use naga::{AddressSpace, ArraySize, Handle, Module, ScalarKind, Type, TypeInner};
use wasm_encoder::ValType;

/// Maximum bytes for a parameter/result to be flattened into scalar values.
/// Above this threshold, we use Frame passing.
const MAX_FLATTEN_BYTES: u32 = 16;

/// Maximum number of flattened scalar parameters to prevent excessive stack usage.
const MAX_PARAM_COUNT: usize = 16;

/// ABI information for a function signature.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionABI {
    pub params: Vec<ParameterABI>,
    pub result: Option<ResultABI>,
    pub uses_frame: bool,
    pub frame_size: u32,
    pub frame_alignment: u32,
}

/// How a parameter is passed.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterABI {
    /// Passed as flattened WASM scalar values.
    Flattened {
        valtypes: Vec<ValType>,
        byte_size: u32,
    },
    /// Passed via memory frame pointer.
    Frame {
        offset: u32,
        size: u32,
        align: u32,
        copy_in: bool,
        copy_out: bool,
        semantic: ParamSemantic,
    },
}

/// How a result is returned.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultABI {
    /// Returned as flattened WASM scalar values.
    Flattened {
        valtypes: Vec<ValType>,
        byte_size: u32,
    },
    /// Returned via memory frame pointer (caller allocates space).
    Frame { size: u32, align: u32 },
}

/// Parameter semantic for Frame passing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamSemantic {
    In,
    Out,
    InOut,
}

impl FunctionABI {
    /// Compute the ABI for a function given its parameter types and optional result type.
    pub fn compute(
        module: &Module,
        param_types: &[Handle<Type>],
        result_type: Option<Handle<Type>>,
    ) -> Result<Self, ABIError> {
        let mut params = Vec::new();
        let mut frame_offset = 0u32;
        let mut frame_alignment = 1u32;
        let mut uses_frame = false;
        let mut total_flattened = 0;

        // Classify parameters
        for &param_ty in param_types {
            // Detect parameter semantic from type encoding
            let (semantic, actual_ty) = match &module.types[param_ty].inner {
                TypeInner::Pointer { base, space } if *space == AddressSpace::Function => {
                    // out/inout parameter - wrapped in pointer
                    // Conservative: treat all Function-space pointers as InOut
                    (ParamSemantic::InOut, *base)
                }
                _ => {
                    // in parameter - value type
                    (ParamSemantic::In, param_ty)
                }
            };

            let classification = classify_type(module, actual_ty)?;

            match classification {
                TypeClass::Flattened(valtypes, byte_size) => {
                    total_flattened += valtypes.len();
                    if total_flattened > MAX_PARAM_COUNT {
                        return Err(ABIError::TooManyParameters);
                    }
                    params.push(ParameterABI::Flattened {
                        valtypes,
                        byte_size,
                    });
                }
                TypeClass::Frame(size, align) => {
                    uses_frame = true;
                    frame_alignment = frame_alignment.max(align);

                    // Align offset
                    frame_offset = align_up(frame_offset, align);

                    // Set copy flags based on semantic
                    let (copy_in, copy_out) = match semantic {
                        ParamSemantic::In => (true, false),
                        ParamSemantic::Out => (false, true),
                        ParamSemantic::InOut => (true, true),
                    };

                    params.push(ParameterABI::Frame {
                        offset: frame_offset,
                        size,
                        align,
                        copy_in,
                        copy_out,
                        semantic,
                    });

                    frame_offset += size;
                }
            }
        }

        // Classify result
        let result = if let Some(result_ty) = result_type {
            let classification = classify_type(module, result_ty)?;

            match classification {
                TypeClass::Flattened(valtypes, byte_size) => Some(ResultABI::Flattened {
                    valtypes,
                    byte_size,
                }),
                TypeClass::Frame(size, align) => {
                    uses_frame = true;
                    frame_alignment = frame_alignment.max(align);

                    // Align offset for result space
                    frame_offset = align_up(frame_offset, align);
                    frame_offset += size;

                    Some(ResultABI::Frame { size, align })
                }
            }
        } else {
            None
        };

        Ok(FunctionABI {
            params,
            result,
            uses_frame,
            frame_size: frame_offset,
            frame_alignment,
        })
    }

    /// Get WASM function type signature (parameter types only).
    pub fn param_valtypes(&self) -> Vec<ValType> {
        let mut valtypes = Vec::new();

        for param in &self.params {
            match param {
                ParameterABI::Flattened {
                    valtypes: vtypes, ..
                } => {
                    valtypes.extend_from_slice(vtypes);
                }
                ParameterABI::Frame { .. } => {
                    // Frame params are passed as i32 pointer
                    valtypes.push(ValType::I32);
                }
            }
        }

        valtypes
    }

    /// Get WASM function type signature (result types only).
    pub fn result_valtypes(&self) -> Vec<ValType> {
        match &self.result {
            Some(ResultABI::Flattened { valtypes, .. }) => valtypes.clone(),
            Some(ResultABI::Frame { .. }) => {
                // Frame results don't appear in WASM signature (written to memory)
                Vec::new()
            }
            None => Vec::new(),
        }
    }
}

/// Classification result for a type.
#[derive(Debug, Clone, PartialEq)]
enum TypeClass {
    /// Can be flattened into these WASM value types.
    Flattened(Vec<ValType>, u32), // valtypes, byte_size
    /// Must be passed via frame pointer.
    Frame(u32, u32), // size, align
}

/// Classify a Naga type for ABI purposes.
fn classify_type(module: &Module, ty: Handle<Type>) -> Result<TypeClass, ABIError> {
    let type_info = &module.types[ty];

    match &type_info.inner {
        TypeInner::Scalar(scalar) => {
            // Scalars are always flattened
            let valtype = scalar_to_valtype(scalar.kind)?;
            let byte_size = scalar.width as u32;
            Ok(TypeClass::Flattened(vec![valtype], byte_size))
        }

        TypeInner::Vector { size, scalar } => {
            // Vectors flatten if small enough
            let component_count = match size {
                naga::VectorSize::Bi => 2,
                naga::VectorSize::Tri => 3,
                naga::VectorSize::Quad => 4,
            };
            let component_bytes = scalar.width as u32;
            let total_bytes = component_count * component_bytes;

            if total_bytes <= MAX_FLATTEN_BYTES {
                let valtype = scalar_to_valtype(scalar.kind)?;
                let valtypes = vec![valtype; component_count as usize];
                Ok(TypeClass::Flattened(valtypes, total_bytes))
            } else {
                // Large vector, use frame
                let align = component_bytes;
                Ok(TypeClass::Frame(total_bytes, align))
            }
        }

        TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => {
            // Matrices flatten if small enough
            let col_count = match columns {
                naga::VectorSize::Bi => 2,
                naga::VectorSize::Tri => 3,
                naga::VectorSize::Quad => 4,
            };
            let row_count = match rows {
                naga::VectorSize::Bi => 2,
                naga::VectorSize::Tri => 3,
                naga::VectorSize::Quad => 4,
            };
            let component_bytes = scalar.width as u32;
            let total_bytes = col_count * row_count * component_bytes;

            if total_bytes <= MAX_FLATTEN_BYTES {
                let valtype = scalar_to_valtype(scalar.kind)?;
                let valtypes = vec![valtype; (col_count * row_count) as usize];
                Ok(TypeClass::Flattened(valtypes, total_bytes))
            } else {
                // Large matrix, use frame
                let align = component_bytes;
                Ok(TypeClass::Frame(total_bytes, align))
            }
        }

        TypeInner::Struct { members, span } => {
            // Try to flatten struct if all members flattenable and total size small
            let total_bytes = *span;

            if total_bytes <= MAX_FLATTEN_BYTES {
                let mut all_valtypes = Vec::new();
                let mut can_flatten = true;

                for member in members {
                    match classify_type(module, member.ty)? {
                        TypeClass::Flattened(valtypes, _) => {
                            all_valtypes.extend(valtypes);
                        }
                        TypeClass::Frame(_, _) => {
                            can_flatten = false;
                            break;
                        }
                    }
                }

                if can_flatten && all_valtypes.len() <= MAX_PARAM_COUNT {
                    Ok(TypeClass::Flattened(all_valtypes, total_bytes))
                } else {
                    // Struct too complex or large, use frame
                    let align = members
                        .iter()
                        .map(|m| type_alignment(module, m.ty))
                        .max()
                        .unwrap_or(1);
                    Ok(TypeClass::Frame(total_bytes, align))
                }
            } else {
                // Struct too large, use frame
                let align = members
                    .iter()
                    .map(|m| type_alignment(module, m.ty))
                    .max()
                    .unwrap_or(1);
                Ok(TypeClass::Frame(total_bytes, align))
            }
        }

        TypeInner::Array { base, size, stride } => {
            match size {
                ArraySize::Constant(count) => {
                    let element_class = classify_type(module, *base)?;
                    let count = count.get() as u32;
                    let total_bytes = *stride * count;

                    // Arrays rarely flatten well, but allow small constant arrays
                    if total_bytes <= MAX_FLATTEN_BYTES {
                        if let TypeClass::Flattened(elem_valtypes, _) = element_class {
                            let mut all_valtypes = Vec::new();
                            for _ in 0..count {
                                all_valtypes.extend_from_slice(&elem_valtypes);
                            }

                            if all_valtypes.len() <= MAX_PARAM_COUNT {
                                return Ok(TypeClass::Flattened(all_valtypes, total_bytes));
                            }
                        }
                    }

                    // Use frame for arrays
                    let align = type_alignment(module, *base);
                    Ok(TypeClass::Frame(total_bytes, align))
                }
                ArraySize::Dynamic => {
                    return Err(ABIError::DynamicArrayInSignature);
                }
                ArraySize::Pending(_) => {
                    return Err(ABIError::UnsupportedType);
                }
            }
        }

        TypeInner::Pointer { .. } | TypeInner::ValuePointer { .. } => {
            // Pointers are passed as i32
            Ok(TypeClass::Flattened(vec![ValType::I32], 4))
        }

        TypeInner::Image { .. } | TypeInner::Sampler { .. } => {
            // Resources are handles (i32)
            Ok(TypeClass::Flattened(vec![ValType::I32], 4))
        }

        TypeInner::Atomic(scalar) => {
            // Atomics treated as scalars
            let valtype = scalar_to_valtype(scalar.kind)?;
            let byte_size = scalar.width as u32;
            Ok(TypeClass::Flattened(vec![valtype], byte_size))
        }

        TypeInner::BindingArray { .. }
        | TypeInner::RayQuery { .. }
        | TypeInner::AccelerationStructure { .. }
        | TypeInner::CooperativeMatrix { .. } => {
            return Err(ABIError::UnsupportedType);
        }
    }
}

/// Map Naga scalar kind to WASM value type.
fn scalar_to_valtype(kind: ScalarKind) -> Result<ValType, ABIError> {
    match kind {
        ScalarKind::Sint => Ok(ValType::I32),
        ScalarKind::Uint => Ok(ValType::I32),
        ScalarKind::Float => Ok(ValType::F32),
        ScalarKind::Bool => Ok(ValType::I32),
        ScalarKind::AbstractInt | ScalarKind::AbstractFloat => Err(ABIError::UnsupportedType),
    }
}

/// Get alignment for a type (simplified).
fn type_alignment(module: &Module, ty: Handle<Type>) -> u32 {
    let type_info = &module.types[ty];

    match &type_info.inner {
        TypeInner::Scalar(scalar) => scalar.width as u32,
        TypeInner::Vector { scalar, .. } => scalar.width as u32,
        TypeInner::Matrix { scalar, .. } => scalar.width as u32,
        TypeInner::Struct { members, .. } => members
            .iter()
            .map(|m| type_alignment(module, m.ty))
            .max()
            .unwrap_or(1),
        TypeInner::Array { base, .. } => type_alignment(module, *base),
        _ => 4, // Default alignment for pointers/resources
    }
}

/// Align a value up to the given alignment.
fn align_up(value: u32, align: u32) -> u32 {
    (value + align - 1) & !(align - 1)
}

/// Errors that can occur during ABI computation.
#[derive(Debug, Clone, PartialEq)]
pub enum ABIError {
    UnsupportedType,
    DynamicArrayInSignature,
    TooManyParameters,
}

impl std::fmt::Display for ABIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ABIError::UnsupportedType => write!(f, "Unsupported type in function signature"),
            ABIError::DynamicArrayInSignature => {
                write!(f, "Dynamic arrays cannot be used in function signatures")
            }
            ABIError::TooManyParameters => {
                write!(f, "Too many flattened parameters (exceeds MAX_PARAM_COUNT)")
            }
        }
    }
}

impl std::error::Error for ABIError {}

#[cfg(test)]
mod tests {
    use super::*;
    use naga::{Scalar, Span, Type, TypeInner, VectorSize};

    fn create_test_module() -> Module {
        Module::default()
    }

    fn add_scalar_type(module: &mut Module, kind: ScalarKind, width: u8) -> Handle<Type> {
        module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Scalar(Scalar { kind, width }),
            },
            Span::UNDEFINED,
        )
    }

    fn add_vector_type(
        module: &mut Module,
        size: VectorSize,
        kind: ScalarKind,
        width: u8,
    ) -> Handle<Type> {
        module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size,
                    scalar: Scalar { kind, width },
                },
            },
            Span::UNDEFINED,
        )
    }

    #[test]
    fn test_scalar_f32_flattened() {
        let mut module = create_test_module();
        let f32_ty = add_scalar_type(&mut module, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[f32_ty], None).unwrap();

        assert_eq!(abi.params.len(), 1);
        match &abi.params[0] {
            ParameterABI::Flattened {
                valtypes,
                byte_size,
            } => {
                assert_eq!(valtypes, &[ValType::F32]);
                assert_eq!(*byte_size, 4);
            }
            _ => panic!("Expected Flattened"),
        }
        assert!(!abi.uses_frame);
    }

    #[test]
    fn test_scalar_i32_flattened() {
        let mut module = create_test_module();
        let i32_ty = add_scalar_type(&mut module, ScalarKind::Sint, 4);

        let abi = FunctionABI::compute(&module, &[i32_ty], None).unwrap();

        assert_eq!(abi.params.len(), 1);
        match &abi.params[0] {
            ParameterABI::Flattened {
                valtypes,
                byte_size,
            } => {
                assert_eq!(valtypes, &[ValType::I32]);
                assert_eq!(*byte_size, 4);
            }
            _ => panic!("Expected Flattened"),
        }
    }

    #[test]
    fn test_vec2_flattened() {
        let mut module = create_test_module();
        let vec2_ty = add_vector_type(&mut module, VectorSize::Bi, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[vec2_ty], None).unwrap();

        assert_eq!(abi.params.len(), 1);
        match &abi.params[0] {
            ParameterABI::Flattened {
                valtypes,
                byte_size,
            } => {
                assert_eq!(valtypes, &[ValType::F32, ValType::F32]);
                assert_eq!(*byte_size, 8);
            }
            _ => panic!("Expected Flattened"),
        }
    }

    #[test]
    fn test_vec4_flattened() {
        let mut module = create_test_module();
        let vec4_ty = add_vector_type(&mut module, VectorSize::Quad, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[vec4_ty], None).unwrap();

        assert_eq!(abi.params.len(), 1);
        match &abi.params[0] {
            ParameterABI::Flattened {
                valtypes,
                byte_size,
            } => {
                assert_eq!(valtypes, &[ValType::F32; 4]);
                assert_eq!(*byte_size, 16);
            }
            _ => panic!("Expected Flattened"),
        }
    }

    #[test]
    fn test_multiple_scalars() {
        let mut module = create_test_module();
        let f32_ty = add_scalar_type(&mut module, ScalarKind::Float, 4);
        let i32_ty = add_scalar_type(&mut module, ScalarKind::Sint, 4);

        let abi = FunctionABI::compute(&module, &[f32_ty, i32_ty, f32_ty], None).unwrap();

        assert_eq!(abi.params.len(), 3);
        assert!(!abi.uses_frame);

        let valtypes = abi.param_valtypes();
        assert_eq!(valtypes, &[ValType::F32, ValType::I32, ValType::F32]);
    }

    #[test]
    fn test_scalar_result() {
        let mut module = create_test_module();
        let f32_ty = add_scalar_type(&mut module, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[], Some(f32_ty)).unwrap();

        assert!(abi.params.is_empty());
        assert!(abi.result.is_some());

        match &abi.result.unwrap() {
            ResultABI::Flattened {
                valtypes,
                byte_size,
            } => {
                assert_eq!(valtypes, &[ValType::F32]);
                assert_eq!(*byte_size, 4);
            }
            _ => panic!("Expected Flattened result"),
        }
    }

    #[test]
    fn test_vec3_result() {
        let mut module = create_test_module();
        let vec3_ty = add_vector_type(&mut module, VectorSize::Tri, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[], Some(vec3_ty)).unwrap();

        match &abi.result.unwrap() {
            ResultABI::Flattened {
                valtypes,
                byte_size,
            } => {
                assert_eq!(valtypes, &[ValType::F32; 3]);
                assert_eq!(*byte_size, 12);
            }
            _ => panic!("Expected Flattened result"),
        }
    }

    #[test]
    fn test_param_valtypes() {
        let mut module = create_test_module();
        let f32_ty = add_scalar_type(&mut module, ScalarKind::Float, 4);
        let vec2_ty = add_vector_type(&mut module, VectorSize::Bi, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[f32_ty, vec2_ty], None).unwrap();

        let valtypes = abi.param_valtypes();
        assert_eq!(valtypes, &[ValType::F32, ValType::F32, ValType::F32]);
    }

    #[test]
    fn test_result_valtypes() {
        let mut module = create_test_module();
        let vec3_ty = add_vector_type(&mut module, VectorSize::Tri, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[], Some(vec3_ty)).unwrap();

        let valtypes = abi.result_valtypes();
        assert_eq!(valtypes, &[ValType::F32; 3]);
    }

    #[test]
    fn test_void_return() {
        let mut module = create_test_module();
        let f32_ty = add_scalar_type(&mut module, ScalarKind::Float, 4);

        let abi = FunctionABI::compute(&module, &[f32_ty], None).unwrap();

        assert!(abi.result.is_none());
        assert!(abi.result_valtypes().is_empty());
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(9, 8), 16);
    }

    #[test]
    fn test_semantic_detection_inout_pointer() {
        let mut module = create_test_module();

        // Create vec4 type first to avoid borrow checker issues
        let vec4_ty = add_vector_type(&mut module, VectorSize::Quad, ScalarKind::Float, 4);

        // Create a large struct (>16 bytes) to force Frame classification
        let big_struct_ty = module.types.insert(
            Type {
                name: Some("BigStruct".to_string()),
                inner: TypeInner::Struct {
                    members: vec![
                        naga::StructMember {
                            name: Some("field0".to_string()),
                            ty: vec4_ty,
                            binding: None,
                            offset: 0,
                        },
                        naga::StructMember {
                            name: Some("field1".to_string()),
                            ty: vec4_ty,
                            binding: None,
                            offset: 16,
                        },
                        naga::StructMember {
                            name: Some("field2".to_string()),
                            ty: vec4_ty,
                            binding: None,
                            offset: 32,
                        },
                        naga::StructMember {
                            name: Some("field3".to_string()),
                            ty: vec4_ty,
                            binding: None,
                            offset: 48,
                        },
                        naga::StructMember {
                            name: Some("field4".to_string()),
                            ty: vec4_ty,
                            binding: None,
                            offset: 64,
                        },
                    ],
                    span: 80,
                },
            },
            naga::Span::default(),
        );

        // Wrap in Function-space pointer (simulates out/inout parameter)
        let pointer_ty = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Pointer {
                    base: big_struct_ty,
                    space: AddressSpace::Function,
                },
            },
            naga::Span::default(),
        );

        let abi = FunctionABI::compute(&module, &[pointer_ty], None).unwrap();

        assert_eq!(abi.params.len(), 1);
        match &abi.params[0] {
            ParameterABI::Frame {
                copy_in,
                copy_out,
                semantic,
                ..
            } => {
                assert_eq!(*semantic, ParamSemantic::InOut, "Pointer should be InOut");
                assert!(*copy_in, "InOut should copy_in");
                assert!(*copy_out, "InOut should copy_out");
            }
            _ => panic!("Expected Frame parameter"),
        }
    }
}
