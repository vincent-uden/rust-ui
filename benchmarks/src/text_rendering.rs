use std::{cell::RefCell, path::PathBuf, str::FromStr as _};

use glfw::Context as _;
use rust_ui::{
    geometry::Vector,
    init_open_gl,
    render::{
        COLOR_LIGHT, Text,
        line::LineRenderer,
        rect::RectRenderer,
        renderer::{Anchor, AppState, NodeContext, RenderLayout, Renderer, UiBuilder},
        sprite::{SpriteAtlas, SpriteRenderer},
        text::TextRenderer,
    },
    shader::{Shader, ShaderName},
};
use taffy::{AvailableSpace, Size};

const TEST_TEXT: &'static str = "

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed volutpat ipsum fringilla, hendrerit tortor in, tempus lectus. Fusce interdum odio in mauris rhoncus ornare. Duis eu tortor id tellus bibendum vehicula id non magna. Duis fermentum tincidunt pulvinar. Morbi nec justo in risus maximus aliquet ut vitae risus. Aliquam erat volutpat. Aliquam in vehicula lectus. Sed sapien risus, commodo quis blandit id, facilisis sit amet nulla. Etiam fringilla arcu a viverra aliquet. Cras semper arcu a magna consequat malesuada. Curabitur sagittis nibh varius mauris porttitor mollis. Aliquam erat volutpat. Donec dapibus aliquet ante non ultrices. Sed quis turpis a est eleifend ultrices. Vestibulum consectetur est sed lectus pulvinar placerat. Cras et urna pulvinar, ultrices orci vitae, pellentesque ligula.

Cras consectetur venenatis interdum. Nullam sed imperdiet erat. Integer vel porttitor felis, at iaculis ante. Integer mi sem, bibendum id libero vel, mollis pellentesque massa. Duis hendrerit felis non nisi euismod finibus. Phasellus sed velit maximus, sollicitudin odio dapibus, euismod quam. Cras dignissim ullamcorper urna sit amet bibendum. Vestibulum nec volutpat ipsum. Suspendisse potenti. Integer rhoncus aliquam aliquet.

Quisque justo ligula, efficitur et urna a, efficitur vestibulum elit. Duis at tortor leo. Mauris placerat rhoncus lobortis. Integer nibh nulla, lacinia ac finibus in, vehicula et ipsum. Duis non tempus dolor, ut placerat lacus. Pellentesque sit amet velit sapien. Nulla sapien arcu, dapibus a lacus quis, pellentesque malesuada metus. Maecenas id orci vehicula ex vehicula molestie vel ut nisl. Phasellus vel dui nisl. Donec in tincidunt ante. Etiam accumsan ipsum non dignissim mattis. Integer eu tellus id ligula ullamcorper malesuada. Ut eu est eu augue iaculis efficitur.

Mauris vitae ante auctor, porta risus vel, scelerisque sapien. Quisque facilisis, ex elementum vulputate volutpat, felis eros aliquet tortor, vitae posuere urna ipsum at risus. In imperdiet molestie turpis eget condimentum. Donec vehicula ac magna a fringilla. Ut sollicitudin tincidunt posuere. In id neque aliquam, vulputate augue sed, bibendum quam. Aenean facilisis, arcu eget cursus pulvinar, justo neque euismod velit, ac consequat mauris enim sed ante. Donec vestibulum, ligula et sagittis mollis, libero enim lacinia odio, nec porta dolor odio egestas est. Nulla bibendum fringilla ante, at euismod metus porttitor at. Nam egestas orci libero, vel accumsan quam facilisis eu. Proin quis consequat est. In nisi lorem, dictum eget sapien ut, hendrerit pellentesque odio. Praesent tempor interdum nulla, sit amet tristique ligula dignissim at. Nam metus massa, fermentum at imperdiet eget, venenatis vitae ante. Donec vestibulum erat quam, quis malesuada mi gravida a.

Suspendisse a sollicitudin velit. Nullam gravida, tortor eu viverra fermentum, nibh odio suscipit neque, sed porttitor urna tellus eu ipsum. Pellentesque a hendrerit mauris. Sed mollis elit velit. Pellentesque eu lobortis magna. Quisque nunc arcu, efficitur ac fermentum porttitor, commodo a massa. Morbi in turpis id diam lacinia maximus sit amet a lectus. Nunc quis felis ex. Aenean ultrices maximus leo ac maximus. Fusce sed mi id sapien posuere rhoncus. Fusce in arcu pretium, iaculis est id, aliquam mauris.

Aenean tempor urna nec tortor consectetur lacinia. Vestibulum vitae elit interdum lectus accumsan suscipit. Nullam id accumsan quam, ac faucibus elit. Aenean eleifend scelerisque lectus, at aliquet risus bibendum non. Nulla luctus, elit at posuere laoreet, magna mi fermentum nisl, non vehicula odio nibh a augue. Duis consectetur aliquam dui, ut lacinia tortor faucibus vel. Mauris pellentesque non neque nec cursus. Etiam vel tellus eu mi dignissim iaculis nec sit amet nisl. Nullam purus dui, tristique eu aliquet eget, porttitor quis diam. Nam ante lectus, aliquet ut dolor quis, posuere feugiat massa. Suspendisse mattis tortor mi, vel ultricies nisl porttitor vitae. Fusce sed lacus id sem blandit convallis. Donec quis urna dui.

Aenean ornare luctus ipsum eu tristique. Morbi sit amet sem tempus, laoreet arcu ultricies, ornare diam. Vivamus sed tellus feugiat odio gravida rutrum a sed turpis. Nam elementum eros nisi, eu finibus ex varius eget. Etiam consectetur imperdiet ipsum, porta sollicitudin risus lacinia et. Ut sollicitudin tincidunt leo at scelerisque. Vestibulum egestas ut lacus nec vestibulum. In hac habitasse platea dictumst. Sed ullamcorper dui non urna imperdiet imperdiet. Etiam ultrices ligula sit amet molestie dapibus. Duis feugiat lectus a finibus viverra. Ut condimentum eleifend sagittis. Sed fringilla sem eget orci feugiat aliquet et vel ex. Morbi id ultricies ex. Donec est ex, dictum nec elit vitae, consectetur semper lacus.

Suspendisse euismod eleifend interdum. Sed quis est et tortor imperdiet malesuada. Phasellus porttitor, orci eu pulvinar tristique, purus eros consequat tellus, ut dapibus leo ligula at felis. Vivamus gravida, felis scelerisque ultrices posuere, mauris mauris fermentum lectus, ac sagittis felis odio sed nulla. Praesent congue libero nisl, et malesuada massa ullamcorper nec. Sed sollicitudin ornare arcu, eget condimentum quam consequat pretium. Donec volutpat cursus dignissim. Vivamus eget blandit nibh. Vivamus cursus sed turpis ut consequat. Pellentesque elit magna, viverra quis magna ac, lacinia consequat urna. ";

#[derive(Default)]
struct TextRendering {
    pub i: usize,
}

impl AppState for TextRendering {
    type SpriteKey = String;

    fn generate_layout(
        &mut self,
        window_size: rust_ui::geometry::Vector<f32>,
        ui: &UiBuilder<Self>,
    ) -> Vec<rust_ui::render::renderer::RenderLayout<Self>> {
        let root = ui.div(
            "p-16 w-full h-full",
            &[
                ui.text("", Text::new(format!("{}", self.i), 12, COLOR_LIGHT)),
                ui.text_explicit("", Text::new(TEST_TEXT, 12, COLOR_LIGHT)),
            ],
        );

        vec![RenderLayout {
            tree: ui.tree(),
            root,
            desired_size: Size {
                width: AvailableSpace::Definite(window_size.x),
                height: AvailableSpace::Definite(window_size.y),
            },
            root_pos: Vector::zero(),
            anchor: Anchor::TopLeft,
            scissor: true,
        }]
    }
}

pub fn render_text(iters: usize) {
    let (mut glfw, mut window, events) = init_open_gl(1000, 800, true, false);

    let rect_shader = Shader::new_from_name(&ShaderName::Rect).unwrap();
    let text_shader = Shader::new_from_name(&ShaderName::Text).unwrap();
    let line_shader = Shader::new_from_name(&ShaderName::Line).unwrap();

    let rect_r = RectRenderer::new(rect_shader);
    let text_r = TextRenderer::new(
        text_shader,
        &PathBuf::from_str("assets/fonts/LiberationMono.ttf").unwrap(),
    )
    .unwrap();
    let line_r = LineRenderer::new(line_shader);
    let sprite_r = SpriteRenderer::new(Shader::empty(), SpriteAtlas::empty());

    let mut state = Renderer::new(rect_r, text_r, line_r, sprite_r, TextRendering::default());
    while !window.should_close() && state.app_state.i < iters {
        let _span = tracy_client::span!("Loop iteration");
        glfw.poll_events();
        state.pre_update();
        for (_, _) in glfw::flush_messages(&events) {}
        state.update();

        let projection = glm::ortho(0.0, state.width as f32, state.height as f32, 0.0, -1.0, 1.0);
        rect_shader.use_shader();
        rect_shader.set_uniform("projection", &projection);
        text_shader.use_shader();
        text_shader.set_uniform("projection", &projection);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        state.render();
        window.swap_buffers();

        state.app_state.i += 1;
    }
    unsafe {
        gl::Flush();
        gl::Finish();
    }
    glfw::make_context_current(None);
    // Segfaults due to bug in glfw with wayland
    std::mem::forget(window);
}
