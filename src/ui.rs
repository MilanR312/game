use std::borrow::Cow;

use bevy::{diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, ecs::event::EventRegistry, input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState}, prelude::*, render::diagnostic::RenderDiagnosticsPlugin, utils::HashMap, window::WindowMode};
use bevy_egui::{egui::{self, Layout, TextEdit, Ui, Widget}, EguiContexts, EguiPlugin};
use itertools::Either;
use iyes_perf_ui::{prelude::PerfUiAllEntries, PerfUiPlugin};

use crate::state::GameState;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(( // these plugins allow us to see debug info like fps
                FrameTimeDiagnosticsPlugin,
                EntityCountDiagnosticsPlugin,
                SystemInformationDiagnosticsPlugin,
                RenderDiagnosticsPlugin,
            ))
            .add_plugins(PerfUiPlugin)
            .add_systems(Update, debug_ui.run_if(in_state(GameState::Game)))
            .add_systems(Startup, setup_ui)
            .add_systems(Update, ui.run_if(in_state(GameState::Menu)));
    }
}

/// State of the ui kept as a component in the ECS
#[derive(Resource, Default)]
pub struct DebugUiState{
    pub sun_position: f32,
    pub map_displayed: bool
}


/// egui is a rust alternatieve for dear imgui, it runs every frame and is used here to give use control of the suns location
pub fn debug_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<DebugUiState>,
    
) {
    egui::Window::new("Test").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut ui_state.sun_position, 0.0 ..= 6.24))
    });
}



fn window_mode_str(mode: &WindowMode) -> &'static str {
    match mode {
        WindowMode::BorderlessFullscreen(_) => "borderless",
        WindowMode::Fullscreen(_) => "fullscreen",
        WindowMode::Windowed => "windowed",
        WindowMode::SizedFullscreen(_) => "fullscreen"
    }
}

pub type AnyKey =Either<KeyCode, MouseButton>; 

#[derive(Resource)]
pub struct UiState {
    selected_input_mapping: Option<usize>,
    pub mappings: HashMap<AnyKey, AnyKey>
}

fn key_code_name(x: Either<KeyCode, MouseButton>) -> String {
    match x {
        Either::Left(x) => format!("{x:?}"),
        Either::Right(x) => format!("{x:?}") 
    }
}

fn mapping_ui(
    ui: &mut Ui, 
    default: Either<KeyCode, MouseButton>, 
    keys: &mut EventReader<KeyboardInput>,
    mouse: &mut EventReader<MouseButtonInput>,
    ui_state: &mut UiState,
    idx: usize
){
    let keymap = ui_state.mappings.entry(default).or_insert(default);
    let is_selected = ui_state.selected_input_mapping.is_some_and(|x| x == idx);
    let text = if is_selected {
        Cow::Borrowed("please press an input")
    } else {
        Cow::Owned(key_code_name(*keymap))
    };
    ui.horizontal(|ui| {
        if ui.selectable_label(is_selected, text).clicked() {
            ui_state.selected_input_mapping = Some(idx)
        }
        if is_selected {
            let inputs =
                keys.read().filter(|x| x.state == ButtonState::Pressed)
                    .map(|x| Either::Left(x.key_code))
                    .next()
                .or(
                    mouse.read().filter(|f| f.state == ButtonState::Pressed)
                        .map(|x| Either::Right(x.button))
                        .next()
                );
            if let Some(inputs) = inputs {
                *keymap = inputs;
                ui_state.selected_input_mapping = None;
            }
        }
        if ui.button("reset").clicked() {
            *keymap = default;
        };
    });
}

pub fn ui(
    mut contexts: EguiContexts,
    mut window: Single<&mut Window>,
    mut keys: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut state: ResMut<UiState>
) {
    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui|{
        egui::Grid::new("Settings").show(ui, |ui| {
            let mode = &mut window.mode;   
            ui.label("Mode:");
            egui::ComboBox::from_id_salt("mode_box")
                .selected_text(window_mode_str(&mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(mode, WindowMode::Fullscreen(MonitorSelection::Current), "fullsceen");
                    ui.selectable_value(mode, WindowMode::BorderlessFullscreen(MonitorSelection::Current), "borderless");
                    ui.selectable_value(mode, WindowMode::Windowed, "windowed");
                });
            ui.end_row();
            ui.end_row();
            ui.label("Inputs");
            ui.end_row();
            ui.label("Forward");
            mapping_ui(ui, Either::Left(KeyCode::KeyW), &mut keys, &mut mouse, &mut state, 0);
            ui.end_row();
            ui.label("Backwards");
            mapping_ui(ui, Either::Left(KeyCode::KeyS), &mut keys, &mut mouse, &mut state, 1);
            ui.end_row();
            ui.label("Left");
            mapping_ui(ui, Either::Left(KeyCode::KeyA), &mut keys, &mut mouse, &mut state, 2);
            ui.end_row();
            ui.label("Right");
            mapping_ui(ui, Either::Left(KeyCode::KeyD), &mut keys, &mut mouse, &mut state, 3);
            ui.end_row();
            

        });
    });
}

pub fn setup_ui(
    mut commands: Commands
) {
    commands.insert_resource(DebugUiState{
        sun_position: 0.5,
        map_displayed: false
    });
    commands.spawn(
        PerfUiAllEntries::default()
    );
    commands.insert_resource(UiState {
        selected_input_mapping: None,
        mappings: HashMap::new()
    });
}