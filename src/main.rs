use std::time::Duration;

use binpack2d::{bin_new, BinType, Dimension};
use gl_rs as gl;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    GlProfile,
};

use layers::{
    prelude::{timing::TimingFunction, *},
    skia::{self, Color4f, ColorType},
    types::Size,
};
use rand::Rng;

pub fn draw(canvas: &mut skia::Canvas, width: f32, _height: f32) {
    let mut text_style = skia::textlayout::TextStyle::new();
    text_style.set_font_size(60.0);
    let foreground_paint = skia::Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
    text_style.set_foreground_color(&foreground_paint);
    text_style.set_font_families(&["Inter"]);

    let font_mgr = skia::FontMgr::new();
    let type_face_font_provider = skia::textlayout::TypefaceFontProvider::new();
    let mut font_collection = skia::textlayout::FontCollection::new();
    font_collection.set_asset_font_manager(Some(type_face_font_provider.clone().into()));
    font_collection.set_dynamic_font_manager(font_mgr.clone());

    let mut paragraph_style = skia::textlayout::ParagraphStyle::new();

    paragraph_style.set_text_style(&text_style);
    paragraph_style.set_max_lines(2);
    paragraph_style.set_text_align(skia::textlayout::TextAlign::Center);
    paragraph_style.set_text_direction(skia::textlayout::TextDirection::LTR);
    paragraph_style.set_ellipsis("…");
    let mut paragraph = skia::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection)
        .add_text("Hello World! 👋🌍")
        .build();

    paragraph.layout(width);
    paragraph.paint(canvas, (0.0, 0.0));
}

fn expose(windows: &mut Vec<Layer>, space_width: f32, space_height: f32) {
    let num_windows = windows.len();
    let num_cols = (num_windows as f32).sqrt().ceil() as usize;
    let num_rows = num_cols;

    let cell_width = space_width / num_cols as f32;
    let cell_height = space_height / num_rows as f32;

    let mut cell_assigned = vec![false; num_rows * num_cols];

    for window in windows.iter_mut() {
        let size = window.size();
        let (scale_x, scale_y, window_width, window_height) = match (size.width, size.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => {
                let scale_x = cell_width / width;
                let scale_y = cell_height / height;
                (scale_x, scale_y, width, height)
            }
            _ => (1.0, 1.0, 0.0, 0.0),
        };
        let scale = scale_x.min(scale_y);

        let mut min_distance = f32::MAX;
        let mut closest_cell = (0, 0);

        let position = window.position();
        for row in 0..num_rows {
            for col in 0..num_cols {
                if cell_assigned[row * num_cols + col] {
                    continue;
                }

                let cell_center_x = col as f32 * cell_width + cell_width * 0.5;
                let cell_center_y = row as f32 * cell_height + cell_height * 0.5;

                let distance = ((position.x - cell_center_x).powi(2)
                    + (position.y - cell_center_y).powi(2))
                .sqrt();

                if distance < min_distance {
                    min_distance = distance;
                    closest_cell = (row, col);
                }
            }
        }

        let (row, col) = closest_cell;
        cell_assigned[row * num_cols + col] = true;

        let x = col as f32 * cell_width + cell_width * 0.5 - window_width * 0.5 * scale;
        let y = row as f32 * cell_height + cell_height * 0.5 - window_height * 0.5 * scale;

        window.set_scale((scale, scale), Some(Transition::default()));
        window.set_position((x, y), Some(Transition::default()));
    }
}

fn expose_step(windows: &mut Vec<Layer>, space_width: f32, space_height: f32, step: i32) {
    let step = step as f32 / 100.0;
    let num_windows = windows.len();
    let num_cols = (num_windows as f32).sqrt().ceil() as usize;
    let num_rows = num_cols;

    let cell_width = space_width / num_cols as f32;
    let cell_height = space_height / num_rows as f32;

    let mut cell_assigned = vec![false; num_rows * num_cols];

    for window in windows.iter_mut() {
        let size = window.size();
        let (scale_x, scale_y, window_width, window_height) = match (size.width, size.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => {
                let scale_x = cell_width / width;
                let scale_y = cell_height / height;
                (scale_x, scale_y, width, height)
            }
            _ => (1.0, 1.0, 0.0, 0.0),
        };

        let mut min_distance = f32::MAX;
        let mut closest_cell = (0, 0);

        let position = window.position();
        for row in 0..num_rows {
            for col in 0..num_cols {
                if cell_assigned[row * num_cols + col] {
                    continue;
                }

                let cell_center_x = col as f32 * cell_width + cell_width * 0.5;
                let cell_center_y = row as f32 * cell_height + cell_height * 0.5;

                let distance = ((position.x - cell_center_x).powi(2)
                    + (position.y - cell_center_y).powi(2))
                .sqrt();

                if distance < min_distance {
                    min_distance = distance;
                    closest_cell = (row, col);
                }
            }
        }

        let (row, col) = closest_cell;
        cell_assigned[row * num_cols + col] = true;

        let scale = window.scale().x;
        let to_scale = scale_x.min(scale_y);
        let scale = scale.interpolate(&to_scale, step);
        let x = col as f32 * cell_width + cell_width * 0.5 - window_width * 0.5 * scale;
        let y = row as f32 * cell_height + cell_height * 0.5 - window_height * 0.5 * scale;
        let x = position.x.interpolate(&x, step);
        let y = position.y.interpolate(&y, step);

        window.set_scale((scale, scale), Some(Transition::default()));
        window.set_position((x, y), Some(Transition::default()));
    }
}

fn normalize(windows: &mut Vec<Layer>, space_width: f32, space_height: f32) {
    let num_windows = windows.len();
    let num_cols = (num_windows as f32).sqrt().ceil() as usize;
    let num_rows = num_cols;

    let window_width = space_width / num_cols as f32;
    let window_height = space_height / num_rows as f32;

    for (index, window) in windows.iter_mut().enumerate() {
        let row = index / num_cols;
        let col = index % num_cols;

        let x = col as f32 * window_width;
        let y = row as f32 * window_height;
        let size = window.size();

        window.set_scale((1.0, 1.0), Some(Transition::default()));
        window.set_position(
            (50.0 * index as f32, 50.0 * index as f32),
            Some(Transition::default()),
        );
    }
}

pub struct Bin {
    width: f32,
    height: f32,
    windows: Vec<Layer>,
}

impl Bin {
    pub fn new(width: f32, height: f32) -> Self {
        Bin {
            width,
            height,
            windows: Vec::new(),
        }
    }

    pub fn add(&mut self, window: Layer) -> bool {
        let size = window.size();
        let (window_width, window_height) = match (size.width, size.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => (width, height),
            _ => (0.0, 0.0),
        };
        if self.width >= window_width && self.height >= window_height {
            self.windows.push(window);
            true
        } else {
            false
        }
    }

    pub fn can_fit(&self, window: &Layer) -> bool {
        let size = window.size();
        let (window_width, window_height) = match (size.width, size.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => (width, height),
            _ => (0.0, 0.0),
        };
        self.width >= window_width && self.height >= window_height
    }

    pub fn empty_space_after_insertion(&self, window: &Layer) -> f32 {
        let size = window.size();
        let (window_width, window_height) = match (size.width, size.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => (width, height),
            _ => (0.0, 0.0),
        };
        (self.width - window_width) * (self.height - window_height)
    }
}
pub fn bin_pack(windows: &mut Vec<Layer>, bin_width: f32, bin_height: f32) {
    // Sort windows in decreasing order of size
    windows.sort_by(|a, b| {
        let size_a = a.size();
        let size_b = b.size();
        let area_a = match (size_a.width, size_a.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => width * height,
            _ => 0.0,
        };
        let area_b = match (size_b.width, size_b.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => width * height,
            _ => 0.0,
        };
        area_b.partial_cmp(&area_a).unwrap()
    });

    let mut bins: Vec<Bin> = Vec::new();

    for window in windows.iter() {
        let mut best_fit = None;
        let mut min_empty_space = f32::MAX;

        for (i, bin) in bins.iter_mut().enumerate() {
            if bin.can_fit(window) {
                let empty_space = bin.empty_space_after_insertion(window);
                if empty_space < min_empty_space {
                    best_fit = Some(i);
                    min_empty_space = empty_space;
                }
            }
        }

        if let Some(i) = best_fit {
            bins[i].add(window.clone());
        } else {
            let mut bin = Bin::new(bin_width, bin_height);
            bin.add(window.clone());
            bins.push(bin);
        }
    }
   
    let mut max_height_in_row = 0.0;

   
    let total_window_area: f32 = {
        windows
            .iter()
            .map(|window| {
                let size = window.size();
                match (size.width, size.height) {
                    (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => {
                        width * height
                    }
                    _ => 0.0,
                }
            })
            .sum()
    };

    let total_bin_area = bin_width * bin_height * bins.len() as f32;
    let scale_factor = (total_bin_area / total_window_area).sqrt();

    for bin in &mut bins {
        println!("bin: {}x{}", bin.width, bin.height);
        let mut x = 0.0;
        let mut y = 0.0;
        for window in &mut bin.windows {
            let size = window.size();
            let (window_width, window_height) = match (size.width, size.height) {
                (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => {
                    (width, height)
                }
                _ => (0.0, 0.0),
            };
            window.set_position((x, y), Some(Transition::default()));
            window.set_scale((scale_factor, scale_factor), Some(Transition::default()));
            let window_height = window_height * scale_factor;
            let window_width = window_width * scale_factor;

            if window_height > max_height_in_row {
                max_height_in_row = window_height;
            }
            if x + window_width * 2.0 > bin.width {
                x = 0.0;
                y += max_height_in_row;
                max_height_in_row = 0.0;
            } else {
                x += window_width;
            }
        }
    }
}

pub fn bin_pack2(windows: &mut Vec<Layer>, bin_width: f32, bin_height: f32) {
    let total_window_area: f32 = {
        windows
            .iter()
            .map(|window| {
                let size = window.size();
                match (size.width, size.height) {
                    (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => {
                        width * height
                    }
                    _ => 0.0,
                }
            })
            .sum()
    };

    let total_bin_area = bin_width * bin_height;
    let mut scale_factor = (total_bin_area / total_window_area).sqrt();
    let mut items_to_place = Vec::new();
    for window in windows.iter() {
        let size = window.size();
        let (window_width, window_height) = match (size.width, size.height) {
            (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => (width, height),
            _ => (0.0, 0.0),
        };
        let id = window.id().unwrap();
        let id:usize = id.0.into();
        let dimension = Dimension::with_id(id as isize, (window_width * scale_factor) as i32, (window_height * scale_factor) as i32, 20);
        items_to_place.push(dimension);
    }

    let mut bin = bin_new(BinType::MaxRects, bin_width as i32, bin_height as i32);
    let (mut inserted, mut rejected) = bin.insert_list(&items_to_place);
    let mut tries = 0;
    while (!rejected.is_empty() || inserted.len() != windows.len()) && tries < 40 {
        scale_factor *= 0.99;
        scale_factor = scale_factor.max(0.1);
        let mut items_to_place = Vec::new();
        for window in windows.iter() {
            let size = window.size();
            let (window_width, window_height) = match (size.width, size.height) {
                (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => (width, height),
                _ => (0.0, 0.0),
            };
            let id = window.id().unwrap();
            let id:usize = id.0.into();
            let dimension = Dimension::with_id(id as isize, (window_width * scale_factor) as i32, (window_height * scale_factor) as i32, 20);
            items_to_place.push(dimension);
            // println!("{} {} {}", id, (window_width*scale_factor) as i32, (window_height*scale_factor) as i32);
        }
        bin.clear();
        (inserted, rejected) = bin.insert_list(&items_to_place);
        // println!("items placed: {}/{} scale {}", inserted.len(), windows.len(), scale_factor);
        tries += 1;
    }
    // println!("rejected: {}", rejected.len());
    // println!("inserted: {}/{}", inserted.len(), windows.len());
    for (index, window) in windows.iter_mut().enumerate() {
        let id = window.id().unwrap();
        let id:usize = id.0.into();
        if let Some(rect) = bin.find_by_id(id as isize) {
            let x = rect.x();
            let y = rect.y();
            let width = rect.width();
            let height = rect.height();
            let size = window.size();
            let (window_width, window_height) = match (size.width, size.height) {
                (taffy::Dimension::Points(width), taffy::Dimension::Points(height)) => (width, height),
                _ => (0.0, 0.0),
            };
            let scale_x = width as f32 / window_width;
            let scale_y = height as f32 / window_height;
            let scale = scale_x.min(scale_y).min(1.0);

            // println!("{}, {}, {}", scale, x, y);
            window.set_position((x as f32, y as f32), Some(Transition::default()));
            window.set_scale((scale, scale), Some(Transition::default()));
        } else {
            println!("{} not found", id);
        }
    }

}
fn main() {
    type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

    use winit::dpi::LogicalSize;

    const NUM_WINDOWS: u8 = 10;

    const SPACE_WIDTH: i32 = 1000;
    const SPACE_HEIGHT: i32 = 1000;

    let size: LogicalSize<i32> = LogicalSize::new(SPACE_WIDTH, SPACE_HEIGHT);

    let events_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Renderer".to_string());

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true);
    let windowed_context = cb.build_windowed(window, &events_loop).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    gl::load_with(|s| windowed_context.get_proc_address(s));

    let pixel_format = windowed_context.get_pixel_format();

    let window_size = windowed_context.window().inner_size();
    let sample_count: usize = pixel_format
        .multisampling
        .map(|s| s.try_into().unwrap())
        .unwrap_or(0);
    let pixel_format: usize = pixel_format.stencil_bits.try_into().unwrap();

    let mut skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
        window_size.width as i32,
        window_size.height as i32,
        sample_count,
        pixel_format,
        ColorType::RGBA8888,
        layers::skia::gpu::SurfaceOrigin::BottomLeft,
        0_u32,
    );

    struct Env {
        windowed_context: WindowedContext,
    }
    let env = Env { windowed_context };
    let engine = LayersEngine::new(SPACE_WIDTH as f32 * 2.0, SPACE_HEIGHT as f32 * 2.0);
    let root_layer = engine.new_layer();

    root_layer.set_size(layers::types::Size::points(2000.0, 2000.0), None);
    root_layer.set_background_color(
        PaintColor::Solid {
            color: Color::new_rgba255(180, 180, 180, 255),
        },
        None,
    );
    root_layer.set_border_corner_radius(10.0, None);
    root_layer.set_layout_style(taffy::Style {
        // display: taffy::Display::Flex,
        // align_content: Some(taffy::AlignContent::Center),
        // align_items: Some(taffy::AlignItems::Center),
        // justify_content: Some(taffy::JustifyContent::Center),
        ..Default::default()
    });
    engine.scene_add_layer(root_layer.clone());

    let mut windows = Vec::new();

    let mut rng = rand::thread_rng();
    for i in 0..NUM_WINDOWS {
        let window = engine.new_layer();
        let width = rng.gen_range(200.0..1000.0);
        let height = rng.gen_range(300.0..1000.0);
        window.set_size(layers::types::Size::points(width, height), None);
        let r = rng.gen_range(0..255);
        let g = rng.gen_range(0..255);
        let b = rng.gen_range(0..255);

        window.set_background_color(
            PaintColor::Solid {
                color: Color::new_rgba255(r, g, b, 255),
            },
            None,
        );
        window.set_border_width(1.0, None);
        window.set_border_color(
            PaintColor::Solid {
                color: Color::new_rgba255(0, 0, 0, 255),
            },
            None,
        );
        window.set_layout_style(taffy::Style {
            position: taffy::Position::Absolute,
            ..Default::default()
        });
        let x = rng.gen_range(0.0..(SPACE_WIDTH as f32 * 2.0));
        let y = rng.gen_range(0.0..SPACE_HEIGHT as f32 * 2.0);

        window.set_position((x, y), None);
        window.set_border_corner_radius(BorderRadius::new_single(20.0), None);
        engine.scene_add_layer(window.clone());

        windows.push(window);
    }

    let instant = std::time::Instant::now();
    let mut update_frame = 0;
    let mut draw_frame = -1;
    let last_instant = instant;
    let mut step = 0;
    events_loop.run(move |event, _, control_flow| {
        let now = std::time::Instant::now();
        let dt = (now - last_instant).as_secs_f32();
        let next = now.checked_add(Duration::new(0, 2 * 1000000)).unwrap();
        *control_flow = ControlFlow::WaitUntil(next);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    env.windowed_context.resize(physical_size);

                    let size = env.windowed_context.window().inner_size();
                    skia_renderer = layers::renderer::skia_fbo::SkiaFboRenderer::create(
                        size.width as i32,
                        size.height as i32,
                        sample_count,
                        pixel_format,
                        ColorType::RGBA8888,
                        layers::skia::gpu::SurfaceOrigin::BottomLeft,
                        0_u32,
                    );
                    let _transition = root_layer
                        .set_size(Size::points(size.width as f32, size.height as f32), None);
                    env.windowed_context.window().request_redraw();
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => {
                    #[allow(clippy::single_match)]
                    match input.virtual_keycode {
                        Some(keycode) => match keycode {
                            winit::event::VirtualKeyCode::Space => {
                                if input.state == winit::event::ElementState::Released {
                                    let dt = 0.016;
                                    let needs_redraw = engine.update(dt);
                                    if needs_redraw {
                                        env.windowed_context.window().request_redraw();
                                        // draw_frame = -1;
                                    }
                                }
                            }
                            winit::event::VirtualKeyCode::A => {
                                bin_pack2(&mut windows, 2000.0, 2000.0);
                            }

                            winit::event::VirtualKeyCode::S => {
                                normalize(&mut windows, 2000.0, 2000.0);
                            }
                            winit::event::VirtualKeyCode::Return => {
                                step += 2;
                                expose_step(&mut windows, 2000.0, 2000.0, step);
                            }

                            winit::event::VirtualKeyCode::Escape => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => (),
                        },
                        None => (),
                    }
                }
                WindowEvent::CursorMoved { position: _, .. } => {
                    // _mouse_x = position.x;
                    // _mouse_y = position.y;
                }

                WindowEvent::MouseInput { state: _, .. } => {}
                _ => (),
            },
            Event::MainEventsCleared => {
                let now = instant.elapsed().as_secs_f64();
                let frame_number = (now / 0.016).floor() as i32;
                if update_frame != frame_number {
                    update_frame = frame_number;
                    let dt = 0.016;
                    let needs_redraw = engine.update(dt);
                    if needs_redraw {
                        env.windowed_context.window().request_redraw();
                        // draw_frame = -1;
                    }
                }
            }
            Event::RedrawRequested(_) => {
                if draw_frame != update_frame {
                    if let Some(root) = engine.scene_root() {
                        let skia_renderer = skia_renderer.get_mut();
                        let damage = engine.damage();
                        let damage_rect =
                            skia::Rect::from_xywh(damage.x, damage.y, damage.width, damage.height);

                        skia_renderer.draw_scene(engine.scene(), root, None);

                        let mut surface = skia_renderer.surface();
                        let canvas = surface.canvas();

                        // draw expose rects
                        let mut paint = skia::Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
                        paint.set_stroke(true);
                        paint.set_stroke_width(2.0);

                        let num_cols = (NUM_WINDOWS as f32).sqrt().ceil() as usize;
                        let num_rows = num_cols;

                        let window_width = SPACE_WIDTH as f32 * 2.0 / num_cols as f32;
                        let window_height = SPACE_HEIGHT as f32 * 2.0 / num_rows as f32;
                        for index in 0..NUM_WINDOWS {
                            let row = index as usize / num_cols;
                            let col = index as usize % num_cols;

                            let x = col as f32 * window_width;
                            let y = row as f32 * window_height;
                            let rect = skia::Rect::from_xywh(x, y, window_width, window_height);
                            // canvas.draw_rect(rect, &paint);
                        }

                        // draw damage
                        // let mut paint = skia::Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None);
                        // paint.set_stroke(true);
                        // paint.set_stroke_width(10.0);
                        // canvas.draw_rect(damage_rect, &paint);

                        surface.flush_and_submit();
                    }
                    engine.clear_damage();
                    // this will be blocking until the GPU is done with the frame
                    env.windowed_context.swap_buffers().unwrap();
                    draw_frame = update_frame;
                } else {
                    // println!("skipping draw");
                }
            }
            _ => {}
        }
        // });
    });
}
