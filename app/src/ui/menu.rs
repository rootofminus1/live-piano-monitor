use bevy::ecs::resource::Resource;
use bevy::ecs::system::ResMut;
use bevy_egui::{EguiContexts, egui};


#[derive(Resource)]
pub struct UiSettings {
    pub note_speed: f32,
    pub keyboard_kind: KeyboardKind
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            note_speed: 100.0,
            keyboard_kind: KeyboardKind::Piano88
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyboardKind {
    Piano88,
    PianoSmall,
}


#[derive(Resource, Default)]
pub struct UiState {
    menu_open: bool,
}

pub fn ui_menu_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut settings: ResMut<UiSettings>,
) {
    let ctx = contexts.ctx_mut().unwrap();

    egui::Area::new("hamburger_button".into())
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
        .show(ctx, |ui| {
            if ui.button("☰").clicked() {
                ui_state.menu_open = !ui_state.menu_open;
            }
        });

    if ui_state.menu_open {
        egui::Window::new("Menu")
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {

                ui.label("Piano Controls");

                ui.separator();

                ui.add(
                    egui::Slider::new(&mut settings.note_speed, 10.0..=400.0)
                        .text("Note Speed")
                );

                egui::ComboBox::from_label("Keyboard")
                    .selected_text(match settings.keyboard_kind {
                        KeyboardKind::Piano88 => "88 Keys",
                        KeyboardKind::PianoSmall => "Small Keyboard",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut settings.keyboard_kind,
                            KeyboardKind::Piano88,
                            "88 Keys"
                        );

                        ui.selectable_value(
                            &mut settings.keyboard_kind,
                            KeyboardKind::PianoSmall,
                            "Small Keyboard"
                        );
                    });

            });
    }
}
