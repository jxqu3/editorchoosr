#![windows_subsystem = "windows"]

use eframe::{egui};
use egui::{ColorImage, Key, TextureHandle, TextureOptions, Visuals};
use image::{DynamicImage, RgbaImage};

use std::{env, fs, process::Command};
use yaml_rust2::{YamlLoader};

const ICON_SIZE: u16 = 32;
struct Editor {
    name: String,
    path: String,
    icon_handle: TextureHandle,
}
struct EditorSelectorApp {
    editors: Vec<Editor>,
    original_file: String,
    should_run: i32
}

impl EditorSelectorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load from yaml
        let mut editors: Vec<Editor> = vec![];
        let binding = std::env::current_exe().unwrap();
        let current_path = binding.parent().expect("failed to get parent folder");
        println!("{:?}", current_path);

        let conf_path = format!("{}\\config.yaml", current_path.to_str().unwrap());
        println!("{}", conf_path);

        if let Ok(conf) = fs::read_to_string(conf_path) {
            if let Ok(docs) = YamlLoader::load_from_str(conf.as_str()) {
                let doc = &docs[0];
                for (id, editor) in doc["editors"].clone().into_iter().enumerate() {
                    let name = editor["name"].as_str().expect(&format!("Missing name in editor {id}."));
                    let path = editor["path"].as_str().expect(&format!("Missing path in editor {id}."));
                    let icon_handle = load_icon_exe(path, &cc.egui_ctx, name);
                    editors.push(Editor {
                        name: format!("{}: {name}", id+1),
                        path: path.to_owned(),
                        icon_handle: icon_handle,
                    });
                }
            }
        }

        let args: Vec<String> = env::args().collect();
        let original_file = args[1].clone();

        Self {
            editors,
            original_file: original_file,
            should_run: -1,
        }
    }
}

impl eframe::App for EditorSelectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.should_run != -1 {
                Command::new(self.editors[self.should_run as usize].path.clone())
                .args([&self.original_file])
                .output()
                .expect("failed to run");

                println!("exiting");
                self.should_run = -1;
                fs::copy(&self.original_file, &self.original_file).expect("Wtf, failed to copy :(");
                if self.original_file != self.original_file {
                    fs::remove_file(&self.original_file).expect("failed to remove old file");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ui.heading("Select Editor to Open");

            ui.centered_and_justified(|ui| {
                for (id, editor) in self.editors.iter().enumerate() {
                    ui.vertical_centered(|ui| {
                        let sized_image = egui::load::SizedTexture::new(editor.icon_handle.id(), egui::vec2(ICON_SIZE as f32, ICON_SIZE as f32));
                        let image = egui::Image::from_texture(sized_image);
                        ui.add(image);
                        if ui.button(&editor.name).clicked() {
                            // Open editor.
                            self.should_run = id as i32;
                        }
                        ctx.input(|i| {
                            let key_name =format!("{}", id+1);
                            if i.key_pressed( egui::Key::from_name(&key_name).unwrap_or(Key::Insert)) {
                                println!("Pressed {}", key_name);
                                self.should_run = id as i32;
                            }
                        });
                    });
                }
            });
        });
    }
}

use file_icon_provider::get_file_icon;

fn load_icon_exe(path: &str, ctx: &egui::Context, name: &str) -> egui::TextureHandle {
    let icon = get_file_icon(path, ICON_SIZE).expect(&format!("Failed to get icon for {path}"));
    let binding = RgbaImage::from_raw(icon.width, icon.height, icon.pixels)
        .map(DynamicImage::ImageRgba8)
        .expect("Failed to convert icon to Image");
    let bytes = binding
        .as_bytes();
    let color_image = ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], bytes);
    let handle = ctx.load_texture(name, color_image, TextureOptions::default());

    handle
}

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 600.0]),
        centered: true,
        vsync: true,
        ..Default::default()
    };
    let _ = eframe::run_native("editorchoosr", native_options, Box::new(|cc| {
        Ok(Box::new(EditorSelectorApp::new(cc)))
    }));
}
