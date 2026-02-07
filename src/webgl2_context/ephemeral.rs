use super::types::Context;
use std::cell::RefCell;

thread_local! {
    static TLS_STRING_ARENA: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static TLS_BLOB_ARENA: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static TLS_SMALL_ARENA: RefCell<Vec<u8>> = RefCell::new(vec![0u8; 128 + 16]);
}

/// Write the 16-byte ephemeral header at and before the payload pointer.
/// bytes [ptr-16..ptr-13]: len: u32
/// bytes [ptr-12..ptr-1]: reserved (0)
fn write_header(arena: &mut [u8], offset: usize, len: u32) {
    let header = &mut arena[offset..offset + 16];
    header[0..4].copy_from_slice(&len.to_le_bytes());
    header[4..16].fill(0);
}

/// Allocate from small fixed arena (up to 128 bytes).
/// Returns a pointer to the payload, or 0 on error.
pub fn alloc_small(ctx: &mut Context, size: usize) -> u32 {
    if size > 128 {
        return 0;
    }
    // small_arena is pre-allocated with 144 bytes
    write_header(&mut ctx.small_arena, 0, size as u32);
    (ctx.small_arena.as_ptr() as u32) + 16
}

/// Allocate from growable blob arena.
/// Returns a pointer to the payload, or 0 on error.
pub fn alloc_blob(ctx: &mut Context, size: usize) -> u32 {
    let total_size = size + 16;
    if ctx.blob_arena.len() < total_size {
        ctx.blob_arena.resize(total_size, 0);
    }
    write_header(&mut ctx.blob_arena, 0, size as u32);
    (ctx.blob_arena.as_ptr() as u32) + 16
}

/// Allocate from string arena and copy the string into it.
/// Returns a pointer to the payload, or 0 on error.
pub fn alloc_string(ctx: &mut Context, s: &str) -> u32 {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let total_needed = len + 16;
    
    if ctx.string_arena.len() < total_needed {
        ctx.string_arena.resize(total_needed, 0);
    }
    
    write_header(&mut ctx.string_arena, 0, len as u32);
    ctx.string_arena[16..16 + len].copy_from_slice(bytes);
    (ctx.string_arena.as_ptr() as u32) + 16
}

/// Allocate from thread-local string arena.
pub fn alloc_tls_string(s: &str) -> u32 {
    TLS_STRING_ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        let bytes = s.as_bytes();
        let len = bytes.len();
        let total_needed = len + 16;
        if arena.len() < total_needed {
            arena.resize(total_needed, 0);
        }
        write_header(&mut arena, 0, len as u32);
        arena[16..16 + len].copy_from_slice(bytes);
        (arena.as_ptr() as u32) + 16
    })
}

/// Allocate from thread-local blob arena.
pub fn alloc_tls_blob(size: usize) -> u32 {
    TLS_BLOB_ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        let total_size = size + 16;
        if arena.len() < total_size {
            arena.resize(total_size, 0);
        }
        write_header(&mut arena, 0, size as u32);
        (arena.as_ptr() as u32) + 16
    })
}
