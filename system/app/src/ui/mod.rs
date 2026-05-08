use alg::processor::DetectionMode;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::{
    features::{keyboard::RebuildKeyboard, pipeline::{DeviceList, RestartPipeline}}, settings::data::{AppSettings, KeyboardKind}, state::AppState
};



#[derive(Resource, Clone)]
pub struct SettingsUiState {
    pub draft: AppSettings,
}

impl FromWorld for SettingsUiState {
    fn from_world(world: &mut World) -> Self {
        let settings = world.resource::<AppSettings>();

        Self {
            draft: settings.clone(),
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MenuState::default())
            .init_resource::<SettingsUiState>()
            .add_systems(EguiPrimaryContextPass, ui_menu_system);
    }
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub open: bool,
}

pub fn ui_menu_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut menu_state: ResMut<MenuState>,

    settings: Res<AppSettings>,
    mut ui_state: ResMut<SettingsUiState>,

    device_list: Option<Res<DeviceList>>,

    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
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

    let draft = &mut ui_state.draft;

    egui::Window::new("Menu")
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            
            // nav (temporary)

            ui.label("Navigate");
            ui.separator();

            let states = [
                (AppState::MainMenu, "Main Menu"),
                (AppState::LiveListen, "Live Listen"),
                (AppState::ModelRecording, "Model Recording"),
            ];

            for (state, label) in states {
                let active = *current_state.get() == state;

                if ui.selectable_label(active, label).clicked() && !active {
                    next_state.set(state);
                    menu_state.open = false;
                }
            }

            // piano kind section

            ui.add_space(8.0);
            ui.label("Piano");
            ui.separator();

            ui.add(
                egui::Slider::new(&mut draft.note_speed, 10.0..=400.0)
                    .text("Note Speed"),
            );

            egui::ComboBox::from_label("Keyboard")
                .selected_text(match draft.keyboard_kind {
                    KeyboardKind::Piano88 => "88 Keys",
                    KeyboardKind::PianoSmall => "Small Keyboard",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut draft.keyboard_kind,
                        KeyboardKind::Piano88,
                        "88 Keys",
                    );

                    ui.selectable_value(
                        &mut draft.keyboard_kind,
                        KeyboardKind::PianoSmall,
                        "Small Keyboard",
                    );
                });

            // audio device secion

            if let Some(device_list) = device_list {
                ui.add_space(8.0);
                ui.label("Audio");
                ui.separator();

                let current_device =
                    draft.device_name.as_deref().unwrap_or("None");

                egui::ComboBox::from_label("Input Device")
                    .selected_text(current_device)
                    .show_ui(ui, |ui| {
                        for device in &device_list.0 {
                            let selected =
                                draft.device_name.as_deref()
                                    == Some(device.name.as_str());

                            if ui
                                .selectable_label(selected, &device.name)
                                .clicked()
                            {
                                draft.device_name =
                                    Some(device.name.clone());
                            }
                        }
                    });
            }

            // detection mode section

            ui.add_space(8.0);
            ui.label("Detection");
            ui.separator();

            egui::ComboBox::from_label("Mode")
                .selected_text(match draft.detection_mode {
                    DetectionMode::Polyphonic => "Polyphonic",
                    DetectionMode::Monophonic => "Monophonic",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut draft.detection_mode,
                        DetectionMode::Polyphonic,
                        "Polyphonic",
                    );

                    ui.selectable_value(
                        &mut draft.detection_mode,
                        DetectionMode::Monophonic,
                        "Monophonic",
                    );
                });

            // models section

            if draft.detection_mode == DetectionMode::Polyphonic {
                ui.add_space(8.0);
                ui.label("Model");
                ui.separator();

                if draft.models.is_empty() {
                    ui.label("No models configured.");
                } else {
                    let current_label = draft
                        .active_model_index
                        .and_then(|i| draft.models.get(i))
                        .map(|m| m.name.as_str())
                        .unwrap_or("None");

                    let model_names: Vec<String> =
                        draft.models.iter().map(|m| m.name.clone()).collect();

                    egui::ComboBox::from_label("Active Model")
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            for (i, name) in model_names.iter().enumerate() {
                                ui.selectable_value(
                                    &mut draft.active_model_index,
                                    Some(i),
                                    name,
                                );
                            }
                        });

                    if let Some(model) = draft
                        .active_model_index
                        .and_then(|i| draft.models.get(i))
                    {
                        ui.label(model.summary());
                    }
                }
            }
        });


    // TODO: figure out a cleaner way to do this ui stuff

    if *settings != ui_state.draft {
        let pipeline_changed =
            settings.device_name != ui_state.draft.device_name
            || settings.detection_mode != ui_state.draft.detection_mode
            || settings.active_model_index != ui_state.draft.active_model_index;
 
        let keyboard_changed = settings.keyboard_kind != ui_state.draft.keyboard_kind;
 
        commands.insert_resource(ui_state.draft.clone());
 
        if pipeline_changed {
            commands.trigger(RestartPipeline);
        }
 
        if keyboard_changed && *current_state.get() == AppState::LiveListen {
            commands.trigger(RebuildKeyboard);
        }
    }


}