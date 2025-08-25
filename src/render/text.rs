use std::{collections::HashMap, ffi::c_void, path::Path};

use anyhow::{Result, anyhow};
use freetype as ft;
use gl::types::GLuint;
use taffy::AvailableSpace;

use crate::{
    geometry::Vector,
    render::{Color, Text},
    shader::Shader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    character: char,
    font_size: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Character {
    atlas_coords: Vector<f32>,
    atlas_size: Vector<f32>,
    size: Vector<i32>,
    bearing: Vector<i32>,
    advance: f32,
    ascent: f32,
    descent: f32,
}

#[derive(Debug)]
pub struct FontAtlas {
    texture_id: GLuint,
    size: Vector<i32>,
    characters: HashMap<char, Character>,
    current_x: i32,
    current_y: i32,
    line_height: i32,
}

#[derive(Debug)]
pub struct TextLine {
    position: Vector<f32>,
    size: Vector<f32>,
    contents: String,
}

pub struct TextRenderer {
    shader: Shader,
    quad_vao: GLuint,
    quad_vbo: GLuint,
    ft_library: ft::Library,
    ft_face: ft::Face,
    atlases: HashMap<u32, FontAtlas>,
}

impl std::fmt::Debug for TextRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextRenderer")
            .field("shader", &self.shader)
            .field("quad_vao", &self.quad_vao)
            .field("quad_vbo", &self.quad_vbo)
            .field("atlases", &self.atlases)
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

        let atlases = HashMap::new();
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
            atlases,
        })
    }

    fn get_or_create_atlas(&mut self, font_size: u32) -> &mut FontAtlas {
        if !self.atlases.contains_key(&font_size) {
            let atlas_size = Vector::new(512, 512);
            let mut texture_id: GLuint = 0;

            unsafe {
                gl::GenTextures(1, &mut texture_id);
                gl::BindTexture(gl::TEXTURE_2D, texture_id);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    atlas_size.x,
                    atlas_size.y,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            let atlas = FontAtlas {
                texture_id,
                size: atlas_size,
                characters: HashMap::new(),
                current_x: 2,
                current_y: 2,
                line_height: 0,
            };

            self.atlases.insert(font_size, atlas);
        }

        self.atlases.get_mut(&font_size).unwrap()
    }

    fn load_character(&mut self, character: char, font_size: u32) -> Result<Character> {
        let atlas = self.get_or_create_atlas(font_size);

        if let Some(&char_info) = atlas.characters.get(&character) {
            return Ok(char_info);
        }

        self.ft_face.set_pixel_sizes(0, font_size)?;
        self.ft_face
            .load_char(character as usize, ft::face::LoadFlag::DEFAULT)?;

        let glyph = self.ft_face.glyph();
        glyph.render_glyph(ft::render_mode::RenderMode::Normal)?;

        let bitmap = glyph.bitmap();
        let glyph_width = bitmap.width();
        let glyph_height = bitmap.rows();
        let bitmap_left = glyph.bitmap_left();
        let bitmap_top = glyph.bitmap_top();
        let advance_x = (glyph.advance().x >> 6) as f32;
        let buffer_ptr = bitmap.buffer().as_ptr();
        let buffer_empty = bitmap.buffer().is_empty();

        let atlas = self.get_or_create_atlas(font_size);

        // Check if we need to move to next line
        if atlas.current_x + glyph_width + 2 > atlas.size.x {
            atlas.current_x = 2;
            atlas.current_y += atlas.line_height + 2;
            atlas.line_height = 0;
        }

        // Check if we've run out of space
        if atlas.current_y + glyph_height + 2 > atlas.size.y {
            return Err(anyhow!("Atlas full for font size {}", font_size));
        }

        // Copy glyph bitmap to atlas
        if !buffer_empty {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, atlas.texture_id);
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    atlas.current_x,
                    atlas.current_y,
                    glyph_width,
                    glyph_height,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    buffer_ptr as *const c_void,
                );
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
        }

        // Calculate UV coordinates
        let u1 = atlas.current_x as f32 / atlas.size.x as f32;
        let v1 = atlas.current_y as f32 / atlas.size.y as f32;
        let u2 = (atlas.current_x + glyph_width) as f32 / atlas.size.x as f32;
        let v2 = (atlas.current_y + glyph_height) as f32 / atlas.size.y as f32;

        let char_info = Character {
            atlas_coords: Vector::new(u1, v1),
            atlas_size: Vector::new(u2 - u1, v2 - v1),
            size: Vector::new(glyph_width, glyph_height),
            bearing: Vector::new(bitmap_left, bitmap_top),
            advance: advance_x,
            ascent: bitmap_top as f32,
            descent: (glyph_height - bitmap_top) as f32,
        };

        atlas.line_height = atlas.line_height.max(glyph_height);
        atlas.current_x += glyph_width + 2;
        atlas.characters.insert(character, char_info);

        Ok(char_info)
    }

    pub fn draw_line(
        &mut self,
        text: &str,
        position: Vector<f32>,
        font_size: u32,
        scale: f32,
        color: Color,
    ) {
        self.shader.use_shader();
        let text_unit = 0;
        self.shader.set_uniform("text", &text_unit);
        let color_vec = glm::make_vec3(&[color.r, color.g, color.b]);
        self.shader.set_uniform("textColor", &color_vec);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.quad_vao);
        }

        let size = self.measure_text_size(text, font_size);
        let mut x = position.x;
        let baseline_y = position.y + size.y * 0.8;

        let atlas_texture_id = if let Some(atlas) = self.atlases.get(&font_size) {
            atlas.texture_id
        } else {
            // Create atlas if it doesn't exist by loading first character
            if let Some(first_char) = text.chars().next() {
                if self.load_character(first_char, font_size).is_err() {
                    return;
                }
                self.atlases[&font_size].texture_id
            } else {
                return;
            }
        };

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, atlas_texture_id);
        }

        for c in text.chars() {
            let ch = match self.load_character(c, font_size) {
                Ok(ch) => ch,
                Err(_) => continue,
            };

            // Round to nearest pixel for crisp text rendering
            let xpos = f32::floor(x + ch.bearing.x as f32 * scale + 0.5);
            let ypos = f32::floor(baseline_y - ch.bearing.y as f32 * scale + 0.5);

            let w = ch.size.x as f32 * scale;
            let h = ch.size.y as f32 * scale;

            // Calculate atlas UV coordinates
            let u1 = ch.atlas_coords.x;
            let v1 = ch.atlas_coords.y;
            let u2 = ch.atlas_coords.x + ch.atlas_size.x;
            let v2 = ch.atlas_coords.y + ch.atlas_size.y;

            // Each vertex: [x, y, tex_x, tex_y]
            let vertices: [[f32; 4]; 6] = [
                [xpos, ypos + h, u1, v2],
                [xpos, ypos, u1, v1],
                [xpos + w, ypos, u2, v1],
                [xpos, ypos + h, u1, v2],
                [xpos + w, ypos, u2, v1],
                [xpos + w, ypos + h, u2, v2],
            ];

            unsafe {
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

            x += ch.advance * scale;
        }

        unsafe {
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn draw_in_box(
        &mut self,
        text: Text,
        position: Vector<f32>,
        size: taffy::geometry::Size<f32>,
    ) {
        for line in self.layout_text(
            taffy::Size {
                width: AvailableSpace::Definite(size.width),
                height: AvailableSpace::Definite(size.height),
            },
            text.text,
            text.font_size,
        ) {
            self.draw_line(
                &line.contents,
                position + line.position,
                text.font_size,
                1.0,
                text.color,
            );
        }
    }

    pub fn measure_text_size(&mut self, text: &str, font_size: u32) -> Vector<f32> {
        if text.is_empty() {
            return Vector::new(0.0, font_size as f32);
        }

        let mut width: f32 = 0.0;
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;

        for c in text.chars() {
            let loaded = self.load_character(c, font_size).unwrap();
            width += loaded.advance;
            max_ascent = max_ascent.max(loaded.ascent);
            max_descent = max_descent.max(loaded.descent);
        }

        let mut height = max_ascent + max_descent;
        if height == 0.0 {
            height = font_size as f32; // Fallback if no height data
        }

        Vector::new(width, height)
    }

    pub fn layout_text(
        &mut self,
        available_space: taffy::geometry::Size<taffy::style::AvailableSpace>,
        text: String,
        font_size: u32,
    ) -> Vec<TextLine> {
        let mut out = vec![];

        let mut y = 0.0;
        let mut current_line = String::new();
        let mut pending_line = String::new();
        for word in split_with_trailing_whitespace(&text) {
            pending_line.push_str(word);
            let pending_size = self.measure_text_size(&pending_line, font_size);
            // TODO: Think about non-definite cases
            if pending_size.x
                > (match available_space.width {
                    AvailableSpace::Definite(px) => px,
                    AvailableSpace::MinContent => 0.0,
                    AvailableSpace::MaxContent => 9999.0,
                })
                && !current_line.is_empty()
            {
                let size = self.measure_text_size(&current_line, font_size);
                out.push(TextLine {
                    position: Vector::new(0.0, y),
                    size,
                    contents: current_line.clone(),
                });
                y += (font_size as f32) * 1.2;
                current_line.clear();
                pending_line.clear();
                pending_line.push_str(word);
            }
            current_line.clone_from(&pending_line);
        }
        if !current_line.is_empty() {
            out.push(TextLine {
                position: Vector::new(0.0, y),
                size: self.measure_text_size(&current_line, font_size),
                contents: current_line.clone(),
            });
        }

        out
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        // Free OpenGL atlas textures to avoid memory leaks
        for atlas in self.atlases.values() {
            unsafe {
                gl::DeleteTextures(1, &atlas.texture_id);
            }
        }

        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
            gl::DeleteBuffers(1, &self.quad_vbo);
        }
    }
}

pub fn total_size(lines: &[TextLine]) -> Vector<f32> {
    let mut out = Vector::<f32>::zero();

    for line in lines {
        out.x = out.x.max(line.position.x + line.size.x);
        out.y = out.y.max(line.position.y + line.size.y);
    }

    out
}

fn split_with_trailing_whitespace(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < s.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        while i < s.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        while i < s.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        parts.push(&s[start..i]);
    }
    parts
}

#[cfg(test)]
mod tests {
    use crate::geometry::Vector;
    use freetype as ft;
    use image::{GrayImage, ImageBuffer};
    use std::path::Path;

    fn get_test_font_path() -> &'static Path {
        Path::new("./assets/fonts/LiberationMono.ttf")
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
