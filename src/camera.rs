use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::{bloom::Bloom, experimental::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing}, tonemapping::Tonemapping}, input::{keyboard::KeyboardInput, mouse::AccumulatedMouseMotion}, pbr::{ScreenSpaceReflections, VolumetricFog}, prelude::*, window::CursorGrabMode};
use bevy_atmosphere::plugin::AtmosphereCamera;
use bevy_rapier3d::rapier::data;
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransformPlugin};





pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        /*app.add_plugins((
            LookTransformPlugin,
            UnrealCameraPlugin::default() We used to use a prebuilt package for movement now switched to custom one
        ))*/
        app.add_plugins(TemporalAntiAliasPlugin) // allows use of TAA to reduce noise whilst we figure out mipmaps in the custom material
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (rotate_camera, move_camera, handle_others));
    }
}

// camera state currently only allows flying camera smoothed with velocity based inputs, this could also make use of rapiers physics engine in the future
#[derive(Component)]
pub enum CameraType{
    Flying{
        v_yaw: f32,
        v_pitch: f32
    }
}

// state that tracks wether or not the cursor is grabbed
#[derive(Component)]
pub struct CameraData{
    is_mouse_grabbed: bool
}

// rotate the camera based on the mouse motion in the past frame
pub fn rotate_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut CameraType, &CameraData)>
) {
    let delta = accumulated_mouse_motion.delta * time.delta_secs();
    const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
    const MAX_V: f32 = 1.0;
    const REDUCE: f32 = 2.0;
    let (mut transform,mut camera, data) = camera.into_inner();
    match &mut *camera {
        CameraType::Flying{v_pitch, v_yaw} if data.is_mouse_grabbed => {
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
// move the camera based on key inputs
pub fn move_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut CameraType, &CameraData)>
) {
    let (mut transform, mut camera, data) = camera.into_inner();
    match &mut *camera {
        CameraType::Flying { .. } if data.is_mouse_grabbed => {
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
            transform.translation += rotation * move_delta; // rotation transforms it from local space to camera orientation so z is forwards
        },
        _ => {}
    }
}
// handle other keys like switching between captured inputs and later maybe other camera modes
pub fn handle_others(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(&mut CameraType, &mut CameraData, &Transform)>,
    mut window: Single<&mut Window>
) {
    let (cam, mut data, location) = camera.into_inner();
    match &*cam {
        CameraType::Flying { .. } => {
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
    if keys.pressed(KeyCode::KeyM) {
        println!("at: {location:?}");
    }
}

pub fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(), // enable atmosphere rendering
        Camera {
            hdr: true,
            ..Default::default()
        },
        CameraType::Flying{ v_pitch: 0.0, v_yaw: 0.0 }, // start the camera as a flying cam

        //Msaa::Off, // disable MSAA
        //TemporalAntiAliasing::default(), // but use TAA
        //DepthPrepass,
        //DeferredPrepass,
        //ScreenSpaceReflections::default(),
        Bloom {
            ..Default::default()
        },
        Tonemapping::TonyMcMapface,
        CameraData {
            is_mouse_grabbed: true
        }, // grab the mouse at the start
        VolumetricFog {
            ..Default::default()
        },
        Transform::from_xyz(50.0, 30.0, -10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y) // and give it a default position
    ));
}