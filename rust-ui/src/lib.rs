#![allow(
    clippy::uninlined_format_args,
    clippy::too_many_arguments,
    clippy::uninlined_format_args
)]

use glfw::Context;
use std::{
    cell::{RefCell, RefMut},
    env,
};
use taffy::{NodeId, Style, TaffyTree};

use crate::render::renderer::{AppState, NodeContext};

pub mod geometry;
pub mod perf_overlay;
pub mod render;
pub mod shader;

pub fn init_open_gl(
    width: u32,
    height: u32,
    resizable: bool,
) -> (
    glfw::Glfw,
    glfw::PWindow,
    glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // Configure OpenGL context based on target architecture
    #[cfg(target_arch = "aarch64")]
    {
        // Raspberry Pi / ARM configuration - let driver choose defaults
        // No OpenGL version or profile hints for maximum compatibility
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // x86/x64 desktop configuration
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
    }

    glfw.window_hint(glfw::WindowHint::Resizable(resizable));

    let (mut window, events) = glfw
        .create_window(width, height, "App", glfw::WindowMode::Windowed)
        .unwrap();

    window.make_current();
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);

    gl::load_with(|ptr| {
        let f = window.get_proc_address(ptr);
        match f {
            Some(f) => f as *const _,
            None => std::ptr::null(),
        }
    });

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Enable(gl::BLEND);
        gl::Enable(gl::MULTISAMPLE);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    (glfw, window, events)
}

pub fn print_env() {
    let cwd = env::current_dir().unwrap();
    println!("Current dir: {}", cwd.display());

    println!("--- Environment ---");
    for (k, v) in env::vars() {
        println!("{k}={v}");
    }
}

pub fn ui<T>(tree: &RefCell<TaffyTree<NodeContext<T>>>, style: &str, children: &[NodeId]) -> NodeId
where
    T: AppState + Default,
{
    let (style, context) = parse_style(style);
    let mut tree = tree.borrow_mut();
    let parent = tree.new_leaf_with_context(style, context).unwrap();
    for child in children {
        tree.add_child(parent, *child).unwrap();
    }
    return parent;
}

pub fn parse_style<T>(style: &str) -> (Style, NodeContext<T>)
where
    T: AppState + Default,
{
    (Style::DEFAULT, NodeContext::default())
}

#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;

    use crate::{
        render::renderer::{AppState, NodeContext},
        ui,
    };

    #[derive(Default)]
    struct DummyState {}

    impl AppState for DummyState {
        fn generate_layout(
            &mut self,
            _: crate::geometry::Vector<f32>,
        ) -> Vec<crate::render::renderer::RenderLayout<Self>> {
            todo!()
        }
    }

    #[test]
    pub fn ui_shorthand_doesnt_deadlock() {
        let tree: taffy::TaffyTree<NodeContext<DummyState>> = taffy::TaffyTree::new();
        let tree = RefCell::new(tree);

        ui(&tree, "", &[ui(&tree, "", &[]), ui(&tree, "", &[])]);
    }
}
