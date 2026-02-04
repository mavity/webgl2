use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::ActiveInfo;
use super::types::*;
use crate::naga_wasm_backend::{WasmBackend, WasmBackendConfig};
use naga::front::glsl::{Frontend, Options};
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::{AddressSpace, Binding, ShaderStage};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Shader and Program Operations
// ============================================================================

/// Check if object is a shader.
pub fn ctx_is_shader(ctx: u32, handle: u32) -> bool {
    clear_last_error();
    if handle == 0 {
        return false;
    }
    let reg = get_registry().borrow();
    if let Some(c) = reg.contexts.get(&ctx) {
        c.shaders.contains_key(&handle)
    } else {
        false
    }
}

/// Check if object is a program.
pub fn ctx_is_program(ctx: u32, handle: u32) -> bool {
    clear_last_error();
    if handle == 0 {
        return false;
    }
    let reg = get_registry().borrow();
    if let Some(c) = reg.contexts.get(&ctx) {
        c.programs.contains_key(&handle)
    } else {
        false
    }
}

/// Create a shader.
pub fn ctx_create_shader(ctx: u32, type_: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let shader_id = ctx_obj.allocate_shader_handle();
    ctx_obj.shaders.insert(
        shader_id,
        Shader {
            type_,
            source: String::new(),
            compiled: false,
            info_log: String::new(),
            module: None,
            info: None,
        },
    );
    shader_id
}

/// Delete a shader.
pub fn ctx_delete_shader(ctx: u32, shader: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.shaders.remove(&shader);
    ERR_OK
}

/// Set shader source.
pub fn ctx_shader_source(ctx: u32, shader: u32, source_ptr: u32, source_len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let source_slice =
        unsafe { std::slice::from_raw_parts(source_ptr as *const u8, source_len as usize) };
    let source = String::from_utf8_lossy(source_slice).into_owned();

    if let Some(s) = ctx_obj.shaders.get_mut(&shader) {
        s.source = source;
        ERR_OK
    } else {
        set_last_error("shader not found");
        ERR_INVALID_HANDLE
    }
}

/// Compile a shader.
pub fn ctx_compile_shader(ctx: u32, shader: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if let Some(s) = ctx_obj.shaders.get_mut(&shader) {
        let stage = match s.type_ {
            GL_VERTEX_SHADER => naga::ShaderStage::Vertex,
            GL_FRAGMENT_SHADER => naga::ShaderStage::Fragment,
            _ => {
                s.compiled = false;
                s.info_log = "Invalid shader type".to_string();
                return ERR_INVALID_ARGS;
            }
        };

        let mut frontend = Frontend::default();
        let options = Options::from(stage);

        match frontend.parse(&options, &s.source) {
            Ok(module) => {
                let mut validator = Validator::new(
                    ValidationFlags::all() & !ValidationFlags::BINDINGS,
                    Capabilities::all(),
                );
                match validator.validate(&module) {
                    Ok(info) => {
                        s.compiled = true;
                        s.info_log = "Shader compiled successfully".to_string();
                        s.module = Some(Arc::new(module));
                        s.info = Some(Arc::new(info));
                        ERR_OK
                    }
                    Err(e) => {
                        s.compiled = false;
                        s.info_log = format!("Validation error: {:?}", e);
                        ERR_OK
                    }
                }
            }
            Err(e) => {
                s.compiled = false;
                s.info_log = format!("Compilation error: {:?}", e);
                ERR_OK
            }
        }
    } else {
        set_last_error("shader not found");
        ERR_INVALID_HANDLE
    }
}

/// Get shader parameter.
pub fn ctx_get_shader_parameter(ctx: u32, shader: u32, pname: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(s) = ctx_obj.shaders.get(&shader) {
        match pname {
            GL_SHADER_TYPE => s.type_ as i32,
            GL_COMPILE_STATUS => {
                if s.compiled {
                    1
                } else {
                    0
                }
            }
            GL_DELETE_STATUS => 0, // Not implemented
            _ => 0,
        }
    } else {
        0
    }
}

/// Get shader info log.
pub fn ctx_get_shader_info_log(ctx: u32, shader: u32, dest_ptr: u32, dest_len: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(s) = ctx_obj.shaders.get(&shader) {
        let bytes = s.info_log.as_bytes();
        let len = std::cmp::min(bytes.len(), dest_len as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr as *mut u8, len);
        }
        len as u32
    } else {
        0
    }
}

/// Create a program.
pub fn ctx_create_program(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let program_id = ctx_obj.allocate_program_handle();
    ctx_obj.programs.insert(program_id, Program::default());
    program_id
}

/// Delete a program.
pub fn ctx_delete_program(ctx: u32, program: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.programs.remove(&program);
    if ctx_obj.current_program == Some(program) {
        ctx_obj.current_program = None;
    }
    ERR_OK
}

/// Attach a shader to a program.
pub fn ctx_attach_shader(ctx: u32, program: u32, shader: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if !ctx_obj.shaders.contains_key(&shader) {
        set_last_error("shader not found");
        return ERR_INVALID_HANDLE;
    }

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        if !p.attached_shaders.contains(&shader) {
            p.attached_shaders.push(shader);
        }
        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

/// Link a program.
pub fn ctx_link_program(ctx: u32, program: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        let mut vs_module = None;
        let mut fs_module = None;
        let mut vs_info = None;
        let mut fs_info = None;
        let mut vs_source = String::new();
        let mut fs_source = String::new();

        for &s_id in &p.attached_shaders {
            if let Some(s) = ctx_obj.shaders.get(&s_id) {
                if !s.compiled {
                    p.linked = false;
                    p.info_log = format!("Shader {} is not compiled", s_id);
                    return ERR_OK;
                }
                match s.type_ {
                    GL_VERTEX_SHADER => {
                        vs_module = s.module.clone();
                        vs_info = s.info.clone();
                        vs_source = s.source.clone();
                    }
                    GL_FRAGMENT_SHADER => {
                        fs_module = s.module.clone();
                        fs_info = s.info.clone();
                        fs_source = s.source.clone();
                    }
                    _ => {}
                }
            }
        }

        if vs_module.is_none() || fs_module.is_none() {
            p.linked = false;
            p.info_log = "Program must have both vertex and fragment shaders".to_string();
            return ERR_OK;
        }

        p.vs_module = vs_module;
        p.fs_module = fs_module;
        p.vs_info = vs_info;
        p.fs_info = fs_info;

        // Extract attributes and uniforms from Naga modules to ensure consistent locations
        p.attributes.clear();
        p.uniforms.clear();
        p.uniform_blocks.clear();
        p.uniform_block_bindings.clear();
        let mut attribute_locations = HashMap::new();
        let mut uniform_locations = HashMap::new();
        let mut next_uniform_loc = 0;
        let mut varying_locations = HashMap::new();
        let mut next_varying_loc = 0; // gl_Position is handled separately at offset 0
                                      // Temporary maps to collect type information from VS and FS so we can validate compatibility
        let mut vs_varying_types: HashMap<String, (u8, u32)> = HashMap::new();
        let mut fs_varying_types: HashMap<String, (u8, u32)> = HashMap::new();

        // Helper to map naga::Type to (type_code, components)
        let get_type_info = |ty: &naga::Type| -> (u8, u32) {
            match ty.inner {
                naga::TypeInner::Scalar(s) => match s.kind {
                    naga::ScalarKind::Sint => (1, 1),
                    naga::ScalarKind::Uint => (2, 1),
                    _ => (0, 1),
                },
                naga::TypeInner::Vector { size, scalar, .. } => match scalar.kind {
                    naga::ScalarKind::Sint => (1, size as u32),
                    naga::ScalarKind::Uint => (2, size as u32),
                    _ => (0, size as u32),
                },
                naga::TypeInner::Matrix { columns, rows, .. } => {
                    (0, (columns as u32) * (rows as u32))
                }
                _ => (0, 1),
            }
        };

        if let Some(vs) = &p.vs_module {
            // 1. Collect active attributes and their requested locations
            let mut active_attributes = Vec::new();
            for ep in &vs.entry_points {
                if ep.stage == ShaderStage::Vertex {
                    for arg in &ep.function.arguments {
                        if let Some(name) = &arg.name {
                            active_attributes.push(name.clone());
                        }
                    }
                }
            }

            // 2. Assign locations
            // We keep track of the origin of a location so we can resolve collisions
            // Priority: Layout (shader) > bindAttribLocation (user) > Naga default (assigned)
            // used_locations: location -> (name, origin)
            // origin: 0 = assigned (default), 1 = layout, 2 = bindAttribLocation
            let mut used_locations: HashMap<u32, (String, u8)> = HashMap::new();
            let mut unassigned = Vec::new();

            for name in &active_attributes {
                let mut location: Option<u32> = None;
                let mut origin: u8 = 0; // default: assigned

                // Prefer user-specified bindAttribLocation first (so user binding overrides default Naga-assigned locations)
                if let Some(&loc) = p.attribute_bindings.get(name) {
                    location = Some(loc);
                    origin = 2;
                }

                // If no user binding, check for layout qualifier in shader (explicit location)
                if location.is_none() {
                    for ep in &vs.entry_points {
                        if ep.stage == ShaderStage::Vertex {
                            for arg in &ep.function.arguments {
                                if arg.name.as_ref() == Some(name) {
                                    if let Some(Binding::Location { location: loc, .. }) =
                                        &arg.binding
                                    {
                                        location = Some(*loc);
                                        origin = 1;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(loc) = location {
                    if let Some((other_name_ref, other_origin)) = used_locations.get(&loc) {
                        // Clone the data we need so we don't hold an immutable borrow while mutating
                        let other_name = other_name_ref.clone();
                        let other_origin = *other_origin;
                        if other_name != *name {
                            // If existing mapping was a Naga-assigned default (0) and the new one is user/layout (1 or 2),
                            // prefer the new mapping and bump the old attribute to unassigned so it can get a new location.
                            if other_origin == 0 && origin > 0 {
                                // remove previous assignment and mark it unassigned
                                attribute_locations.remove(&other_name);
                                p.attributes.remove(&other_name);
                                used_locations.insert(loc, (name.clone(), origin));
                                attribute_locations.insert(name.clone(), loc);
                                p.attributes.insert(name.clone(), loc as i32);
                                unassigned.push(other_name);
                            } else if origin == 0 && other_origin > 0 {
                                // keep existing user/layout mapping, mark current as unassigned
                                unassigned.push(name.clone());
                            } else {
                                // Both are user-specified (or both layout/bind) -> link error
                                p.linked = false;
                                p.info_log = format!(
                                    "Link failed: Attributes '{}' and '{}' are both bound to location {}",
                                    name, other_name, loc
                                );
                                return ERR_OK;
                            }
                        }
                    } else {
                        used_locations.insert(loc, (name.clone(), origin));
                        attribute_locations.insert(name.clone(), loc);
                        p.attributes.insert(name.clone(), loc as i32);
                        // Record attribute type info based on VS entry point args
                        for ep in &vs.entry_points {
                            if ep.stage == ShaderStage::Vertex {
                                for arg in &ep.function.arguments {
                                    if arg.name.as_ref() == Some(name) {
                                        let ty = &vs.types[arg.ty];
                                        let type_info = get_type_info(ty);
                                        p.attribute_types.insert(name.clone(), type_info);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    unassigned.push(name.clone());
                }
            }

            // 3. Assign remaining attributes to unused locations
            let mut next_loc = 0;
            for name in unassigned {
                while used_locations.contains_key(&next_loc) {
                    next_loc += 1;
                }
                used_locations.insert(next_loc, (name.clone(), 0));
                attribute_locations.insert(name.clone(), next_loc);
                p.attributes.insert(name.clone(), next_loc as i32);
                // Record attribute type info based on VS entry point args
                for ep in &vs.entry_points {
                    if ep.stage == ShaderStage::Vertex {
                        for arg in &ep.function.arguments {
                            if arg.name.as_ref() == Some(&name) {
                                let ty = &vs.types[arg.ty];
                                let type_info = get_type_info(ty);
                                p.attribute_types.insert(name.clone(), type_info);
                            }
                        }
                    }
                }
                next_loc += 1;
            }

            // 0. Pre-scan VS entry point results for explicit varying locations
            let mut explicit_candidates = Vec::new();
            for ep in &vs.entry_points {
                if ep.stage == ShaderStage::Vertex {
                    if let Some(result) = &ep.function.result {
                        if let naga::TypeInner::Struct { members, .. } = &vs.types[result.ty].inner
                        {
                            for member in members {
                                if let Some(Binding::Location { location, .. }) = member.binding {
                                    if let Some(name) = &member.name {
                                        explicit_candidates.push((name.clone(), location));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Filter candidates: if a location is used multiple times, discard all usages (treat as implicit)
            let mut loc_counts = HashMap::new();
            for (_, loc) in &explicit_candidates {
                *loc_counts.entry(*loc).or_insert(0) += 1;
            }

            for (name, loc) in explicit_candidates {
                if loc_counts[&loc] == 1 {
                    varying_locations.insert(name, loc);
                    if loc >= next_varying_loc {
                        next_varying_loc = loc + 1;
                    }
                }
            }

            // Collect uniforms and varyings from globals _and_ from entry point args/results
            let mut vs_globals: Vec<_> = vs.global_variables.iter().collect();
            vs_globals.sort_by_key(|(handle, _)| handle.index());

            // TODO: is handle a stray local in the API? Why is it there?
            for (_handle, var) in vs_globals {
                if var.space == AddressSpace::Handle
                    || (var.space == AddressSpace::Uniform && var.binding.is_none())
                {
                    if let Some(name) = &var.name {
                        if !p.uniforms.contains_key(name) {
                            let location = next_uniform_loc;
                            next_uniform_loc += 1;
                            p.uniforms.insert(name.clone(), location);
                            uniform_locations.insert(name.clone(), location as u32);
                            // Record uniform type info
                            let ty = &vs.types[var.ty];
                            let type_info = get_type_info(ty);
                            p.uniform_types.insert(name.clone(), type_info);
                        }
                    }
                } else if var.space == AddressSpace::Uniform {
                    // Uniform Block (has binding)
                    if let Some(name) = &var.name {
                        if !p.uniform_blocks.contains(name) {
                            p.uniform_blocks.push(name.clone());
                        }
                    }
                } else {
                    // Treat any non-uniform/handle globals as varyings (covers Private / In / Out)
                    if let Some(name) = &var.name {
                        if name != "gl_Position"
                            && name != "gl_Position_1"
                            && !p.attributes.contains_key(name)
                            && !varying_locations.contains_key(name)
                        {
                            varying_locations.insert(name.clone(), next_varying_loc);
                            // record VS type info
                            let ty = &vs.types[var.ty];
                            let type_info = get_type_info(ty);
                            vs_varying_types.insert(name.clone(), type_info);
                            // also store on the program for runtime mapping
                            p.varying_locations.insert(name.clone(), next_varying_loc);
                            p.varying_types.insert(name.clone(), type_info);
                            next_varying_loc += 1;
                        }
                    }
                }
            }

            // Also scan entry point arguments/results for varyings (addresses AddressSpace::Input/Output cases)
            for ep in &vs.entry_points {
                if ep.stage == ShaderStage::Vertex {
                    for arg in &ep.function.arguments {
                        if let Some(name) = &arg.name {
                            if name != "gl_Position"
                                && !p.attributes.contains_key(name)
                                && !p.uniforms.contains_key(name)
                                && !varying_locations.contains_key(name)
                            {
                                // If this arg has an explicit location binding, use it when possible
                                if let Some(Binding::Location { location: loc, .. }) = &arg.binding
                                {
                                    // If that location is already used by another varying, do not overwrite (keep unique)
                                    if !varying_locations.values().any(|&v| v == *loc) {
                                        varying_locations.insert(name.clone(), *loc);
                                        if *loc >= next_varying_loc {
                                            next_varying_loc = *loc + 1;
                                        }
                                        continue;
                                    }
                                }
                                varying_locations.insert(name.clone(), next_varying_loc);
                                next_varying_loc += 1;
                            }
                        }
                    }
                }
            }

            // Collect uniforms and varyings from globals _and_ from entry point args/results
            let mut vs_globals: Vec<_> = vs.global_variables.iter().collect();
            vs_globals.sort_by_key(|(handle, _)| handle.index());

            for (_handle, var) in vs_globals {
                if var.space == AddressSpace::Handle
                    || (var.space == AddressSpace::Uniform && var.binding.is_none())
                {
                    if let Some(name) = &var.name {
                        if !p.uniforms.contains_key(name) {
                            let location = next_uniform_loc;
                            next_uniform_loc += 1;
                            p.uniforms.insert(name.clone(), location);
                            uniform_locations.insert(name.clone(), location as u32);
                            // Record uniform type info
                            let ty = &vs.types[var.ty];
                            let type_info = get_type_info(ty);
                            p.uniform_types.insert(name.clone(), type_info);
                        }
                    }
                } else if var.space == AddressSpace::Uniform {
                    // Collect uniform blocks
                    if let Some(name) = &var.name {
                        if !p.uniform_blocks.contains(name) {
                            p.uniform_blocks.push(name.clone());
                        }
                    }
                }
            }

            if let Some(fs) = &p.fs_module {
                // Also scan FS entry points for input varyings
                for ep in &fs.entry_points {
                    if ep.stage == ShaderStage::Fragment {
                        for arg in &ep.function.arguments {
                            if let Some(name) = &arg.name {
                                if name != "color"
                                    && name != "fragColor"
                                    && !name.ends_with("Color")
                                    && !p.uniforms.contains_key(name)
                                    && !varying_locations.contains_key(name)
                                {
                                    if let Some(Binding::Location { location: loc, .. }) =
                                        &arg.binding
                                    {
                                        if !varying_locations.values().any(|&v| v == *loc) {
                                            varying_locations.insert(name.clone(), *loc);
                                            // FS arg type info
                                            let ty = &fs.types[arg.ty];
                                            let type_info = get_type_info(ty);
                                            fs_varying_types.insert(name.clone(), type_info);
                                            p.varying_locations.insert(name.clone(), *loc);
                                            p.varying_types.insert(name.clone(), type_info);
                                            if *loc >= next_varying_loc {
                                                next_varying_loc = *loc + 1;
                                            }
                                            continue;
                                        }
                                    }
                                    varying_locations.insert(name.clone(), next_varying_loc);
                                    let ty = &fs.types[arg.ty];
                                    let type_info = get_type_info(ty);
                                    fs_varying_types.insert(name.clone(), type_info);
                                    p.varying_locations.insert(name.clone(), next_varying_loc);
                                    p.varying_types.insert(name.clone(), type_info);
                                    next_varying_loc += 1;
                                }
                            }
                        }
                    }
                }

                let mut fs_globals: Vec<_> = fs.global_variables.iter().collect();
                fs_globals.sort_by_key(|(handle, _)| handle.index());

                for (_handle, var) in fs_globals {
                    if var.space == AddressSpace::Handle
                        || (var.space == AddressSpace::Uniform && var.binding.is_none())
                    {
                        if let Some(name) = &var.name {
                            if !p.uniforms.contains_key(name) {
                                let location = next_uniform_loc;
                                next_uniform_loc += 1;
                                p.uniforms.insert(name.clone(), location);
                                uniform_locations.insert(name.clone(), location as u32);
                                // Record uniform type info
                                let ty = &fs.types[var.ty];
                                let type_info = get_type_info(ty);
                                p.uniform_types.insert(name.clone(), type_info);
                            }
                        }
                    } else if var.space == AddressSpace::Uniform {
                        // Uniform Block (has binding)
                        if let Some(name) = &var.name {
                            if !p.uniform_blocks.contains(name) {
                                p.uniform_blocks.push(name.clone());
                            }
                        }
                    } else {
                        // Treat any non-uniform/handle globals as varyings (covers Private / In / Out)
                        if let Some(name) = &var.name {
                            if name != "color"
                                && name != "gl_FragColor"
                                && name != "gl_FragColor_1"
                                && name != "fragColor"
                                && !name.ends_with("Color")
                                && !varying_locations.contains_key(name)
                            {
                                varying_locations.insert(name.clone(), next_varying_loc);
                                // record FS type info
                                let ty = &fs.types[var.ty];
                                let type_info = get_type_info(ty);
                                fs_varying_types.insert(name.clone(), type_info);
                                p.varying_locations.insert(name.clone(), next_varying_loc);
                                p.varying_types.insert(name.clone(), type_info);
                                next_varying_loc += 1;
                            }
                        }
                    }
                }
            }
        }

        // Link-time type compatibility checks: ensure VS and FS agree on varying types
        for (name, &loc) in varying_locations.iter() {
            if let (Some(vs_type), Some(fs_type)) =
                (vs_varying_types.get(name), fs_varying_types.get(name))
            {
                if vs_type != fs_type {
                    p.linked = false;
                    p.info_log = format!(
                        "Link failed: Varying '{}' type mismatch at location {}: VS={:?} FS={:?}",
                        name, loc, vs_type, fs_type
                    );
                    return ERR_OK;
                }
            }
            // Ensure program-level maps are populated (in case some were only discovered in FS)
            if !p.varying_locations.contains_key(name) {
                p.varying_locations.insert(name.clone(), loc);
            }
            if !p.varying_types.contains_key(name) {
                if let Some(t) = vs_varying_types.get(name) {
                    p.varying_types.insert(name.clone(), *t);
                } else if let Some(t) = fs_varying_types.get(name) {
                    p.varying_types.insert(name.clone(), *t);
                }
            }
        }

        // Compile to WASM
        let config = WasmBackendConfig {
            debug_shaders: ctx_obj.debug_shaders,
            ..Default::default()
        };
        let backend = WasmBackend::new(config);

        if let (Some(vs), Some(vsi)) = (&p.vs_module, &p.vs_info) {
            let vs_name = format!("program_{}_vs.glsl", program);
            match backend.compile(
                crate::naga_wasm_backend::CompileConfig {
                    module: vs,
                    info: vsi,
                    source: &vs_source,
                    stage: naga::ShaderStage::Vertex,
                    entry_point: Some("main"),
                    attribute_locations: &attribute_locations,
                    uniform_locations: &uniform_locations,
                    varying_locations: &varying_locations,
                    varying_types: &p.varying_types,
                    uniform_types: &p.uniform_types,
                    attribute_types: &p.attribute_types,
                },
                Some(&vs_name),
            ) {
                Ok(wasm) => {
                    p.vs_wasm = Some(wasm.wasm_bytes);
                    p.vs_stub = wasm.debug_stub;
                }
                Err(e) => {
                    p.linked = false;
                    p.info_log = format!("VS Backend error: {:?}", e);
                    return ERR_OK;
                }
            }
        }

        if let (Some(fs), Some(fsi)) = (&p.fs_module, &p.fs_info) {
            let fs_name = format!("program_{}_fs.glsl", program);
            match backend.compile(
                crate::naga_wasm_backend::CompileConfig {
                    module: fs,
                    info: fsi,
                    source: &fs_source,
                    stage: naga::ShaderStage::Fragment,
                    entry_point: Some("main"),
                    attribute_locations: &attribute_locations,
                    uniform_locations: &uniform_locations,
                    varying_locations: &varying_locations,
                    varying_types: &p.varying_types,
                    uniform_types: &p.uniform_types,
                    attribute_types: &p.attribute_types,
                },
                Some(&fs_name),
            ) {
                Ok(wasm) => {
                    p.fs_wasm = Some(wasm.wasm_bytes);
                    p.fs_stub = wasm.debug_stub;
                }
                Err(e) => {
                    p.linked = false;
                    p.info_log = format!("FS Backend error: {:?}", e);
                    return ERR_OK;
                }
            }
        }

        p.linked = true;
        p.info_log = "Program linked successfully.".to_string();

        reflect_program_resources(p);

        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

/// Get program parameter.
pub fn ctx_get_program_parameter(ctx: u32, program: u32, pname: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        match pname {
            GL_LINK_STATUS => {
                if p.linked {
                    1
                } else {
                    0
                }
            }
            GL_ATTACHED_SHADERS => p.attached_shaders.len() as i32,
            GL_ACTIVE_UNIFORMS => p.active_uniforms.len() as i32,
            GL_ACTIVE_ATTRIBUTES => p.active_attributes.len() as i32,
            GL_DELETE_STATUS => 0,
            _ => 0,
        }
    } else {
        0
    }
}

/// Get program info log.
pub fn ctx_get_program_info_log(ctx: u32, program: u32, dest_ptr: u32, dest_len: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let bytes = p.info_log.as_bytes();
        let len = std::cmp::min(bytes.len(), dest_len as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr as *mut u8, len);
        }
        len as u32
    } else {
        0
    }
}

/// Get the length of the generated WASM for a program's shader.
pub fn ctx_get_program_wasm_len(ctx: u32, program: u32, shader_type: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let wasm = match shader_type {
            GL_VERTEX_SHADER => &p.vs_wasm,
            GL_FRAGMENT_SHADER => &p.fs_wasm,
            _ => return 0,
        };

        if let Some(bytes) = wasm {
            bytes.len() as u32
        } else {
            0
        }
    } else {
        0
    }
}

/// Get the generated WASM for a program's shader.
pub fn ctx_get_program_wasm(
    ctx: u32,
    program: u32,
    shader_type: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let wasm = match shader_type {
            GL_VERTEX_SHADER => &p.vs_wasm,
            GL_FRAGMENT_SHADER => &p.fs_wasm,
            _ => return 0,
        };

        if let Some(bytes) = wasm {
            let len = std::cmp::min(bytes.len(), dest_len as usize);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr as *mut u8, len);
            }
            len as u32
        } else {
            0
        }
    } else {
        0
    }
}

pub fn ctx_use_program(ctx: u32, program: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if program == 0 {
        ctx_obj.current_program = None;
        return ERR_OK;
    }

    if ctx_obj.programs.contains_key(&program) {
        ctx_obj.current_program = Some(program);
        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_get_uniform_location(ctx: u32, program: u32, name_ptr: u32, name_len: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return -1,
    };

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
    let name = String::from_utf8_lossy(name_slice).into_owned();

    if let Some(p) = ctx_obj.programs.get(&program) {
        if let Some(&loc) = p.uniforms.get(&name) {
            loc
        } else if name.ends_with("[0]") {
            let base_name = &name[0..name.len() - 3];
            if let Some(&loc) = p.uniforms.get(base_name) {
                loc
            } else {
                -1
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn ctx_get_uniform_block_index(ctx: u32, program: u32, name_ptr: u32, name_len: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0xFFFFFFFF,
    };

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
    let name = String::from_utf8_lossy(name_slice).into_owned();

    if let Some(p) = ctx_obj.programs.get(&program) {
        for (i, block_name) in p.uniform_blocks.iter().enumerate() {
            if block_name == &name {
                return i as u32;
            }
        }
        0xFFFFFFFF
    } else {
        0xFFFFFFFF
    }
}

pub fn ctx_uniform_block_binding(ctx: u32, program: u32, index: u32, binding: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        if (index as usize) < p.uniform_blocks.len() {
            p.uniform_block_bindings.insert(index, binding);
            ERR_OK
        } else {
            set_last_error("uniform block index out of range");
            GL_INVALID_VALUE
        }
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_get_attrib_location(ctx: u32, program: u32, name_ptr: u32, name_len: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return -1,
    };

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
    let name = String::from_utf8_lossy(name_slice).into_owned();

    if let Some(p) = ctx_obj.programs.get(&program) {
        if let Some(&loc) = p.attributes.get(&name) {
            loc
        } else {
            -1
        }
    } else {
        -1
    }
}

/// Bind attribute location.
pub fn ctx_bind_attrib_location(
    ctx: u32,
    program: u32,
    index: u32,
    name_ptr: u32,
    name_len: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
    let name = String::from_utf8_lossy(name_slice).into_owned();

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        p.attribute_bindings.insert(name, index);
        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

/// Set uniform 1f.
pub fn ctx_uniform1f(ctx: u32, location: i32, x: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    let offset =
        crate::naga_wasm_backend::output_layout::get_webgl_uniform_data_offset(location as u32)
            as usize;
    if (offset + 4) <= ctx_obj.uniform_data.len() {
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 2f.
pub fn ctx_uniform2f(ctx: u32, location: i32, x: f32, y: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    let offset =
        crate::naga_wasm_backend::output_layout::get_webgl_uniform_data_offset(location as u32)
            as usize;
    if (offset + 8) <= ctx_obj.uniform_data.len() {
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ctx_obj.uniform_data[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 3f.
pub fn ctx_uniform3f(ctx: u32, location: i32, x: f32, y: f32, z: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    let offset =
        crate::naga_wasm_backend::output_layout::get_webgl_uniform_data_offset(location as u32)
            as usize;
    if (offset + 12) <= ctx_obj.uniform_data.len() {
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ctx_obj.uniform_data[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
        ctx_obj.uniform_data[offset + 8..offset + 12].copy_from_slice(&z.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 4f.
pub fn ctx_uniform4f(ctx: u32, location: i32, x: f32, y: f32, z: f32, w: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    let offset =
        crate::naga_wasm_backend::output_layout::get_webgl_uniform_data_offset(location as u32)
            as usize;
    if (offset + 16) <= ctx_obj.uniform_data.len() {
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ctx_obj.uniform_data[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
        ctx_obj.uniform_data[offset + 8..offset + 12].copy_from_slice(&z.to_le_bytes());
        ctx_obj.uniform_data[offset + 12..offset + 16].copy_from_slice(&w.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 1i.
pub fn ctx_uniform1i(ctx: u32, location: i32, x: i32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    let offset =
        crate::naga_wasm_backend::output_layout::get_webgl_uniform_data_offset(location as u32)
            as usize;
    if (offset + 4) <= ctx_obj.uniform_data.len() {
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform matrix 4fv.
pub fn ctx_uniform_matrix_4fv(ctx: u32, location: i32, transpose: bool, ptr: u32, len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if transpose {
        set_last_error("transpose not supported");
        return ERR_INVALID_ARGS;
    }

    let offset =
        crate::naga_wasm_backend::output_layout::get_webgl_uniform_data_offset(location as u32)
            as usize;
    if (offset + len as usize * 4) <= ctx_obj.uniform_data.len() {
        let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, (len * 4) as usize) };
        ctx_obj.uniform_data[offset..offset + (len * 4) as usize].copy_from_slice(src_slice);
        ERR_OK
    } else {
        set_last_error("invalid uniform location or data length");
        ERR_INVALID_ARGS
    }
}

/// Get program debug stub.
pub fn ctx_get_program_debug_stub(
    ctx: u32,
    program: u32,
    shader_type: u32,
    ptr: u32,
    max_len: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let stub = match shader_type {
            GL_VERTEX_SHADER => &p.vs_stub,   // VERTEX_SHADER
            GL_FRAGMENT_SHADER => &p.fs_stub, // FRAGMENT_SHADER
            _ => {
                set_last_error("invalid shader type");
                return 0;
            }
        };

        if let Some(s) = stub {
            let bytes = s.as_bytes();
            let len = bytes.len() as u32;
            if len > max_len {
                // Return needed length if buffer too small?
                // Or just copy what fits?
                // Standard pattern: if ptr is 0, return length.
                if ptr == 0 {
                    return len;
                }
                // If ptr != 0 and max_len too small, return 0 or partial?
                // Let's return actual length copied.
            }

            if ptr != 0 {
                let copy_len = std::cmp::min(len, max_len);
                let dest_slice =
                    unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, copy_len as usize) };
                dest_slice.copy_from_slice(&bytes[..copy_len as usize]);
                copy_len
            } else {
                len
            }
        } else {
            0
        }
    } else {
        set_last_error("program not found");
        0
    }
}

// ============================================================================
// WAT Testing Support (docs/1.9-wat-testing.md)
// ============================================================================

// Thread-local storage for ephemeral WAT strings.
// These strings are generated on-demand and must be copied synchronously by the caller.
thread_local! {
    static WAT_STRING_STORAGE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Get a reference to the compiled WASM bytes for a program's shader.
/// Returns (ptr, len) pointing to WASM bytes owned by Rust.
/// On failure or missing module, returns (0, 0).
/// The pointer is ephemeral; callers must copy bytes synchronously.
pub fn ctx_get_program_wasm_ref(ctx: u32, program: u32, shader_type: u32) -> (u32, u32) {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return (0, 0);
        }
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let wasm = match shader_type {
            GL_VERTEX_SHADER => &p.vs_wasm,   // VERTEX_SHADER
            GL_FRAGMENT_SHADER => &p.fs_wasm, // FRAGMENT_SHADER
            _ => {
                set_last_error("invalid shader type");
                return (0, 0);
            }
        };

        if let Some(bytes) = wasm {
            let ptr = bytes.as_ptr() as u32;
            let len = bytes.len() as u32;
            (ptr, len)
        } else {
            // Module not compiled yet
            (0, 0)
        }
    } else {
        set_last_error("program not found");
        (0, 0)
    }
}

/// Get a reference to the WAT (WebAssembly Text) representation for a program's shader.
/// Returns (ptr, len) pointing to a UTF-8 encoded WAT string owned by Rust.
/// On failure or missing module, returns (0, 0).
/// The pointer is ephemeral; callers must copy/decode synchronously.
///
/// Implementation: WAT is generated on-demand using wasmprinter and stored in thread-local storage.
pub fn ctx_get_program_wat_ref(ctx: u32, program: u32, shader_type: u32) -> (u32, u32) {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return (0, 0);
        }
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let wasm = match shader_type {
            GL_VERTEX_SHADER => &p.vs_wasm,   // VERTEX_SHADER
            GL_FRAGMENT_SHADER => &p.fs_wasm, // FRAGMENT_SHADER
            _ => {
                set_last_error("invalid shader type");
                return (0, 0);
            }
        };

        if let Some(bytes) = wasm {
            // Generate WAT from WASM bytes using wasmprinter
            match wasmprinter::print_bytes(bytes) {
                Ok(wat_string) => {
                    // Store in thread-local storage and return pointer
                    WAT_STRING_STORAGE.with(|storage| {
                        let mut s = storage.borrow_mut();
                        *s = wat_string;
                        let ptr = s.as_ptr() as u32;
                        let len = s.len() as u32;
                        (ptr, len)
                    })
                }
                Err(e) => {
                    set_last_error(&format!("Failed to generate WAT: {}", e));
                    (0, 0)
                }
            }
        } else {
            // Module not compiled yet
            (0, 0)
        }
    } else {
        set_last_error("program not found");
        (0, 0)
    }
}

fn reflect_program_resources(p: &mut Program) {
    use naga::{AddressSpace, Binding, ScalarKind, ShaderStage, TypeInner, VectorSize};
    p.active_attributes.clear();
    p.active_uniforms.clear();
    // Do NOT clear p.uniforms and p.attributes here, as they were populated during linkProgram
    // with the correct locations used by the WASM backend.
    // We only need to ensure ActiveInfo is populated for introspection APIs.

    fn map_type(ty: &naga::Type, arena: &naga::UniqueArena<naga::Type>) -> (u32, i32) {
        let gl_float: u32 = 0x1406;
        let gl_float_vec2: u32 = 0x8B50;
        let gl_float_vec3: u32 = 0x8B51;
        let gl_float_vec4: u32 = 0x8B52;
        let gl_int: u32 = 0x1404;
        let gl_int_vec2: u32 = 0x8B53;
        let gl_int_vec3: u32 = 0x8B54;
        let gl_int_vec4: u32 = 0x8B55;
        let gl_unsigned_int: u32 = 0x1405;
        let gl_unsigned_int_vec2: u32 = 0x8DC6;
        let gl_unsigned_int_vec3: u32 = 0x8DC7;
        let gl_unsigned_int_vec4: u32 = 0x8DC8;
        let gl_bool: u32 = 0x8B56;
        let gl_bool_vec2: u32 = 0x8B57;
        let gl_bool_vec3: u32 = 0x8B58;
        let gl_bool_vec4: u32 = 0x8B59;
        let gl_float_mat2: u32 = 0x8B5A;
        let gl_float_mat3: u32 = 0x8B5B;
        let gl_float_mat4: u32 = 0x8B5C;
        let gl_sampler_2d: u32 = 0x8B5E;
        let gl_sampler_3d: u32 = 0x8B5F;
        let gl_sampler_cube: u32 = 0x8B60;

        match &ty.inner {
            TypeInner::Scalar(s) => match s.kind {
                ScalarKind::Float => (gl_float, 1),
                ScalarKind::Sint => (gl_int, 1),
                ScalarKind::Uint => (gl_unsigned_int, 1),
                ScalarKind::Bool => (gl_bool, 1),
                _ => (0, 1),
            },
            TypeInner::Vector { size, scalar } => match (scalar.kind, size) {
                (ScalarKind::Float, VectorSize::Bi) => (gl_float_vec2, 1),
                (ScalarKind::Float, VectorSize::Tri) => (gl_float_vec3, 1),
                (ScalarKind::Float, VectorSize::Quad) => (gl_float_vec4, 1),
                (ScalarKind::Sint, VectorSize::Bi) => (gl_int_vec2, 1),
                (ScalarKind::Sint, VectorSize::Tri) => (gl_int_vec3, 1),
                (ScalarKind::Sint, VectorSize::Quad) => (gl_int_vec4, 1),
                (ScalarKind::Uint, VectorSize::Bi) => (gl_unsigned_int_vec2, 1),
                (ScalarKind::Uint, VectorSize::Tri) => (gl_unsigned_int_vec3, 1),
                (ScalarKind::Uint, VectorSize::Quad) => (gl_unsigned_int_vec4, 1),
                (ScalarKind::Bool, VectorSize::Bi) => (gl_bool_vec2, 1),
                (ScalarKind::Bool, VectorSize::Tri) => (gl_bool_vec3, 1),
                (ScalarKind::Bool, VectorSize::Quad) => (gl_bool_vec4, 1),
                _ => (0, 1),
            },
            TypeInner::Matrix { columns, rows, .. } => match (columns, rows) {
                (VectorSize::Bi, VectorSize::Bi) => (gl_float_mat2, 1),
                (VectorSize::Tri, VectorSize::Tri) => (gl_float_mat3, 1),
                (VectorSize::Quad, VectorSize::Quad) => (gl_float_mat4, 1),
                _ => (0, 1),
            },
            TypeInner::Image { dim, .. } => match dim {
                naga::ImageDimension::D2 => (gl_sampler_2d, 1),
                naga::ImageDimension::D3 => (gl_sampler_3d, 1),
                naga::ImageDimension::Cube => (gl_sampler_cube, 1),
                _ => (0, 1),
            },
            TypeInner::Array { base, size, .. } => {
                let (base_type, _) = map_type(&arena[*base], arena);
                let count = match size {
                    naga::ArraySize::Constant(c) => c.get() as i32,
                    _ => 1,
                };
                (base_type, count)
            }
            _ => (0, 1),
        }
    }

    if let Some(vs) = &p.vs_module {
        for ep in &vs.entry_points {
            if ep.stage == ShaderStage::Vertex {
                for arg in &ep.function.arguments {
                    if let Some(name) = &arg.name {
                        if let Some(Binding::Location { .. }) = arg.binding {
                            let (gl_type, size) = map_type(&vs.types[arg.ty], &vs.types);
                            if gl_type != 0 {
                                p.active_attributes.push(ActiveInfo {
                                    name: name.clone(),
                                    size,
                                    type_: gl_type,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let mut uni_map = HashMap::new();

    let mut extract_active_uniforms =
        |module: &naga::Module, program_uniforms: &mut HashMap<String, i32>| {
            let mut globals: Vec<_> = module.global_variables.iter().collect();
            globals.sort_by_key(|(handle, _)| handle.index());

            for (handle, var) in globals {
                if matches!(var.space, AddressSpace::Uniform | AddressSpace::Handle) {
                    if let Some(name) = &var.name {
                        let ty = &module.types[var.ty];
                        let (gl_type, size) = map_type(ty, &module.types);
                        if gl_type != 0 {
                            let final_name = if size > 1 {
                                format!("{}[0]", name)
                            } else {
                                name.clone()
                            };

                            let location = handle.index() as i32;

                            if !uni_map.contains_key(&final_name) {
                                uni_map.insert(
                                    final_name.clone(),
                                    ActiveInfo {
                                        name: final_name.clone(),
                                        size,
                                        type_: gl_type,
                                    },
                                );

                                // Only map 'name[0]' if we have a location for 'name'
                                if size > 1 {
                                    if let Some(&loc) = program_uniforms.get(name) {
                                        program_uniforms.insert(final_name, loc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

    if let Some(vs) = &p.vs_module {
        extract_active_uniforms(vs, &mut p.uniforms);
    }
    if let Some(fs) = &p.fs_module {
        extract_active_uniforms(fs, &mut p.uniforms);
    }

    let mut uniforms: Vec<ActiveInfo> = uni_map.into_values().collect();
    uniforms.sort_by(|a, b| a.name.cmp(&b.name));
    p.active_uniforms = uniforms;
}

pub fn ctx_get_active_uniform(
    ctx: u32,
    program: u32,
    index: u32,
    size_ptr: u32,
    type_ptr: u32,
    name_ptr: u32,
    name_capacity: u32,
) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        if index as usize >= p.active_uniforms.len() {
            set_last_error("invalid uniform index");
            return 0;
        }
        let info = &p.active_uniforms[index as usize];

        unsafe {
            *(size_ptr as *mut i32) = info.size;
            *(type_ptr as *mut u32) = info.type_;
        }

        let name_bytes = info.name.as_bytes();
        let name_len = name_bytes.len();

        if name_capacity > 0 {
            let copy_len = std::cmp::min(name_len, name_capacity as usize);
            unsafe {
                let dest = std::slice::from_raw_parts_mut(name_ptr as *mut u8, copy_len);
                dest.copy_from_slice(&name_bytes[0..copy_len]);
            }
        }

        name_len as u32
    } else {
        set_last_error("program not found");
        0
    }
}

pub fn ctx_get_active_attrib(
    ctx: u32,
    program: u32,
    index: u32,
    size_ptr: u32,
    type_ptr: u32,
    name_ptr: u32,
    name_capacity: u32,
) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        if index as usize >= p.active_attributes.len() {
            set_last_error("invalid attribute index");
            return 0;
        }
        let info = &p.active_attributes[index as usize];

        unsafe {
            *(size_ptr as *mut i32) = info.size;
            *(type_ptr as *mut u32) = info.type_;
        }

        let name_bytes = info.name.as_bytes();
        let name_len = name_bytes.len();

        if name_capacity > 0 {
            let copy_len = std::cmp::min(name_len, name_capacity as usize);
            unsafe {
                let dest = std::slice::from_raw_parts_mut(name_ptr as *mut u8, copy_len);
                dest.copy_from_slice(&name_bytes[0..copy_len]);
            }
        }

        name_len as u32
    } else {
        set_last_error("program not found");
        0
    }
}

/// Store shader table indices for direct calling.
/// Called from JS after shader WASM instances are created and registered in the function table.
pub fn ctx_register_shader_indices(ctx: u32, program: u32, vs_idx: u32, fs_idx: u32) -> u32 {
    use super::types::{ERR_INVALID_HANDLE, ERR_OK};

    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if let Some(prog) = ctx_obj.programs.get_mut(&program) {
        prog.vs_table_idx = Some(vs_idx);
        prog.fs_table_idx = Some(fs_idx);
        ERR_OK
    } else {
        set_last_error("invalid program handle");
        ERR_INVALID_HANDLE
    }
}
