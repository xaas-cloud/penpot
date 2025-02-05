use skia_safe as skia;
use std::collections::HashMap;
use uuid::Uuid;

use crate::view::Viewbox;

mod blend;
mod cache;
mod debug;
mod fills;
mod gpu_state;
mod images;
mod options;
mod shadows;
mod strokes;

use crate::shapes::{Kind, Shape};
use cache::CachedSurfaceImage;
use gpu_state::GpuState;
use options::RenderOptions;

pub use blend::BlendMode;
pub use images::*;

const DEFAULT_FONT_BYTES: &[u8] =
    include_bytes!("../../frontend/resources/fonts/RobotoMono-Regular.ttf");
extern "C" {
    fn emscripten_run_script_int(script: *const i8) -> i32;
}

fn get_time() -> i32 {
    let script = std::ffi::CString::new("performance.now()").unwrap();
    unsafe { emscripten_run_script_int(script.as_ptr()) }
}

pub(crate) struct RenderState {
    gpu_state: GpuState,
    pub options: RenderOptions,

    // TODO: Probably we're going to need
    // a surface stack like the one used
    // by SVG: https://www.w3.org/TR/SVG2/render.html
    pub final_surface: skia::Surface,
    pub drawing_surface: skia::Surface,
    pub shadow_surface: skia::Surface,
    pub debug_surface: skia::Surface,
    pub font_provider: skia::textlayout::TypefaceFontProvider,
    pub cached_surface_image: Option<CachedSurfaceImage>,
    pub viewbox: Viewbox,
    pub images: ImageStore,
    pub background_color: skia::Color,
    pub render_time: i32,
    pub render_frame_id: Option<i32>,
    pub is_running: bool,
    pub stack: Vec<(Uuid, bool)>,
}

impl RenderState {
    pub fn new(width: i32, height: i32) -> RenderState {
        // This needs to be done once per WebGL context.
        let mut gpu_state = GpuState::new();
        let mut final_surface = gpu_state.create_target_surface(width, height);
        let shadow_surface = final_surface
            .new_surface_with_dimensions((width, height))
            .unwrap();
        let drawing_surface = final_surface
            .new_surface_with_dimensions((width, height))
            .unwrap();
        let debug_surface = final_surface
            .new_surface_with_dimensions((width, height))
            .unwrap();

        let mut font_provider = skia::textlayout::TypefaceFontProvider::new();
        let default_font = skia::FontMgr::default()
            .new_from_data(DEFAULT_FONT_BYTES, None)
            .expect("Failed to load font");
        font_provider.register_typeface(default_font, "robotomono-regular");

        RenderState {
            gpu_state,
            final_surface,
            shadow_surface,
            drawing_surface,
            debug_surface,
            cached_surface_image: None,
            font_provider,
            options: RenderOptions::default(),
            viewbox: Viewbox::new(width as f32, height as f32),
            images: ImageStore::new(),
            background_color: skia::Color::TRANSPARENT,
            render_time: get_time(),
            render_frame_id: None,
            is_running: false,
            stack: vec![],
        }
    }

    pub fn add_font(&mut self, family_name: String, font_data: &[u8]) -> Result<(), String> {
        let typeface = skia::FontMgr::default()
            .new_from_data(font_data, None)
            .expect("Failed to add font");
        self.font_provider
            .register_typeface(typeface, family_name.as_ref());
        Ok(())
    }

    pub fn add_image(&mut self, id: Uuid, image_data: &[u8]) -> Result<(), String> {
        self.images.add(id, image_data)
    }

    pub fn has_image(&mut self, id: &Uuid) -> bool {
        self.images.contains(id)
    }

    pub fn set_debug_flags(&mut self, debug: u32) {
        self.options.debug_flags = debug;
    }

    pub fn set_dpr(&mut self, dpr: f32) {
        if Some(dpr) != self.options.dpr {
            self.options.dpr = Some(dpr);
            self.resize(
                self.viewbox.width.floor() as i32,
                self.viewbox.height.floor() as i32,
            );
        }
    }

    pub fn set_background_color(&mut self, color: skia::Color) {
        self.background_color = color;
        let _ = self.render_all_from_cache();
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        let dpr_width = (width as f32 * self.options.dpr()).floor() as i32;
        let dpr_height = (height as f32 * self.options.dpr()).floor() as i32;

        let surface = self.gpu_state.create_target_surface(dpr_width, dpr_height);
        self.final_surface = surface;
        self.shadow_surface = self
            .final_surface
            .new_surface_with_dimensions((dpr_width, dpr_height))
            .unwrap();
        self.drawing_surface = self
            .final_surface
            .new_surface_with_dimensions((dpr_width, dpr_height))
            .unwrap();
        self.debug_surface = self
            .final_surface
            .new_surface_with_dimensions((dpr_width, dpr_height))
            .unwrap();

        self.viewbox.set_wh(width as f32, height as f32);
    }

    pub fn flush(&mut self) {
        self.gpu_state
            .context
            .flush_and_submit_surface(&mut self.final_surface, None);
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.drawing_surface.canvas().translate((dx, dy));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.drawing_surface.canvas().scale((sx, sy));
    }

    pub fn reset_canvas(&mut self) {
        // println!("reset canvas");
        self.drawing_surface
            .canvas()
            .clear(self.background_color)
            .reset_matrix();
        self.shadow_surface
            .canvas()
            .clear(self.background_color)
            .reset_matrix();
        self.final_surface
            .canvas()
            .clear(self.background_color)
            .reset_matrix();
        self.debug_surface
            .canvas()
            .clear(skia::Color::TRANSPARENT)
            .reset_matrix();
    }

    pub fn apply_drawing_to_final_canvas(&mut self) {
        self.gpu_state
            .context
            .flush_and_submit_surface(&mut self.drawing_surface, None);

        self.drawing_surface.draw(
            &mut self.final_surface.canvas(),
            (0.0, 0.0),
            skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
            Some(&skia::Paint::default()),
        );

        self.shadow_surface.canvas().clear(skia::Color::TRANSPARENT);

        self.drawing_surface
            .canvas()
            .clear(skia::Color::TRANSPARENT);
    }

    pub fn render_shape(&mut self, shape: &mut Shape, clip: bool) {
        let transform = shape.transform.to_skia_matrix();

        // Check transform-matrix code from common/src/app/common/geom/shapes/transforms.cljc
        let center = shape.bounds().center();
        let mut matrix = skia::Matrix::new_identity();
        matrix.pre_translate(center);
        matrix.pre_concat(&transform);
        matrix.pre_translate(-center);

        self.drawing_surface.canvas().concat(&matrix);

        match &shape.kind {
            Kind::SVGRaw(sr) => {
                if let Some(svg) = shape.svg.as_ref() {
                    svg.render(self.drawing_surface.canvas())
                } else {
                    let font_manager = skia::FontMgr::from(self.font_provider.clone());
                    let dom_result = skia::svg::Dom::from_str(sr.content.to_string(), font_manager);
                    match dom_result {
                        Ok(dom) => {
                            dom.render(self.drawing_surface.canvas());
                            shape.set_svg(dom);
                        }
                        Err(e) => {
                            eprintln!("Error parsing SVG. Error: {}", e);
                        }
                    }
                }
            }
            _ => {
                for fill in shape.fills().rev() {
                    fills::render(self, shape, fill);
                }

                for stroke in shape.strokes().rev() {
                    strokes::render(self, shape, stroke);
                }
            }
        };

        if clip {
            self.drawing_surface
                .canvas()
                .clip_rect(shape.bounds(), skia::ClipOp::Intersect, true);
        }

        for shadow in shape.drop_shadows().rev().filter(|s| !s.hidden()) {
            shadows::render_drop_shadow(self, shadow, self.viewbox.zoom * self.options.dpr());
        }

        self.apply_drawing_to_final_canvas();
    }

    pub fn render_all(
        &mut self,
        tree: &mut HashMap<Uuid, Shape>,
        generate_cached_surface_image: bool,
    ) {
        self.render_time = get_time();
        let is_complete = self.render_shape_tree(tree);
        if generate_cached_surface_image || self.cached_surface_image.is_none() {
            self.cached_surface_image = Some(CachedSurfaceImage {
                image: self.final_surface.image_snapshot(),
                viewbox: self.viewbox,
                has_all_shapes: is_complete,
            });
        }

        if self.options.is_debug_visible() {
            self.render_debug();
        }

        debug::render_wasm_label(self);

        // self.flush();
    }

    pub fn render_all_from_cache(&mut self) -> Result<(), String> {
        self.reset_canvas();

        let cached = self
            .cached_surface_image
            .as_ref()
            .ok_or("Uninitialized cached surface image")?;

        let image = &cached.image;
        let paint = skia::Paint::default();
        self.final_surface.canvas().save();
        self.drawing_surface.canvas().save();

        let navigate_zoom = self.viewbox.zoom / cached.viewbox.zoom;
        let navigate_x = cached.viewbox.zoom * (self.viewbox.pan_x - cached.viewbox.pan_x);
        let navigate_y = cached.viewbox.zoom * (self.viewbox.pan_y - cached.viewbox.pan_y);

        self.final_surface
            .canvas()
            .scale((navigate_zoom, navigate_zoom));
        self.final_surface.canvas().translate((
            navigate_x * self.options.dpr(),
            navigate_y * self.options.dpr(),
        ));
        self.final_surface
            .canvas()
            .draw_image(image.clone(), (0, 0), Some(&paint));

        self.final_surface.canvas().restore();
        self.drawing_surface.canvas().restore();

        self.flush();

        Ok(())
    }

    fn render_debug(&mut self) {
        debug::render(self);
    }

    pub fn start_rendering(&mut self, root_id: Uuid) {
        self.stack = vec![(root_id, false)];
        self.render_time = get_time();
        self.is_running = true;
    }

    pub fn render_shape_tree(&mut self, tree: &HashMap<Uuid, Shape>) -> bool {
        // TODO
        let is_complete = false;
        if !self.is_running {
            return false;
        }

        let mut duration = 0;
        println!("---->render_shape_tree stack {:?}", self.stack);
        while let Some((node_id, visited_children)) = self.stack.pop() {
            // println!("Duration {:?}", duration);
            // println!("---->stack {:?}", self.stack);

            if let Some(element) = tree.get(&node_id) {
                if !visited_children {
                    // let mut is_complete = self.viewbox.area.contains(element.bounds());

                    if !node_id.is_nil() {
                        if !element.bounds().intersects(self.viewbox.area) || element.hidden() {
                            debug::render_debug_element(self, element, false);
                            continue;
                        } else {
                            debug::render_debug_element(self, element, true);
                        }
                    }

                    let mut paint = skia::Paint::default();
                    paint.set_blend_mode(element.blend_mode().into());
                    paint.set_alpha_f(element.opacity());

                    if let Some(image_filter) =
                        element.image_filter(self.viewbox.zoom * self.options.dpr())
                    {
                        paint.set_image_filter(image_filter);
                    }

                    let layer_rec = skia::canvas::SaveLayerRec::default().paint(&paint);
                    // println!("{node_id} self.final_surface.canvas().save_layer(&layer_rec)");
                    self.final_surface.canvas().save_layer(&layer_rec);

                    // println!("{node_id} self.drawing_surface.canvas().save()");
                    self.drawing_surface.canvas().save();
                    if !node_id.is_nil() {
                        // println!("{node_id} self.render_shape(&mut element.clone(), element.clip())");
                        self.render_shape(&mut element.clone(), element.clip());
                    } else {
                        // println!("{node_id} self.apply_drawing_to_final_canvas()");
                        self.apply_drawing_to_final_canvas();
                    }
                    // println!("{node_id} self.drawing_surface.canvas().restore()");
                    self.drawing_surface.canvas().restore();

                    // Marcar el nodo como "visitado" antes de procesar los hijos
                    self.stack.push((node_id, true));
                    // println!("---->stack 2 {:?}", self.stack);

                    // Agregar los hijos a la pila
                    if element.is_recursive() {
                        for child_id in element.children_ids().iter().rev() {
                            self.stack.push((*child_id, false));
                            // println!("---->stack 3 {:?}", self.stack);
                        }
                    }
                } else {
                    // println!("{node_id} self.final_surface.canvas().restore()");
                    self.final_surface.canvas().restore();
                }
            } else {
                eprintln!("Error: Element with root_id {node_id} not found in the tree.");
                // return false;
            }
            // duration = get_time() - self.render_time;
            if duration > 16 {
                return false;
            }
        }

        // Si terminamos de procesar todos los nodos, marcamos la renderización como completa
        self.is_running = false;
        return is_complete;
    }
}
