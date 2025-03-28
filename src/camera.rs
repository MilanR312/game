use std::f32::consts::FRAC_PI_2;

use bevy::{input::{keyboard::KeyboardInput, mouse::AccumulatedMouseMotion}, pbr::ScreenSpaceReflections, prelude::*, window::CursorGrabMode};
use bevy_atmosphere::plugin::AtmosphereCamera;
use bevy_rapier3d::rapier::data;
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransformPlugin};





pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LookTransformPlugin,
            UnrealCameraPlugin::default()
        ))
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (rotate_camera, move_camera, handle_others));
    }
}

#[derive(Component)]
pub enum Camera{
    Flying{
        v_yaw: f32,
        v_pitch: f32
    }
}
#[derive(Component)]
pub struct CameraData{
    is_mouse_grabbed: bool
}

pub fn rotate_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut Camera, &CameraData)>
) {
    let delta = accumulated_mouse_motion.delta * time.delta_secs();
    const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
    const MAX_V: f32 = 1.0;
    const REDUCE: f32 = 2.0;
    let (mut transform,mut camera, data) = camera.into_inner();
    match &mut *camera {
        Camera::Flying{v_pitch, v_yaw} if data.is_mouse_grabbed => {
            // in flying mode we use a velocity based controller so it gives smoother movements
            let v_delta_yaw = -delta.x * 0.05;
            let v_delta_pitch = -delta.y * 0.05;
            *v_pitch = (v_delta_pitch + *v_pitch).clamp(-MAX_V, MAX_V);
            *v_yaw = (v_delta_yaw + *v_yaw).clamp(-MAX_V, MAX_V);

            let delta_yaw = *v_yaw * time.delta_secs();
            let delta_pitch = *v_pitch * time.delta_secs();

            let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
            let yaw = yaw + delta_yaw;
            let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
            *v_pitch -= v_pitch.signum() * REDUCE.clamp(0.0, v_pitch.abs()) * time.delta_secs();
            *v_yaw -= v_yaw.signum() * REDUCE.clamp(0.0, v_yaw.abs()) * time.delta_secs();
        },
        _ => {}
    }
}
pub fn move_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut Camera, &CameraData)>
) {
    let (mut transform, mut camera, data) = camera.into_inner();
    match &mut *camera {
        Camera::Flying { .. } if data.is_mouse_grabbed => {
            let mut move_delta = Vec3::ZERO;
            // keycodes make it work on querty and azerty
            if keys.pressed(KeyCode::KeyW) {
                move_delta.z -= 1.0;
            }
            if keys.pressed(KeyCode::KeyS) {
                move_delta.z += 1.0;
            }
            if keys.pressed(KeyCode::KeyD) {
                move_delta.x += 1.0;
            }
            if keys.pressed(KeyCode::KeyA) {
                move_delta.x -= 1.0;
            }
            if keys.pressed(KeyCode::Space) {
                move_delta.y += 1.0;
            }
            if keys.pressed(KeyCode::ControlLeft) {
                move_delta.y -= 1.0;
            }
            if keys.pressed(KeyCode::ShiftLeft) {
                move_delta *= 3.0;
            }
            move_delta *= time.delta_secs() * 10.0;
            let rotation = transform.rotation;
            transform.translation += rotation * move_delta;
        },
        _ => {}
    }
}
pub fn handle_others(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(&mut Camera, &mut CameraData)>,
    mut window: Single<&mut Window>
) {
    let (cam, mut data) = camera.into_inner();
    match &*cam {
        Camera::Flying { .. } => {
            if keys.pressed(KeyCode::Enter) {
                data.is_mouse_grabbed = true;
                window.cursor_options.grab_mode = CursorGrabMode::Confined;
                window.cursor_options.visible = false;
            }
            if keys.pressed(KeyCode::Escape) {
                data.is_mouse_grabbed = false;
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
            }
        }
    }
}

pub fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(),
        Camera::Flying{ v_pitch: 0.0, v_yaw: 0.0 },

        Msaa::Off,
        //DepthPrepass,
        //DeferredPrepass,
        ScreenSpaceReflections::default(),
        CameraData {
            is_mouse_grabbed: true
        },

        //UnrealCameraBundle::new(
        //    UnrealCameraController::default(), 
        //    Vec3::new(50.0, 30.0, -10.0), Vec3::new(60.0, 3.0, 30.0), Vec3::Y)
        Transform::from_xyz(50.0, 30.0, -10.0).looking_at(Vec3::new(60.0, 3.0, 30.0), Vec3::Y)
    ));
}