use std::{collections::HashMap, ffi::c_void, path::Path};

use anyhow::{Result, anyhow};
use freetype as ft;
use gl::types::GLuint;

use crate::{
    geometry::Vector,
    shader::Shader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    character: char,
    font_size: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Character {
    texture_id: GLuint,
    size: Vector<i32>,
    bearing: Vector<i32>,
    advance: i32,
}

pub struct TextRenderer {
    shader: Shader,
    quad_vao: GLuint,
    quad_vbo: GLuint,
    ft_library: ft::Library,
    ft_face: ft::Face,
    characters: HashMap<GlyphKey, Character>,
}

impl std::fmt::Debug for TextRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextRenderer")
            .field("shader", &self.shader)
            .field("quad_vao", &self.quad_vao)
            .field("quad_vbo", &self.quad_vbo)
            .field("characters", &self.characters)
            .finish_non_exhaustive()
    }
}

impl TextRenderer {
    pub fn new(shader: Shader, font_path: &Path) -> Result<Self> {
        let ft_library =
            ft::Library::init().map_err(|_| anyhow!("Failed to initialize FreeType library"))?;
        
        let ft_face = ft_library
            .new_face(font_path, 0)
            .map_err(|_| anyhow!("Failed to load font"))?;
        
        // FreeType expects 1-byte alignment for proper glyph rendering
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }
        
        let characters = HashMap::new();
        let mut quad_vao = 0;
        let mut quad_vbo = 0;
        
        unsafe {
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut quad_vbo);
            gl::BindVertexArray(quad_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * 6 * 4) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
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
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        
        Ok(Self {
            shader,
            quad_vao,
            quad_vbo,
            ft_library,
            ft_face,
            characters,
        })
    }

    fn load_character(&mut self, character: char, font_size: u32) -> bool {
        if let Err(_) = self.ft_face.set_pixel_sizes(0, font_size) {
            return false;
        }
        
        if let Err(_) = self
            .ft_face
            .load_char(character as usize, ft::face::LoadFlag::DEFAULT)
        {
            return false;
        }
        
        let glyph = self.ft_face.glyph();
        if let Err(_) = glyph.render_glyph(ft::render_mode::RenderMode::Normal) {
            return false;
        }
        
        let bitmap = glyph.bitmap();
        
        let mut texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                bitmap.width(),
                bitmap.rows(),
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                bitmap.buffer().as_ptr() as *const c_void,
            );
            
            // Use CLAMP_TO_EDGE to avoid artifacts when sampling near the edge
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            // NEAREST filtering works best for pixel-perfect text
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        
        let key = GlyphKey {
            character,
            font_size,
        };
        
        let char_info = Character {
            texture_id: texture,
            size: Vector::new(bitmap.width(), bitmap.rows()),
            bearing: Vector::new(glyph.bitmap_left(), glyph.bitmap_top()),
            // Shift by 6 to convert from 1/64 pixels (FreeType's format) to pixels
            advance: (glyph.advance().x >> 6) as i32,
        };
        
        self.characters.insert(key, char_info);
        
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        
        true
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        position: Vector<f32>,
        font_size: u32,
        scale: f32,
        color: [f32; 3],
    ) {
        self.shader.use_shader();
        self.shader.set_uniform_i32("text", 0);
        self.shader
            .set_uniform_vec3("textColor", &glm::make_vec3(&color));
        
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.quad_vao);
        }
        
        let mut x = position.x;
        let baseline_y = position.y;
        
        for c in text.chars() {
            let key = GlyphKey {
                character: c,
                font_size,
            };
            
            let ch = match self.characters.get(&key) {
                Some(ch) => *ch,
                None => {
                    if !self.load_character(c, font_size) {
                        continue;
                    }
                    self.characters[&key]
                }
            };
            
            // Round to nearest pixel for crisp text rendering
            let xpos = f32::floor(x + ch.bearing.x as f32 * scale + 0.5);
            let ypos = f32::floor(baseline_y - ch.bearing.y as f32 * scale + 0.5);
            
            let w = ch.size.x as f32 * scale;
            let h = ch.size.y as f32 * scale;
            
            // Each vertex: [x, y, tex_x, tex_y]
            let vertices: [[f32; 4]; 6] = [
                [xpos, ypos + h, 0.0, 1.0],
                [xpos, ypos, 0.0, 0.0],
                [xpos + w, ypos, 1.0, 0.0],
                
                [xpos, ypos + h, 0.0, 1.0],
                [xpos + w, ypos, 1.0, 0.0],
                [xpos + w, ypos + h, 1.0, 1.0],
            ];
            
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.quad_vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (std::mem::size_of::<f32>() * 4 * 6) as isize,
                    vertices.as_ptr() as *const c_void,
                );
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
            
            x += ch.advance as f32 * scale;
        }
        
        unsafe {
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn measure_text_size(&mut self, text: &str, font_size: u32) -> Vector<f32> {
        if text.is_empty() {
            return Vector::new(0.0, font_size as f32);
        }

        if let Err(_) = self.ft_face.set_pixel_sizes(0, font_size) {
            // Fallback estimation if font size setting fails
            return Vector::new(text.len() as f32 * font_size as f32 * 0.6, font_size as f32);
        }

        let mut width: f32 = 0.0;
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;

        for c in text.chars() {
            if let Err(_) = self
                .ft_face
                .load_char(c as usize, ft::face::LoadFlag::RENDER)
            {
                // Fallback for characters that can't be loaded
                width += font_size as f32 * 0.6;
                continue;
            }

            let glyph = self.ft_face.glyph();

            // FreeType stores advance in 1/64 pixels
            width += (glyph.advance().x >> 6) as f32;

            // Track max ascent and descent for height calculation
            let ascent = glyph.bitmap_top() as f32;
            let descent = glyph.bitmap().rows() as f32 - ascent;

            max_ascent = max_ascent.max(ascent);
            max_descent = max_descent.max(descent);
        }

        let mut height = max_ascent + max_descent;
        if height == 0.0 {
            height = font_size as f32; // Fallback if no height data
        }

        Vector::new(width, height)
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        // Free OpenGL textures to avoid memory leaks
        for (_, ch) in &self.characters {
            unsafe {
                gl::DeleteTextures(1, &ch.texture_id);
            }
        }

        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
            gl::DeleteBuffers(1, &self.quad_vbo);
        }
    }
}

#[cfg(test)]
mod tests {
    use freetype as ft;
    use image::{GrayImage, ImageBuffer};
    use std::path::Path;
    use crate::geometry::Vector;

    fn get_test_font_path() -> &'static Path {
        Path::new("/media/hdd/programming/rust-ui/assets/fonts/LiberationMono.ttf")
    }

    #[test]
    fn test_character_atlas() {
        println!("Testing character atlas generation...");

        let ft_library = ft::Library::init().expect("Failed to initialize FreeType");
        
        let font_path = get_test_font_path();
        let ft_face = ft_library
            .new_face(font_path, 0)
            .expect("Failed to load font face");
        
        let font_size: u32 = 63;
        ft_face
            .set_pixel_sizes(0, font_size)
            .expect("Failed to set font size");

        // Create atlas for ASCII characters 32-126
        let atlas_width: i32 = 512;
        let atlas_height: i32 = 512;
        let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut line_height: i32 = 0;

        // Render printable ASCII characters
        for char in 32..=126 {
            if let Err(_) = ft_face.load_char(char as usize, ft::face::LoadFlag::DEFAULT) {
                println!("Failed to load character: {}", char as u8 as char);
                continue;
            }

            let glyph = ft_face.glyph();
            if let Err(_) = glyph.render_glyph(ft::render_mode::RenderMode::Normal) {
                println!("Failed to render character: {}", char as u8 as char);
                continue;
            }

            let bitmap = glyph.bitmap();

            // Move to next line if we're out of horizontal space
            if x + bitmap.width() > atlas_width {
                x = 0;
                y += line_height + 2;
                line_height = 0;
            }

            // Check if we've run out of vertical space
            if y + bitmap.rows() > atlas_height {
                println!("Atlas too small for all characters");
                break;
            }

            line_height = line_height.max(bitmap.rows());

            // Copy glyph bitmap to atlas
            let buffer = bitmap.buffer();
            if !buffer.is_empty() {
                for row in 0..bitmap.rows() {
                    for col in 0..bitmap.width() {
                        let atlas_x = x + col;
                        let atlas_y = y + row;
                        if atlas_x < atlas_width && atlas_y < atlas_height {
                            let atlas_idx = (atlas_y * atlas_width + atlas_x) as usize;
                            let glyph_idx = (row * bitmap.width() + col) as usize;
                            atlas_data[atlas_idx] = buffer[glyph_idx];
                        }
                    }
                }
            }

            println!(
                "Character '{}': size={}x{}, advance={}, bearing=({},{})",
                char as u8 as char,
                bitmap.width(),
                bitmap.rows(),
                glyph.advance().x >> 6,
                glyph.bitmap_left(),
                glyph.bitmap_top(),
            );

            x += bitmap.width() + 2;
        }

        let img: GrayImage =
            ImageBuffer::from_raw(atlas_width as u32, atlas_height as u32, atlas_data)
                .expect("Failed to create image buffer");

        img.save("character_atlas.png")
            .expect("Failed to write atlas file");
        println!("Character atlas written to character_atlas.png");
    }

    #[test]
    fn test_single_character() {
        println!("Testing single character rendering...");

        let ft_library = ft::Library::init().expect("Failed to initialize FreeType");
        
        let font_path = get_test_font_path();
        let ft_face = ft_library
            .new_face(font_path, 0)
            .expect("Failed to load font face");
        
        let font_size: u32 = 64;
        ft_face
            .set_pixel_sizes(0, font_size)
            .expect("Failed to set font size");

        // Render the letter 'A'
        ft_face
            .load_char('A' as usize, ft::face::LoadFlag::DEFAULT)
            .expect("Failed to load character 'A'");

        let glyph = ft_face.glyph();
        glyph
            .render_glyph(ft::render_mode::RenderMode::Normal)
            .expect("Failed to render character 'A'");

        let bitmap = glyph.bitmap();

        println!("Character 'A' at {}px:", font_size);
        println!("  Bitmap size: {}x{}", bitmap.width(), bitmap.rows());
        println!(
            "  Bearing: ({}, {})",
            glyph.bitmap_left(),
            glyph.bitmap_top()
        );
        println!("  Advance: {}", glyph.advance().x >> 6);
        println!("  Pitch: {}", bitmap.pitch());

        assert!(bitmap.width() > 0, "Bitmap width is zero");
        assert!(bitmap.rows() > 0, "Bitmap height is zero");
        assert!(!bitmap.buffer().is_empty(), "Bitmap buffer is empty");

        if bitmap.width() > 0 && bitmap.rows() > 0 && !bitmap.buffer().is_empty() {
            let img: GrayImage = ImageBuffer::from_raw(
                bitmap.width() as u32,
                bitmap.rows() as u32,
                bitmap.buffer().to_vec(),
            )
            .expect("Failed to create image buffer");

            img.save("character_A.png")
                .expect("Failed to write character file");
            println!("Character 'A' written to character_A.png");
        }
    }
}