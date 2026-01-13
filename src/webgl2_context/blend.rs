use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

pub fn ctx_blend_func(ctx: u32, sfactor: u32, dfactor: u32) -> u32 {
    ctx_blend_func_separate(ctx, sfactor, dfactor, sfactor, dfactor)
}

pub fn ctx_blend_func_separate(
    ctx: u32,
    src_rgb: u32,
    dst_rgb: u32,
    src_alpha: u32,
    dst_alpha: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    ctx_obj.blend_state.src_rgb = src_rgb;
    ctx_obj.blend_state.dst_rgb = dst_rgb;
    ctx_obj.blend_state.src_alpha = src_alpha;
    ctx_obj.blend_state.dst_alpha = dst_alpha;

    ERR_OK
}

pub fn ctx_blend_equation(ctx: u32, mode: u32) -> u32 {
    ctx_blend_equation_separate(ctx, mode, mode)
}

pub fn ctx_blend_equation_separate(ctx: u32, mode_rgb: u32, mode_alpha: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    ctx_obj.blend_state.eq_rgb = mode_rgb;
    ctx_obj.blend_state.eq_alpha = mode_alpha;

    ERR_OK
}

pub fn ctx_blend_color(ctx: u32, r: f32, g: f32, b: f32, a: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    ctx_obj.blend_state.color = [r, g, b, a];
    ERR_OK
}
