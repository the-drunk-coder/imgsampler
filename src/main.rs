mod line_parser;
mod parameter;

use nannou::image::{open, DynamicImage, GenericImageView, Pixel};
use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

use rand::Rng;
use std::collections::HashMap;

use line_parser::ParserResult;
use parameter::*;

enum ImgParams {
    Position(Box<dyn Parameter>, Box<dyn Parameter>),
    Size(Box<dyn Parameter>, Box<dyn Parameter>),
    Crop(
        Box<dyn Parameter>,
        Box<dyn Parameter>,
        Box<dyn Parameter>,
        Box<dyn Parameter>,
    ),
    Blur(Box<dyn Parameter>),
    Opacity(Box<dyn Parameter>),
    Brighten(Box<dyn Parameter>),
    HueRot(Box<dyn Parameter>),
    Contrast(Box<dyn Parameter>),
    Scatter(Box<dyn Parameter>),
    Brownian(Box<dyn Parameter>),
}

enum InterpretedToken {
    String(String),
    Par(Box<dyn Parameter>),
}

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    textures: Vec<(wgpu::Texture, f32, f32, f32, f32)>,
    draw_window_id: WindowId,
    code_window_id: WindowId,
    text: String,
    parameters: HashMap<String, Vec<ImgParams>>,
    positions: HashMap<String, ImgParams>,
    sizes: HashMap<String, ImgParams>,
    images: HashMap<String, DynamicImage>,
    asset_path: std::path::PathBuf,
    egui: Egui,
}
fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);

    if !matches!(
        event,
        nannou::winit::event::WindowEvent::KeyboardInput { .. }
    ) {
        return;
    }

    let mut parameters = HashMap::<String, Vec<ImgParams>>::new();
    let mut positions = HashMap::<String, ImgParams>::new();
    let mut sizes = HashMap::<String, ImgParams>::new();
    let mut images = HashMap::<String, DynamicImage>::new();

    let lines = model.text.split('\n');
    for line in lines {
        if matches!(line.chars().next(), Some('#')) {
            continue;
        }

        // parse line
        if let Ok((_, mut token_vec)) = line_parser::parse_line(line) {
            // "interpret" tokens
            let mut itokens: Vec<InterpretedToken> = Vec::new();
            for token in token_vec.drain(..) {
                match token {
                    ParserResult::String(val) => {
                        itokens.push(InterpretedToken::String(val));
                    }
                    ParserResult::Scalar(val) => {
                        itokens.push(InterpretedToken::Par(Box::new(StaticParameter::from_val(
                            val,
                        ))));
                    }
                    ParserResult::Bounce(seq) => {
                        if seq.len() == 3 {
                            itokens.push(InterpretedToken::Par(Box::new(
                                BounceParameter::from_params(seq[0], seq[1], seq[2]),
                            )));
                        } else if seq.len() == 2 {
                            itokens.push(InterpretedToken::Par(Box::new(
                                BounceParameter::from_params(seq[0], seq[1], 6000.0),
                            )));
                        } else {
                            itokens.push(InterpretedToken::Par(Box::new(
                                BounceParameter::from_params(0.0, 1.0, 6000.0),
                            )));
                        }
                    }
                    ParserResult::Ramp(seq) => {
                        if seq.len() == 3 {
                            itokens.push(InterpretedToken::Par(Box::new(
                                RampParameter::from_params(seq[0], seq[1], seq[2]),
                            )));
                        } else if seq.len() == 2 {
                            itokens.push(InterpretedToken::Par(Box::new(
                                RampParameter::from_params(seq[0], seq[1], 6000.0),
                            )));
                        } else {
                            itokens.push(InterpretedToken::Par(Box::new(
                                RampParameter::from_params(0.0, 1.0, 6000.0),
                            )));
                        }
                    }
                    ParserResult::Choose(seq) => {
                        itokens.push(InterpretedToken::Par(Box::new(ChooseParameter::from_seq(
                            &seq,
                        ))));
                    }
                    ParserResult::Cycle(seq) => {
                        itokens.push(InterpretedToken::Par(Box::new(CycleParameter::from_seq(
                            &seq,
                        ))));
                    }
                }
            }

            let mut cur_name: String = "".to_owned();
            let mut idrain = itokens.drain(..);
            while let Some(t) = idrain.next() {
                match t {
                    InterpretedToken::String(ref val) if val == "img" => {
                        if let Some(InterpretedToken::String(name)) = idrain.next() {
                            let img_path = model.asset_path.join("images").join(name.clone());
                            if let Ok(image) = open(img_path) {
                                images.insert(name.clone(), image);
                            } else {
                                break;
                            }

                            cur_name = name.clone();
                            parameters.insert(cur_name.to_string(), Vec::<ImgParams>::new());
                        }
                    }
                    InterpretedToken::String(ref val) if val == "pos" => {
                        if let Some(InterpretedToken::Par(px)) = idrain.next() {
                            if let Some(InterpretedToken::Par(py)) = idrain.next() {
                                positions.insert(cur_name.to_string(), ImgParams::Position(px, py));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "size" => {
                        if let Some(InterpretedToken::Par(px)) = idrain.next() {
                            if let Some(InterpretedToken::Par(py)) = idrain.next() {
                                sizes.insert(cur_name.to_string(), ImgParams::Size(px, py));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "crop" => {
                        if let Some(InterpretedToken::Par(px)) = idrain.next() {
                            if let Some(InterpretedToken::Par(py)) = idrain.next() {
                                if let Some(InterpretedToken::Par(pw)) = idrain.next() {
                                    if let Some(InterpretedToken::Par(ph)) = idrain.next() {
                                        if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                            param_vec.push(ImgParams::Crop(px, py, pw, ph));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "scatter" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::Scatter(f));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "blur" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::Blur(f));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "brighten" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::Brighten(f));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "huerot" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::HueRot(f));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "contrast" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::Contrast(f));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "opacity" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::Opacity(f));
                            }
                        }
                    }
                    InterpretedToken::String(ref val) if val == "brownian" => {
                        if let Some(InterpretedToken::Par(f)) = idrain.next() {
                            if let Some(param_vec) = parameters.get_mut(&cur_name) {
                                param_vec.push(ImgParams::Brownian(f));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    model.positions = positions;
    model.images = images;
    model.sizes = sizes;
    model.parameters = parameters;

    model.textures.clear();
}

fn model(app: &App) -> Model {
    // this currently doesn't have any effect
    app.set_loop_mode(LoopMode::rate_fps(24.0));
    // Create a window.
    let draw_window_id = app
        .new_window()
        .title("sampler")
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let code_window_id = app
        .new_window()
        .title("code")
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(code_window_id).unwrap();

    let egui = Egui::from_window(&window);

    let text = "".to_string();

    // Load the image from disk and upload it to a GPU texture.
    Model {
        textures: Vec::new(),
        draw_window_id,
        code_window_id,
        text,
        egui,
        parameters: HashMap::new(),
        images: HashMap::new(),
        positions: HashMap::new(),
        sizes: HashMap::new(),
        asset_path: app.assets_path().unwrap(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let egui = &mut model.egui;

    let ctx = egui.begin_frame();
    egui::Window::new("Code").show(&ctx, |ui| {
        ui.add_sized(
            ui.available_size(),
            egui::TextEdit::multiline(&mut model.text),
        );
    });

    for (n, source_image) in model.images.iter() {
        let mut image = source_image.clone();

        let mut x = 0.0_f32;
        let mut y = 0.0_f32;
        let mut w = 50.0_f32;
        let mut h = 50.0_f32;

        if let Some(params) = model.parameters.get_mut(n) {
            if let Some(ImgParams::Position(xp, yp)) = model.positions.get_mut(n) {
                x = xp.get_next();
                y = yp.get_next();
            }

            if let Some(ImgParams::Size(wp, hp)) = model.sizes.get_mut(n) {
                w = wp.get_next();
                h = hp.get_next();
            }

            for param in params.iter_mut() {
                match param {
                    ImgParams::Blur(f) => {
                        image = image.blur(f.get_next());
                    }
                    ImgParams::Brighten(f) => {
                        image = image.brighten(f.get_next() as i32);
                    }
                    ImgParams::Contrast(f) => {
                        image = image.adjust_contrast(f.get_next());
                    }
                    ImgParams::HueRot(f) => {
                        image = image.huerotate(f.get_next() as i32);
                    }
                    ImgParams::Crop(x, y, w, h) => {
                        image = image.crop(
                            ((x.get_next() + 0.01) * image.width() as f32) as u32,
                            ((y.get_next() + 0.01) * image.height() as f32) as u32,
                            ((w.get_next() + 0.01) * image.width() as f32) as u32,
                            ((h.get_next() + 0.01) * image.height() as f32) as u32,
                        )
                    }
                    ImgParams::Opacity(o) => {
                        let val = o.get_next();
                        let mut ibuf = image.clone().into_rgba8();

                        for p in ibuf.pixels_mut() {
                            *p = p.map_with_alpha(|x| x, |a| (a as f32 * val) as u8);
                        }
                        image = DynamicImage::ImageRgba8(ibuf);
                    }
                    ImgParams::Brownian(f) => {
                        let mut rng = rand::thread_rng();
                        let thresh_x: f64 = rng.gen();
                        let thresh_y: f64 = rng.gen();

                        let val = f.get_next();
                        if thresh_x < 0.5 {
                            x += val;
                        } else {
                            x -= val;
                        }
                        if thresh_y < 0.5 {
                            y += val;
                        } else {
                            y -= val;
                        }
                    }
                    ImgParams::Scatter(f) => {
                        let mut rng = rand::thread_rng();
                        let val = f.get_next();
                        let scatter_x: f32 = rng.gen::<f32>() * val;
                        let scatter_y: f32 = rng.gen::<f32>() * val;
                        x *= scatter_x;
                        y *= scatter_y;
                    }
                    _ => {}
                }
            }

            // to be save ...
            if w == 0.0 {
                w = 1.0;
            }
            if h == 0.0 {
                h = 1.0;
            }
        }

        if model.textures.len() >= 500 {
            model.textures.clear();
        }

        model
            .textures
            .push((wgpu::Texture::from_image(app, &image), x, y, w, h));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    match frame.window_id() {
        id if id == model.code_window_id => {
            model.egui.draw_to_frame(&frame).unwrap();
        }
        id if id == model.draw_window_id => {
            if model.textures.is_empty() {
                draw.background().color(BLACK);
            } else {
                for (t, x, y, w, h) in model.textures.iter() {
                    draw.texture(&t).x(*x).y(*y).wh(Vec2::new(*w, *h));
                }
            }
        }
        _ => {}
    }

    draw.to_frame(app, &frame).unwrap();
}
