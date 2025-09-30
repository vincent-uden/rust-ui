use std::ffi::c_void;

use rust_ui::shader::Shader;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct PixelInfo {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Picks entities in a sketch if the mouse cursor is over it on the screen
pub struct EntityPicker {
    pub fbo: u32,
    picking_texture: u32,
    depth_texture: u32,
}

impl EntityPicker {
    pub fn new(window_width: i32, window_height: i32) -> Self {
        let mut fbo: u32 = 0;
        let mut picking_texture: u32 = 0;
        let mut depth_texture: u32 = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            // Picking buffer
            gl::GenTextures(1, &mut picking_texture);
            gl::BindTexture(gl::TEXTURE_2D, picking_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                window_width,
                window_height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                picking_texture,
                0,
            );
            // Depth buffer
            gl::GenTextures(1, &mut depth_texture);
            gl::BindTexture(gl::TEXTURE_2D, depth_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as i32,
                window_width,
                window_height,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_texture,
                0,
            );
            gl::ReadBuffer(0);
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                error!("Framebuffer error: status: {:?}", status);
            } else {
                debug!("Framebuffer successfully created");
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Self {
            fbo,
            picking_texture,
            depth_texture,
        }
    }

    pub fn read_pixel(&self, x: i32, y: i32) -> PixelInfo {
        let mut pixel = PixelInfo::default();
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);

            gl::ReadPixels(
                x,
                y,
                1,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                (&mut pixel) as *mut _ as *mut c_void,
            );

            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }
        return pixel;
    }

    pub fn enable_writing(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
        }
    }

    pub fn disable_writing(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }
    }

    pub fn dump_to_png(&self, width: i32, height: i32, path: &str) -> anyhow::Result<()> {
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::ReadPixels(
                0,
                0,
                width,
                height,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut c_void,
            );
            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }

        let mut flipped = vec![0u8; pixels.len()];
        for y in 0..height {
            let src_row = (height - 1 - y) * width * 4;
            let dst_row = y * width * 4;
            flipped[dst_row as usize..(dst_row + width * 4) as usize]
                .copy_from_slice(&pixels[src_row as usize..(src_row + width * 4) as usize]);
        }

        image::save_buffer(
            path,
            &flipped,
            width as u32,
            height as u32,
            image::ColorType::Rgba8,
        )?;
        info!("Saved framebuffer to {}", path);
        Ok(())
    }
}
