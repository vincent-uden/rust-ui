use tracing::error;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PixelInfo {
    entity_id: f32,
    draw_id: f32,
    prim_id: f32,
}

/// Picks entities in a sketch if the mouse cursor is over it on the screen
pub struct EntityPicker {
    fbo: u32,
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
                gl::RGB32F as i32,
                window_width,
                window_height,
                0,
                gl::RGB,
                gl::FLOAT,
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

    pub fn read_pixel(&self) -> PixelInfo {
        todo!()
    }
}
