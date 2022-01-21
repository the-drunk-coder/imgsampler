use nannou::prelude::{Rect, *};
use nannou::image::{open, GenericImageView};
use nannou_conrod as ui;
use nannou_conrod::prelude::*;

use std::collections::HashMap;

use rand::Rng;

#[derive(Debug)]
enum ImgParams {
    Position(f32, f32),
    Size(f32, f32),
    Crop(f32, f32, f32, f32),
    Blur(f32),
    Opacity(f32),
    Scatter(f32),
    Brownian(f32),
}

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    textures: HashMap<String, wgpu::Texture>,
    positions: HashMap<String, ImgParams>,
    sizes: HashMap<String, ImgParams>,
    parameters: HashMap<String, Vec<ImgParams>>,
    ids: Ids,
    draw_id: WindowId,
    ui_id: WindowId,
    ui: Ui,
    text: String,
    asset_path: std::path::PathBuf,
}

widget_ids! {
    struct Ids {
        text,
    }
}

fn model(app: &App) -> Model {
    // this currently doesn't have any effect
    app.set_loop_mode(LoopMode::rate_fps(24.0));
    // Create a window.
    let draw_id = app.new_window().title("sampler").build().unwrap();

    let ui_id = app
        .new_window()
        .title("code")
        .transparent(true)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    // Create the UI for our window.
    let mut ui = ui::builder(app).window(ui_id).build().unwrap();
    // Generate some ids for our widgets.
    let ids = Ids::new(ui.widget_id_generator());

    let text = "".to_string();

    // Load the image from disk and upload it to a GPU texture.
    Model {
        textures: HashMap::new(),
        positions: HashMap::new(),
        parameters: HashMap::new(),
	sizes: HashMap::new(),
        ids,
        draw_id,
        ui_id,
        ui,
        text,
        asset_path: app.assets_path().unwrap(),
    }
}

fn raw_window_event(app: &App, model: &mut Model, event: &ui::RawWindowEvent) {
    model.ui.handle_raw_event(app, event);
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let ui = &mut model.ui.set_widgets();

    if let Some(value) = widget::TextEdit::new(&model.text)
        .top_left_with_margin(10.0)
        .w_h(1000.0, 200.0)
        .set(model.ids.text, ui)
    {
        model.text = value;
        let mut parameters = HashMap::<String, Vec<ImgParams>>::new();
        let mut positions = HashMap::<String, ImgParams>::new();
	let mut sizes = HashMap::<String, ImgParams>::new();
        let mut img_names = Vec::new();
        let lines = model.text.split('\n');
        for line in lines {
            if matches!(line.chars().next(), Some('#')) {
                continue;
            }

            let mut tokens = line.split(' ');

            let mut cur_name = "";
            while let Some(t) = tokens.next() {
                match t {
                    "img" => {
                        if let Some(name) = tokens.next() {
                            img_names.push(name.clone());
                            cur_name = name.clone();
                            parameters.insert(cur_name.to_string(), Vec::<ImgParams>::new());
                        }
                    }
                    "pos" => {
                        if let Some(x_str) = tokens.next() {
                            if let Some(y_str) = tokens.next() {
                                if let Ok(x) = x_str.parse::<f32>() {
                                    if let Ok(y) = y_str.parse::<f32>() {
                                        positions.insert(
                                            cur_name.to_string(),
                                            ImgParams::Position(x, y),
                                        );
                                    }
                                }
                            }
                        }
                    }
		    "size" => {
                        if let Some(x_str) = tokens.next() {
                            if let Some(y_str) = tokens.next() {
                                if let Ok(x) = x_str.parse::<f32>() {
                                    if let Ok(y) = y_str.parse::<f32>() {
                                        sizes.insert(
                                            cur_name.to_string(),
                                            ImgParams::Size(x, y),
                                        );
                                    }
                                }
                            }
                        }
                    }
		    "crop" => {
                        if let Some(x_str) = tokens.next() {
                            if let Some(y_str) = tokens.next() {
				if let Some(w_str) = tokens.next() {
				    if let Some(h_str) = tokens.next() {
					if let Ok(x) = x_str.parse::<f32>() {
					    if let Ok(y) = y_str.parse::<f32>() {
						if let Ok(w) = w_str.parse::<f32>() {
						    if let Ok(h) = h_str.parse::<f32>() {
							if w != 0.0 && h != 0.0 {
							    if let Some(param_vec) = parameters.get_mut(cur_name) {
								param_vec.push(ImgParams::Crop(x,y,w,h));
							    }
							}														
						    }
						}
					    }
					}
				    }
				}
                            }
                        }
                    }
                    "scatter" => {
                        if let Some(factor_str) = tokens.next() {
                            if let Ok(factor) = factor_str.parse::<f32>() {
                                if let Some(param_vec) = parameters.get_mut(cur_name) {
                                    param_vec.push(ImgParams::Scatter(factor));
                                }
                            }
                        }
                    }
                    "blur" => {
                        if let Some(factor_str) = tokens.next() {
                            if let Ok(factor) = factor_str.parse::<f32>() {
                                if let Some(param_vec) = parameters.get_mut(cur_name) {
                                    param_vec.push(ImgParams::Blur(factor));
                                }
                            }
                        }
                    }
                    "opacity" => {
                        if let Some(factor_str) = tokens.next() {
                            if let Ok(factor) = factor_str.parse::<f32>() {
                                if let Some(param_vec) = parameters.get_mut(cur_name) {
                                    param_vec.push(ImgParams::Opacity(factor));
                                }
                            }
                        }
                    }
                    "brownian" => {
                        if let Some(factor_str) = tokens.next() {
                            if let Ok(factor) = factor_str.parse::<f32>() {
                                if let Some(param_vec) = parameters.get_mut(cur_name) {
                                    param_vec.push(ImgParams::Brownian(factor));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut textures = HashMap::new();
        for name in img_names.drain(..) {
            let img_path = model.asset_path.join("images").join(name);
            if let Ok(mut image) = open(img_path) {
				
		for param in parameters[name].iter() {
		    match param {
			ImgParams::Blur(f) => {
			    image = image.blur(*f);
			}
			ImgParams::Crop(x,y,w,h) => {
			    image = image.crop(
				(*x * image.width() as f32) as u32,
				(*y * image.height() as f32) as u32,
				(*w * image.width() as f32) as u32,
				(*h * image.height() as f32) as u32,
			    )
			}
			_ => {}
		    }
		}
		
                textures.insert(name.to_string(), wgpu::Texture::from_image(app, &image));                
            }
        }
        model.textures = textures;
        model.positions = positions;
	model.sizes = sizes;
        model.parameters = parameters;
    }

    // update positions
    for (n, params) in model.parameters.iter() {
        let mut x = 0.0_f32;
        let mut y = 0.0_f32;

        if let Some(ImgParams::Position(xp, yp)) = model.positions.get(n) {
            x = *xp;
            y = *yp;
        }

        for par in params.iter() {
            match par {
                
                ImgParams::Brownian(f) => {
                    let mut rng = rand::thread_rng();
                    let thresh_x: f64 = rng.gen();
                    let thresh_y: f64 = rng.gen();

                    if thresh_x < 0.5 {
                        x += f;
                    } else {
                        x -= f;
                    }
                    if thresh_y < 0.5 {
                        y += f;
                    } else {
                        y -= f;
                    }
                }
		ImgParams::Scatter(f) => {
		    let mut rng = rand::thread_rng();
		    let scatter_x: f32 = rng.gen::<f32>() * *f;
		    let scatter_y: f32 = rng.gen::<f32>() * *f;
                    x *= scatter_x;
                    y *= scatter_y;
                }
                _ => {}
            }
        }
        model
            .positions
            .insert(n.to_string(), ImgParams::Position(x, y));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    match frame.window_id() {
        id if id == model.ui_id => {
            draw.background()
                .color(rgba(0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32));
            draw.to_frame(app, &frame).unwrap();
            model.ui.draw_to_frame(app, &frame).unwrap();
        }
        id if id == model.draw_id => {
            for (n, t) in model.textures.iter() {
                let r = if let Some(ImgParams::Size(xp, yp)) = model.sizes.get(n) {
		    Rect::from_w_h(*xp, *yp)
		} else {
		    Rect::from_w_h(50.0_f32, 50.0_f32)
		};

                if let Some(ImgParams::Position(xp, yp)) = model.positions.get(n) {
                    draw.texture(&t).x(*xp).y(*yp).wh(r.wh());
                } else {
                    draw.texture(&t).x(0.0).y(0.0).wh(r.wh());
                }
            }
            draw.to_frame(app, &frame).unwrap();
        }
        _ => {}
    }
}
