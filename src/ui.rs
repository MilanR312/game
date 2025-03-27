use bevy::{diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, prelude::*, render::diagnostic::RenderDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use iyes_perf_ui::prelude::PerfUiAllEntries;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins((
                FrameTimeDiagnosticsPlugin,
                EntityCountDiagnosticsPlugin,
                SystemInformationDiagnosticsPlugin,
                RenderDiagnosticsPlugin,
            ))
            .add_systems(Update, ui)
            .add_systems(Startup, setup_ui);
    }
}


#[derive(Resource, Default)]
pub struct UIState{
    pub sun_position: f32
}
pub fn ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UIState>,
) {
    egui::Window::new("Test").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut ui_state.sun_position, 0.0 ..= 6.24))
    });
}
pub fn setup_ui(
    mut commands: Commands
) {
    commands.insert_resource(UIState{
        sun_position: 0.5
    });
    commands.spawn(
        PerfUiAllEntries::default()
    );
}