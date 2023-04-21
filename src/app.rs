use std::collections::{HashMap, HashSet};

use eframe::egui::{self};
use eframe::epi;
use egui::Ui;

use crate::components::save_file_selector::SaveFileSelector;
use crate::components::detail_view::{DetailView};
use crate::parser::SaveInfo;
use crate::sktypes::skui_value::{SkUIValue, UIValueType};
use crate::{load_installed, load_mod_map};
use crate::{load_saveinfo_from_path, mod_search::vortex_scanner::Plugin};

#[derive(Clone)]
pub struct AppState {
    pub file_path: String,
    pub save_info: Option<SaveInfo>,
    pub mod_map: HashMap<String, Plugin>,
    pub installed: HashSet<String>,
    pub error: Option<String>,
    pub plugins: Option<Vec<SkUIValue>>,
    pub folder_path: String,
    pub save_file_list: Vec<String>,
}

fn label_line(ui: &mut Ui, name: &str, value: &str) {
    ui.label(name);
    ui.label(value);
    ui.end_row();
}

// fn handle_file_selector_click(app_state: &mut AppState, file_selector_callback: impl Fn(&str)) {
//     let res = rfd::FileDialog::new()
//     .add_filter("Elder Scrolls Save", &["ess"])
//     .set_directory("./input")
//     .pick_file();

//     match res {
//         Some(path_buf) => {
//             app_state.file_path = String::from(path_buf.to_str().unwrap());
//             let new_state = load_savegame_file(app_state.clone());
//             *app_state = new_state;
//         }
//         None => tracing::error!("No file selected"),
//     }
// }

pub fn convert_plugins_to_skui(plugins: Vec<String>) -> Vec<SkUIValue> {
    let mut skui_plugins = Vec::new();
    for plugin in plugins {
        let new_plugin = SkUIValue::new(plugin.as_str(), plugin.to_string(), UIValueType::Plugin);
        skui_plugins.push(new_plugin);
    }
    skui_plugins
}

fn load_savegame_file(ast: AppState) -> AppState {
    let mut app_state = ast.clone();
    let path = app_state.file_path.to_string();
    tracing::info!("Loading file: {}", path);

    match load_saveinfo_from_path(path) {
        Ok(values) => {
            if values.header.is_se {
                app_state.mod_map = load_mod_map("skyrimse");
                app_state.installed = load_installed("skyrimse");
            } else {
                app_state.mod_map = load_mod_map("skyrim");
                app_state.installed = load_installed("skyrim");
            }
            app_state.error = None;

            let mut plugins = Vec::new();
            for plugin_name in &values.plugin_info.plugins {
                let new_plugin = SkUIValue::new(
                    plugin_name.as_str(),
                    plugin_name.to_string(),
                    UIValueType::Plugin,
                );
                plugins.push(new_plugin);
            }

            app_state.plugins = Some(plugins);
            app_state.save_info = Some(values);
        }
        Err(e) => {
            app_state.error = Some(e.to_string());
            app_state.save_info = None;
            app_state.plugins = None;
        }
    };

    return app_state;
}

fn handle_folder_selector_click(app_state: &mut AppState) {
    let res = rfd::FileDialog::new()
    .pick_folder();

    match res {
        Some(path_buf) => {
            app_state.folder_path = String::from(path_buf.to_str().unwrap());
            tracing::info!("Selected folder: {}", app_state.folder_path);

            // List files in folder_path
            let mut files = Vec::new();
            for entry in std::fs::read_dir(app_state.folder_path.to_string()).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }

            app_state.save_file_list = files.iter().map(|x| x.to_str().unwrap().to_string()).collect();
        }
        None => tracing::error!("No folder selected"),
    }
}


impl epi::App for AppState {

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::SidePanel::left("side-panel").show(ctx, |ui| {
            SaveFileSelector::new(&mut self.save_file_list).show(ui, |item| {
                tracing::info!("File was selected: {}", item);
                self.file_path = item.to_string();
                match load_saveinfo_from_path(self.file_path.to_string()) {
                    Ok(save_file) => {
                        if save_file.header.is_se {
                            self.mod_map = load_mod_map("skyrimse");
                            self.installed = load_installed("skyrimse");
                        } else {
                            self.mod_map = load_mod_map("skyrim");
                            self.installed = load_installed("skyrim");
                        }
                        
                        self.save_info = Some(save_file);
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            DetailView::new(&mut self.file_path, &self.save_info).show(ctx, ui);
        });
    }

    fn name(&self) -> &str {
        "Arcanaeum"
    }
}