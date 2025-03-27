use bevy::prelude::*;
use bevy_atmosphere::plugin::AtmosphereCamera;
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransformPlugin};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LookTransformPlugin,
            UnrealCameraPlugin::default()
        ))
        .add_systems(Startup, spawn_camera);
    }
}

pub fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(),
        //Msaa::Off,
        //DepthPrepass,
        //DeferredPrepass,
        //ScreenSpaceReflections::default(),
        UnrealCameraBundle::new(
            UnrealCameraController::default(), 
            Vec3::new(50.0, 30.0, -10.0), Vec3::new(60.0, 3.0, 30.0), Vec3::Y)
        //Transform::from_xyz(-5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y)
    ));
}