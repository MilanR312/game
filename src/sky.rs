use std::f32::consts::PI;

use bevy::{pbr::{FogVolume, VolumetricLight}, prelude::*};
use bevy_atmosphere::{model::AtmosphereModel, plugin::AtmospherePlugin, prelude::{AtmosphereMut, Nishita}};

use crate::ui::DebugUiState;


pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(
            AtmospherePlugin
        )
        .add_systems(Update, daylight)
        .add_systems(Startup, (setup_sky, spawn_clouds));
    }
}

// updates the position of the sun and the atmosphere sim when the ui says to change, this could later actually use time with a small modification
pub fn daylight(
    mut atmosphere: AtmosphereMut<Nishita>,
    ui_state: Res<DebugUiState>,
    mut query: Single<(&mut Transform, &mut DirectionalLight), (With<Sun>, Without<Moon>)>,
    mut query2: Single<(&mut Transform, &mut DirectionalLight), (With<Moon>, Without<Sun>)>,
) {
    if ui_state.is_changed() {
        let mut pos = atmosphere.sun_position;
        pos.y = ui_state.sun_position.sin();
        pos.z = ui_state.sun_position.cos();
        atmosphere.sun_position = pos;
        query.0.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
        query.1.illuminance = pos.y.max(0.0).powf(2.0) * 10000.0;


        query2.0.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z) + PI);
        query2.1.illuminance = pos.y.max(0.0).powf(2.0) * 10000.0;
    }
}
// marker component that defines which light is the sun
#[derive(Component)]
pub struct Sun;


#[derive(Component)]
pub struct Moon;

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
        VolumetricLight,
        Transform::from_rotation(Quat::from_rotation_x(-0.340)),
    ));
    commands.spawn((
        Moon,
        DirectionalLight{
            illuminance: 11127.0,
            shadows_enabled: true,
            ..Default::default()
        },
        VolumetricLight,
        Transform::from_rotation(Quat::from_rotation_x(-0.340 + PI)),
    ));
}

#[derive(Component)]
pub struct Cloud;

pub fn spawn_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        PointLight {
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.0, 0.0),
            intensity: 100.0,
            ..Default::default()
        },
        VolumetricLight,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(1.0, 0.5, 1.0)
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(1.0, 0.5, -1.0)
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));
    /*commands.spawn((
        FogVolume {
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::splat(35.0))
    ));*/
}