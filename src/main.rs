mod parameter;
mod line_parser;

use nannou::prelude::*;
use nannou::image::{open, GenericImageView, DynamicImage};
use nannou_conrod as ui;
use nannou_conrod::prelude::*;
use std::collections::HashMap;
use rand::Rng;

use line_parser::ParserResult;
use parameter::*;

enum ImgParams {
    Position(Box<dyn Parameter>, Box<dyn Parameter>),
    Size(Box<dyn Parameter>, Box<dyn Parameter>),
    Crop(Box<dyn Parameter>, Box<dyn Parameter>, Box<dyn Parameter>, Box<dyn Parameter>),
    Blur(Box<dyn Parameter>),
    Opacity(Box<dyn Parameter>),
    Scatter(Box<dyn Parameter>),
    Brownian(Box<dyn Parameter>),
}

enum InterpretedToken {
    String(String),
    Par(Box<dyn Parameter>)
}

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    images: HashMap<String, DynamicImage>,
    textures: Vec<(wgpu::Texture, f32, f32, f32, f32)>,
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
	images: HashMap::new(),
        textures: Vec::new(),
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

	// go stateless for once 
	model.text = value;
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
		let mut itokens:Vec<InterpretedToken> = Vec::new();
		for token in token_vec.drain(..) {
		    match token {
			ParserResult::String(val) => {
			    itokens.push(InterpretedToken::String(val));
			}
			ParserResult::Scalar(val) => {
			    itokens.push(InterpretedToken::Par(Box::new(StaticParameter::from_val(val))));
			}
			ParserResult::Bounce(seq) => {
			    if seq.len() == 3 {
				itokens.push(InterpretedToken::Par(Box::new(BounceParameter::from_params(seq[0], seq[1], seq[2]))));
			    } else if seq.len() == 2 {
				itokens.push(InterpretedToken::Par(Box::new(BounceParameter::from_params(seq[0], seq[1], 6000.0))));
			    } else {
				itokens.push(InterpretedToken::Par(Box::new(BounceParameter::from_params(0.0, 1.0, 6000.0))));
			    }
			}
			ParserResult::Ramp(seq) => {
			    if seq.len() == 3 {
				itokens.push(InterpretedToken::Par(Box::new(RampParameter::from_params(seq[0], seq[1], seq[2]))));
			    } else if seq.len() == 2 {
				itokens.push(InterpretedToken::Par(Box::new(RampParameter::from_params(seq[0], seq[1], 6000.0))));
			    } else {
				itokens.push(InterpretedToken::Par(Box::new(RampParameter::from_params(0.0, 1.0, 6000.0))));
			    }
			}
			ParserResult::Choose(seq) => {
			    itokens.push(InterpretedToken::Par(Box::new(ChooseParameter::from_seq(&seq))));
			}
			ParserResult::Cycle(seq) => {
			    itokens.push(InterpretedToken::Par(Box::new(CycleParameter::from_seq(&seq))));
			}
		    }
		}
		
		let mut cur_name:String = "".to_owned();
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
                                    positions.insert(
					cur_name.to_string(),
					ImgParams::Position(px, py),
                                    );
				}
			    }
			}
			InterpretedToken::String(ref val) if val == "size" => {
			    if let Some(InterpretedToken::Par(px)) = idrain.next() {
				if let Some(InterpretedToken::Par(py)) = idrain.next() {
                                    println!("insert {} size", cur_name);
                                    sizes.insert(
					cur_name.to_string(),
					ImgParams::Size(px, py),
                                    );
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

	model.images = images;
        model.positions = positions;
	model.sizes = sizes;
        model.parameters = parameters;
    }
    
    model.textures.clear();
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
		    ImgParams::Crop(x,y,w,h) => {
			image = image.crop(
			    ((x.get_next() + 0.01) * image.width() as f32) as u32,
			    ((y.get_next() + 0.01) * image.height() as f32) as u32,
			    ((w.get_next() + 0.01) * image.width() as f32) as u32,
			    ((h.get_next() + 0.01) * image.height() as f32) as u32,
			)
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
	//println!("{} {} {} {} {}", n, x, y, w, h);

        model.textures.push((wgpu::Texture::from_image(app, &image), x, y, w, h));    

	

	
	
	
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
	    if model.textures.is_empty() {
		draw.background().color(BLACK);
	    } else {
		for (t, x, y, w, h) in model.textures.iter() {              
                    draw.texture(&t).x(*x).y(*y).wh(Vec2::new(*w, *h));                
		}
	    }
            
            draw.to_frame(app, &frame).unwrap();
        }
        _ => {}
    }
}
