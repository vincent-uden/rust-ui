use std::{collections::HashMap, ffi::c_void, fs, hash::Hash, path::Path, str::FromStr};

use gl::types::GLuint;

use anyhow::{Result, anyhow};
use image::ImageReader;

use crate::{geometry::{Rect, Vector}, shader::Shader};

/// Used to draw sprites with GPU instancing
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpriteInstance {
    /// Where on the screen should this be draw
    position: [f32; 2],
    /// How big should it be on screen
    size: [f32; 2], 
    /// Where in the atlas is it located
    atlas_coords: [f32; 2],
    /// How big is it in the atlas
    atlas_size: [f32; 2],
}

pub trait SpriteKey: Hash + Clone + FromStr + PartialEq + Eq {}

#[derive(Debug)]
pub struct SpriteAtlas<K>
where
    K: SpriteKey,
{
    texture_id: GLuint,
    map: HashMap<K, Rect<f32>>,
}

impl<K: SpriteKey> SpriteAtlas<K> {
    fn parse_legend(contents: &str) -> Result<Vec<(K, Rect<f32>)>> {
        let mut out = vec![];
        // Skip csv header, we know the layout
        for (i, l) in contents.lines().skip(1).enumerate() {
            let parts: Vec<&str> = l.split(",").collect();

            let x0 = Vector::new(parts[1].parse()?, parts[2].parse()?);
            let x1 = Vector::new(parts[3].parse::<f32>()? + x0.x, parts[4].parse::<f32>()? + x0.y);

            out.push((K::from_str(parts[0]).map_err(|_| anyhow!("Couldn't parse icon name on line {}: {}", i+1, l))?, Rect { x0, x1, }));
        }

        Ok(out)
    }

    /// The legend is a csv file containing the names and bounding boxes of the different textures
    pub fn from_path(img_path: &Path, legend_path: &Path) -> Result<Self> {
        let img = match ImageReader::open(img_path)?.decode()? {
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

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let atlas_size = Vector::new(atlas_size.x as f32, atlas_size.y as f32);
        let mut map = HashMap::new();
        for (key, location) in Self::parse_legend(&fs::read_to_string(legend_path)?)? {
            let normalized_rect = Rect {
                x0: Vector::new(location.x0.x / atlas_size.x, location.x0.y / atlas_size.y),
                x1: Vector::new(location.x1.x / atlas_size.x, location.x1.y / atlas_size.y),
            };
            map.insert(key, normalized_rect);
        }

        Ok(Self {
            texture_id,
            map,
        })
    }

    pub fn empty() -> Self {
        Self {
            texture_id: u32::MAX,
            map: HashMap::new(),
        }
    }
}

impl<K> Drop for SpriteAtlas<K> where K: SpriteKey {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture_id);
        }
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
    pub atlas: SpriteAtlas<K>,
}

impl<K: SpriteKey> SpriteRenderer<K> {
    /// This is lifted in large part from the GPU instanced text rendering which also uses atlases
    /// to render text
    pub fn new(shader: Shader, atlas: SpriteAtlas<K>) -> Self {
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

        Self {
            shader,
            quad_vao,
            quad_vbo,
            instance_vbo,
            atlas,
        }
    }

    pub fn draw(&self, key: &K, location: Rect<f32>) {
        if let Some(bbox) = self.atlas.map.get(key) {
            let instances = [SpriteInstance {
                position: [(location.x0.x + 0.5).floor(), (location.x0.y + 0.5).floor()],
                size: [(location.size().x + 0.5).floor(), (location.size().y + 0.5).floor()],
                atlas_coords: [bbox.x0.x, bbox.x0.y],
                atlas_size: [bbox.width(), bbox.height()]
            }];

            self.shader.use_shader();
            self.shader.set_uniform("text", &0);
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.atlas.texture_id);
                gl::BindVertexArray(self.quad_vao);

                gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER, 
                    (std::mem::size_of::<SpriteInstance>() * instances.len()) as isize, 
                    instances.as_ptr() as *const c_void, 
                    gl::DYNAMIC_DRAW
                );

                gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, instances.len() as i32);

                gl::BindVertexArray(0);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
        }
    }
}

impl<K: SpriteKey> Drop for SpriteRenderer<K> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
            gl::DeleteBuffers(1, &self.quad_vbo);
            gl::DeleteBuffers(1, &self.instance_vbo);
        }
    }
}
