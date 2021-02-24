#[macro_use]
extern crate lazy_static;
extern crate swgl;

use wasm_bindgen::prelude::*;

use swgl::camera2d::ratio_view::RatioView;
use swgl::graphics_2d::color::Color;
use swgl::graphics_2d::renderer::geometry_renderer::GeometryRenderer;
use swgl::graphics_2d::renderer::rectangle_renderer::RectangleRenderer;

use swgl::gl_wrapper::texture::texture_2d::Texture2D;
use swgl::gl_wrapper::texture::texture_config::{self, TextureConfiguration};
use swgl::gl_wrapper::vertex_array_object::PrimitiveType;
use swgl::global_tools::vector2::Vector2;
use swgl::graphics_2d::vertex_2d::predefined::color_vertex2d::ColorVertex2D;
use swgl::resources_loader;

use swgl::graphics_2d::vertex_2d::predefined::single_tex_vertex2d::SingleTexVertex2D;

use swgl::camera2d::interface::CameraType;

use swgl::gl_wrapper::basics::clear_canvas;

mod app_state;
mod gl_setup;

mod flocking;
use flocking::Flock;

// -----------------------------------------------------------------------------------------

const DISPLAY_SIZE: f32 = 1000.0;
const BORDER_THICK: f32 = 50.0;
const BORDER_COLOR: u32 = 0x222222ff;
const OUTLINE_COLOR: u32 = 0xffffffff;

// -----------------------------------------------------------------------------------------

#[wasm_bindgen]
pub struct AppState {
    context: swgl::AppContext,
    last_tick: f32,

    camera: RatioView,

    flock: Flock,
    batch_renderer: GeometryRenderer<ColorVertex2D>,
    rectangle_renderer: RectangleRenderer<SingleTexVertex2D>,

    cursor_texture: Texture2D,
    mouse_pos: Vector2<f32>,
}

#[wasm_bindgen]
impl AppState {
    #[wasm_bindgen(constructor)]
    pub async fn new(last_tick: f32, width: f32, height: f32) -> Self {
        // ----------------------------- init webgl ---------------------------

        console_error_panic_hook::set_once();
        let (_, context) =
            gl_setup::initialize_webgl_context("#canvas", Color::from_hex(0x222222ff))
                .expect("Cannot initialize WebGL");

        // ----------------------------- load resources -----------------------

        let texture_file_path = "static/fish.png";
        let loaded_resource = resources_loader::read_local_files(&[texture_file_path])
            .await
            .unwrap();
        let image =
            resources_loader::unwrap_image_content(&loaded_resource[texture_file_path]).unwrap();

        // ----------------------------- prepare objects ----------------------

        let camera = RatioView::new(Vector2::new(width, height), DISPLAY_SIZE);

        let cursor_texture = Texture2D::new_texture2d(
            &context,
            &image,
            TextureConfiguration::default(),
        )
        .unwrap();

        let batch_renderer = GeometryRenderer::init(&context, 400).unwrap();
        let rectangle_renderer = RectangleRenderer::init(&context, 100).unwrap();

        let flock = flocking::Flock::new(50, camera.scene_relative_size).unwrap();

        // ----------------------------- construct app ------------------------
        Self {
            context,
            last_tick,
            camera,
            flock,
            batch_renderer,
            rectangle_renderer,
            cursor_texture,
            mouse_pos: Vector2::zero(),
        }
    }

    pub fn update(&mut self, time: f32, width: f32, height: f32) -> Result<(), JsValue> {
        app_state::update_dynamic_data(time, height, width);
        self.camera.update_canvas_size(Vector2::new(width, height));

        let now = time;
        let dt = (now - self.last_tick) / 1000.0;
        self.last_tick = now;

        let mouse_pos = self
            .camera
            .map_pixel_coords_to_game_coords(&app_state::get_curr_state().mouse_pos);

        self.mouse_pos = mouse_pos;

        self.flock.update(
            dt,
            self.camera.scene_relative_size,
            BORDER_THICK,
            &mouse_pos,
        );

        Ok(())
    }

    pub fn render(&mut self) {
        clear_canvas(&self.context);

        self.flock.draw(
            &self.context,
            &mut self.rectangle_renderer,
            &self.camera,
            &self.cursor_texture,
        );
        self.batch_renderer
            .draw(
                &self.context,
                &border_vertices(),
                PrimitiveType::Triangles,
                &self.camera,
            )
            .unwrap();
        self.batch_renderer
            .draw(
                &self.context,
                &outline_vertices(),
                PrimitiveType::LineLoop,
                &self.camera,
            )
            .unwrap();
    }
}

// -----------------------------------------------------------------------------------------

// I know, it is long :)
fn border_vertices() -> [ColorVertex2D; 24] {
    let color = Color::from_hex(BORDER_COLOR);

    [
        // left border
        ColorVertex2D::new(Vector2::new(0.0, 0.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(BORDER_THICK, 0.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(0.0, DISPLAY_SIZE), color, 0.0),
        ColorVertex2D::new(Vector2::new(0.0, DISPLAY_SIZE), color, 0.0),
        ColorVertex2D::new(Vector2::new(BORDER_THICK, 1000.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(BORDER_THICK, 0.0), color, 0.0),
        // right border
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE - BORDER_THICK, 0.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE, 0.0), color, 0.0),
        ColorVertex2D::new(
            Vector2::new(DISPLAY_SIZE - BORDER_THICK, DISPLAY_SIZE),
            color,
            0.0,
        ),
        ColorVertex2D::new(
            Vector2::new(DISPLAY_SIZE - BORDER_THICK, DISPLAY_SIZE),
            color,
            0.0,
        ),
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE, DISPLAY_SIZE), color, 0.0),
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE, 0.0), color, 0.0),
        // top border
        ColorVertex2D::new(Vector2::new(0.0, 0.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(0.0, BORDER_THICK), color, 0.0),
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE, 0.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE, 0.0), color, 0.0),
        ColorVertex2D::new(Vector2::new(1000.0, BORDER_THICK), color, 0.0),
        ColorVertex2D::new(Vector2::new(0.0, BORDER_THICK), color, 0.0),
        // bottom border
        ColorVertex2D::new(Vector2::new(0.0, DISPLAY_SIZE - BORDER_THICK), color, 0.0),
        ColorVertex2D::new(Vector2::new(0.0, DISPLAY_SIZE), color, 0.0),
        ColorVertex2D::new(
            Vector2::new(DISPLAY_SIZE, DISPLAY_SIZE - BORDER_THICK),
            color,
            0.0,
        ),
        ColorVertex2D::new(
            Vector2::new(DISPLAY_SIZE, DISPLAY_SIZE - BORDER_THICK),
            color,
            0.0,
        ),
        ColorVertex2D::new(Vector2::new(DISPLAY_SIZE, DISPLAY_SIZE), color, 0.0),
        ColorVertex2D::new(Vector2::new(0.0, DISPLAY_SIZE), color, 0.0),
    ]
}

fn outline_vertices() -> [ColorVertex2D; 4] {
    let color = Color::from_hex(OUTLINE_COLOR);
    [
        ColorVertex2D::new(Vector2::new(BORDER_THICK, BORDER_THICK), color, 0.0),
        ColorVertex2D::new(
            Vector2::new(DISPLAY_SIZE - BORDER_THICK, BORDER_THICK),
            color,
            0.0,
        ),
        ColorVertex2D::new(
            Vector2::new(DISPLAY_SIZE - BORDER_THICK, DISPLAY_SIZE - BORDER_THICK),
            color,
            0.0,
        ),
        ColorVertex2D::new(
            Vector2::new(0.0 + BORDER_THICK, DISPLAY_SIZE - BORDER_THICK),
            color,
            0.0,
        ),
    ]
}
