use std::{collections::HashMap, ffi::c_void, hash::Hash, path::Path};

use gl::types::GLuint;

use anyhow::{Result, anyhow};
use image::{GenericImageView, ImageReader};

use crate::{geometry::{Rect, Vector}, shader::Shader};

/// Used to draw sprites with GPU instancing
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpriteInstance {
    position: [f32; 2],
    size: [f32; 2], 
    atlas_coords: [f32; 2],
    atlas_size: [f32; 2],
}

pub trait SpriteKey: Hash + Clone + Copy {}

#[derive(Debug)]
pub struct SpriteAtlas<K>
where
    K: SpriteKey,
{
    texture_id: GLuint,
    map: HashMap<K, Rect<u32>>,
}

impl<K: SpriteKey> SpriteAtlas<K> {
    // TODO: How do we populate the atlas map?
    pub fn from_path(path: &Path) -> Result<Self> {
        // Load image
        let img = match ImageReader::open(path)?.decode()? {
            image::DynamicImage::ImageRgba8(image_buffer) => image_buffer,
            img => img.to_rgba8(),
        };
        let atlas_size = Vector::new(img.dimensions().0, img.dimensions().1);
        let mut texture_id: GLuint = 0;

        let img_data = img.into_raw();
            let img_ptr: *const c_void = img_data.as_ptr() as *const c_void;

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                gl::RGBA as i32, 
                atlas_size.x as i32, 
                atlas_size.y as i32, 
                0, 
                gl::RGBA, 
                gl::UNSIGNED_BYTE, 
                img_ptr
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(Self {
            texture_id,
            map: HashMap::new()
        })
    }
}

#[derive(Debug)]
pub struct SpriteRenderer<K>
where
    K: SpriteKey,
{
    shader: Shader,
    quad_vao: GLuint,
    quad_vbo: GLuint,
    /// Will eventually be used to draw all possible icons at once
    instance_vbo: GLuint,
    atlas: SpriteAtlas<K>,
}

impl<K: SpriteKey> SpriteRenderer<K> {
    /// This is lifted in large part from the GPU instanced text rendering which also uses atlases
    /// to render text
    pub fn new(shader: Shader, atlas_path: &Path) -> Result<Self> {
        let mut quad_vao = 0;
        let mut quad_vbo = 0;
        let mut instance_vbo = 0;

        #[rustfmt::skip]
        let quad_vertices: [f32; 24] = [
            // pos   // tex
            0.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            
            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0
        ];

        unsafe {
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut quad_vbo);
            gl::GenBuffers(1, &mut instance_vbo);

            gl::BindVertexArray(quad_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * quad_vertices.len()) as isize,
                quad_vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );
            
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<SpriteInstance>() as i32,
                std::ptr::null(),
            );
            gl::VertexAttribDivisor(1, 1);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<SpriteInstance>() as i32,
                (2 * std::mem::size_of::<f32>()) as *const c_void,
            );
            gl::VertexAttribDivisor(2, 1);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<SpriteInstance>() as i32,
                (4 * std::mem::size_of::<f32>()) as *const c_void,
            );
            gl::VertexAttribDivisor(3, 1);
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<SpriteInstance>() as i32,
                (6 * std::mem::size_of::<f32>()) as *const c_void,
            );
            gl::VertexAttribDivisor(4, 1);
            
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        todo!()
    }

    pub fn draw(&self, key: &K, location: Rect<f32>) {}
}

impl<K: SpriteKey> Drop for SpriteRenderer<K> {
    fn drop(&mut self) {
        // TODO: Drop the alloated opengl textures, buffers and vertex arrays
    }
}
