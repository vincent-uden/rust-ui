# Line Renderer Usage

The Line Renderer allows you to draw individual lines on the screen with customizable color and thickness.

## Basic Usage

```rust
use rust_ui::{
    geometry::Vector,
    render::{Color, line::LineRenderer},
    shader::Shader,
};

// 1. Create the line shader (usually done during app initialization)
let line_shader = Shader::from_paths(
    &PathBuf::from("shaders/glsl330/line.vs"),
    &PathBuf::from("shaders/glsl330/line.frag"),
    None,
).unwrap();

// 2. Create the LineRenderer
let line_renderer = LineRenderer::new(line_shader);

// 3. Draw lines
let window_size = Vector::new(800.0, 600.0);

line_renderer.draw(
    Vector::new(100.0, 100.0),  // start point
    Vector::new(200.0, 150.0),  // end point
    Color::new(1.0, 0.0, 0.0, 1.0), // red color (RGBA)
    2.0,                        // line thickness
    window_size,               // window dimensions for projection
);
```

## Integration with Renderer

When using the main `Renderer<T>` struct, the line renderer is already included:

```rust
// During renderer initialization, pass the line_shader
let mut state = Renderer::new(
    rect_shader,
    text_shader,
    line_shader,    // <- Line shader parameter
    initial_state,
);

// Access the line renderer
state.line_r.draw(
    Vector::new(50.0, 50.0),
    Vector::new(200.0, 100.0),
    COLOR_DANGER,  // Use predefined colors
    2.0,
    Vector::new(state.width as f32, state.height as f32),
);
```

## Available Features

### Colors
Use predefined colors from the `render` module:
- `COLOR_DANGER` - Red
- `COLOR_SUCCESS` - Green  
- `COLOR_PRIMARY` - Blue
- `COLOR_LIGHT` - Light color
- `NORD0` - `NORD15` - Full Nord color palette

### Line Properties
- **Start/End Points**: `Vector<f32>` with x, y coordinates
- **Color**: `Color` struct with RGBA values (0.0-1.0 range)
- **Thickness**: `f32` line width in pixels
- **Window Size**: Required for proper coordinate projection

## Shaders

The line renderer requires two shader files:
- `line.vs` - Vertex shader for line positioning
- `line.frag` - Fragment shader for line coloring

Both GLSL 3.30 and GLES 3.00 variants are provided in the `shaders/` directory.

## Technical Notes

- Lines are rendered using OpenGL's `GL_LINES` primitive
- Screen coordinates: (0,0) is top-left, Y increases downward
- Lines are anti-aliased through OpenGL's default line rendering
- Each `draw()` call renders a single line segment
- For performance with many lines, consider batching (future enhancement)