#![cfg(feature = "tests-only")]

use raylib::ffi::rlFramebufferAttachTextureType::{RL_ATTACHMENT_RENDERBUFFER, RL_ATTACHMENT_TEXTURE2D};
use raylib::ffi::rlFramebufferAttachType::{RL_ATTACHMENT_COLOR_CHANNEL0, RL_ATTACHMENT_DEPTH};
use raylib::ffi::TraceLogLevel::{LOG_INFO, LOG_WARNING};
use raylib::ffi::{
    rlClearColor, rlClearScreenBuffers, rlDisableColorBlend, rlDisableFramebuffer, rlEnableFramebuffer,
    rlFramebufferAttach, rlFramebufferComplete, rlLoadFramebuffer, rlLoadTexture, rlLoadTextureDepth, Texture2D,
    TraceLog,
};
use raylib::texture::RenderTexture2D;
use std::ffi::c_char;
use std::fs::read_to_string;
use std::path::Path;

pub fn create_rgba16_render_texture(width: i32, height: i32) -> RenderTexture2D {
    // raylib_bath code: https://github.com/raysan5/raylib/blob/4bc8d3761c48f4dcf56f126640da8f3567dc516b/src/rtextures.c#L4246
    let render_texture = unsafe {
        let fbo_id = rlLoadFramebuffer();
        rlEnableFramebuffer(fbo_id);
        let texture_id = rlLoadTexture(
            std::ptr::null(),
            width,
            height,
            raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 as i32,
            1,
        );
        // TODO: lmao try setting useRenderBuffer to false and see what happens
        // let depth_texture_id = rlLoadTextureDepth(width, height, false);
        let depth_texture_id = rlLoadTextureDepth(width, height, true);
        let raw_render_texture = raylib::ffi::RenderTexture2D {
            id: fbo_id,
            texture: Texture2D {
                id: texture_id,
                width,
                height,
                mipmaps: 1,
                format: raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 as i32,
            },
            depth: Texture2D {
                id: depth_texture_id,
                width,
                height,
                mipmaps: 1,
                format: 19i32,
            },
        };
        //TODO: just good practice before attaching to FBOs
        rlClearColor(0, 0, 0, 0);
        rlClearScreenBuffers();
        rlFramebufferAttach(
            fbo_id,
            texture_id,
            RL_ATTACHMENT_COLOR_CHANNEL0 as i32,
            RL_ATTACHMENT_TEXTURE2D as i32,
            0,
        );
        rlFramebufferAttach(
            fbo_id,
            depth_texture_id,
            RL_ATTACHMENT_DEPTH as i32,
            RL_ATTACHMENT_RENDERBUFFER as i32,
            0,
        );
        if rlFramebufferComplete(fbo_id) {
            TraceLog(
                LOG_INFO as i32,
                b"FBO: [ID %i] Framebuffer object created successfully\0"
                    .as_ptr()
                    .cast::<c_char>(),
                fbo_id,
            );
        } else {
            TraceLog(
                LOG_WARNING as i32,
                b"FBO: [ID %i] Framebuffer object is not complete\0"
                    .as_ptr()
                    .cast::<c_char>(),
                fbo_id,
            );
        }
        rlDisableColorBlend();
        //rlDisableColorBlend(); //TODO: consolidate this
        rlDisableFramebuffer();
        RenderTexture2D::from_raw(raw_render_texture)
    };
    render_texture //TODO: I like this because it reminds me how returns work
}

pub fn load_shader_with_includes(path: impl AsRef<Path>) -> String {
    let path = path.as_ref().canonicalize().expect("bad shader path");
    let parent = path.parent().unwrap();
    let src = read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    let mut out = String::new();
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("#include") {
            if let Some(name) = rest.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                let incl = parent.join(name);
                out.push_str(&load_shader_with_includes(incl));
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
