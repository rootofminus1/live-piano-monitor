use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use alg::processor::DetectionMode;

use crate::{
    audio::DeviceList,
    settings::data::{AppSettings, KeyboardKind},
    settings::plugin::SettingsDirty,
};

#[derive(Resource, Default)]
pub struct MenuState {
    pub open: bool,
}

pub fn ui_menu_system(
    mut contexts: EguiContexts,
    mut menu_state: ResMut<MenuState>,
    mut settings: ResMut<AppSettings>,
    mut dirty: ResMut<SettingsDirty>,
    device_list: Res<DeviceList>,
) {
    let ctx = contexts.ctx_mut().unwrap();

    egui::Area::new("hamburger_button".into())
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
        .show(ctx, |ui| {
            if ui.button("☰").clicked() {
                menu_state.open = !menu_state.open;
            }
        });

    if !menu_state.open {
        return;
    }

    egui::Window::new("Menu")
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {

            ui.label("Piano");
            ui.separator();

            let prev = settings.note_speed;
            ui.add(egui::Slider::new(&mut settings.note_speed, 10.0..=400.0).text("Note Speed"));
            if settings.note_speed != prev { dirty.0 = true; }

            let prev = settings.keyboard_kind;
            egui::ComboBox::from_label("Keyboard")
                .selected_text(match settings.keyboard_kind {
                    KeyboardKind::Piano88 => "88 Keys",
                    KeyboardKind::PianoSmall => "Small Keyboard",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.keyboard_kind, KeyboardKind::Piano88, "88 Keys");
                    ui.selectable_value(&mut settings.keyboard_kind, KeyboardKind::PianoSmall, "Small Keyboard");
                });
            if settings.keyboard_kind != prev { dirty.0 = true; }

            ui.add_space(8.0);
            ui.label("Audio");
            ui.separator();

            let current_device = settings.device_name.as_deref().unwrap_or("None");
            let prev = settings.device_name.clone();
            egui::ComboBox::from_label("Input Device")
                .selected_text(current_device)
                .show_ui(ui, |ui| {
                    for device in &device_list.0 {
                        let selected = settings.device_name.as_deref() == Some(device.name.as_str());
                        if ui.selectable_label(selected, &device.name).clicked() {
                            settings.device_name = Some(device.name.clone());
                        }
                    }
                });
            if settings.device_name != prev { dirty.0 = true; }

            // detection
            ui.add_space(8.0);
            ui.label("Detection");
            ui.separator();

            let prev = settings.detection_mode;
            egui::ComboBox::from_label("Mode")
                .selected_text(match settings.detection_mode {
                    DetectionMode::Polyphonic => "Polyphonic",
                    DetectionMode::Monophonic => "Monophonic",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.detection_mode, DetectionMode::Polyphonic, "Polyphonic");
                    ui.selectable_value(&mut settings.detection_mode, DetectionMode::Monophonic, "Monophonic");
                });
            if settings.detection_mode != prev { dirty.0 = true; }

            // model (for poly only)
            if settings.detection_mode == DetectionMode::Polyphonic {
                ui.add_space(8.0);
                ui.label("Model");
                ui.separator();

                if settings.models.is_empty() {
                    ui.label("No models configured.");
                } else {
                    let current_label = settings.active_model_index
                        .and_then(|i| settings.models.get(i))
                        .map(|m| m.name.as_str())
                        .unwrap_or("None");

                    let prev = settings.active_model_index;
                    let model_names: Vec<String> = settings.models.iter().map(|m| m.name.clone()).collect();

                    egui::ComboBox::from_label("Active Model")
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            for (i, name) in model_names.iter().enumerate() {
                                ui.selectable_value(&mut settings.active_model_index, Some(i), name);
                            }
                        });
                    if settings.active_model_index != prev { dirty.0 = true; }

                    if let Some(m) = settings.active_model_index.and_then(|i| settings.models.get(i)) {
                        ui.label(m.summary());
                    }
                }
            }
        });
}