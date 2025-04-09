use bevy::prelude::*;
use bevy_atmosphere::{model::AtmosphereModel, plugin::AtmospherePlugin, prelude::{AtmosphereMut, Nishita}};

use crate::ui::UIState;


pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            AtmospherePlugin
        ).add_systems(Update, daylight)
        .add_systems(Startup, setup_sky);
    }
}

// updates the position of the sun and the atmosphere sim when the ui says to change, this could later actually use time with a small modification
pub fn daylight(
    mut atmosphere: AtmosphereMut<Nishita>,
    ui_state: Res<UIState>,
    mut query: Single<(&mut Transform, &mut DirectionalLight), With<Sun>>,
) {
    if ui_state.is_changed() {
        let mut pos = atmosphere.sun_position;
        pos.y = ui_state.sun_position.sin();
        pos.z = ui_state.sun_position.cos();
        atmosphere.sun_position = pos;
        query.0.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
        query.1.illuminance = pos.y.max(0.0).powf(2.0) * 10000.0;
    }
}
// marker component that defines which light is the sun
#[derive(Component)]
pub struct Sun;

pub fn setup_sky(
    mut commands: Commands
) {
    // add the atmosphere model
    commands.insert_resource(AtmosphereModel::default());
    // and spawn the sun marker component with a strong light and a position
    commands.spawn((
        Sun,
        DirectionalLight{
            illuminance: 11127.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-0.340)),
    ));
}