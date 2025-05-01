use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{core_pipeline::{bloom::Bloom, dof::DepthOfField, experimental::taa::{TemporalAntiAliasPlugin, TemporalAntiAliasing}, fxaa::Fxaa, motion_blur::MotionBlur, prepass::DepthPrepass, tonemapping::Tonemapping}, input::{keyboard::KeyboardInput, mouse::AccumulatedMouseMotion}, pbr::{FogVolume, ScreenSpaceAmbientOcclusion, ScreenSpaceReflections, VolumetricFog, VolumetricFogSettings}, prelude::*, render::camera::MipBias, text::cosmic_text::ttf_parser::loca, window::CursorGrabMode};
use bevy_atmosphere::plugin::AtmosphereCamera;
use bevy_rapier3d::prelude::*;
use bevy_tweening::{lens::{TransformPositionLens, TransformRotationLens}, Animator, Lens, Tween};
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransformPlugin};

use crate::environment::{map::MapData, waypoints::Route};


struct TransformLens {
    start: Transform,
    end: Transform,
}

impl Lens<Transform> for TransformLens {
    fn lerp(&mut self, target: &mut dyn bevy_tweening::Targetable<Transform>, ratio: f32) {
        let mut position = TransformPositionLens {
            start: self.start.translation,
            end: self.end.translation
        };
        let mut rotation = TransformRotationLens {
            start: self.start.rotation,
            end: self.end.rotation
        };
        position.lerp(target, ratio);
        rotation.lerp(target, ratio);
    }
}

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
#[derive(Component, Clone, Debug)]
pub enum CameraType{
    Flying{
        v_yaw: f32,
        v_pitch: f32
    },
    MapMode {
        prev_mode: Box<CameraType>,
        prev_position: Transform,
        mouse_position: Vec2,
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
    camera: Single<(&mut Transform, &mut CameraType, &CameraData)>,
    mapdata: Res<MapData>
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
        CameraType::MapMode { mouse_position, .. } => {
            let mapsize = mapdata.size;
            *mouse_position += accumulated_mouse_motion.delta;
            *mouse_position = mouse_position.clamp(Vec2::splat(- mapsize / 2.0), Vec2::splat(mapsize / 2.0));
        }
        _ => {}
    }
}
// move the camera based on key inputs
pub fn move_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut CameraType, &CameraData)>,
    mapdata: Res<MapData>
) {
    let (mut transform, mut camera, data) = camera.into_inner();
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
    move_delta *= time.delta_secs() * 3.0;
    match &mut *camera {
        CameraType::Flying { .. } if data.is_mouse_grabbed => {
            let rotation = transform.rotation;
            transform.translation += rotation * move_delta; // rotation transforms it from local space to camera orientation so z is forwards
        },
        CameraType::MapMode { prev_position, .. } => {
            let center_of_world = mapdata.size / 2.0;
            let rotation = transform.rotation;
            transform.translation += rotation * move_delta.with_z(0.0);
            //*transform = get_map_camera_position(*prev_position, center_of_world);
        }
        _ => {}
    }
}
pub fn get_map_camera_position(player_pos: Transform, height: f32) -> Transform {
    Transform::from_translation(player_pos.translation.with_y(height))
    .looking_at(player_pos.translation.with_y(0.0), Vec3::X)
}
// handle other keys like switching between captured inputs and later maybe other camera modes
pub fn handle_others(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&mut CameraType, &mut CameraData, &mut Transform, &mut Animator<Transform>, &GlobalTransform, &Camera)>,
    mut window: Single<&mut Window>,
    mapdata: Res<MapData>,
    rapier_context: Single<RapierContext>,
    route: Single<(&mut Route)>,
) {
    let (mut cam, mut data, mut location, mut animator, global_transform, cam_inner) = camera.into_inner();
    let new_mode = match &*cam {
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
            if keys.just_pressed(KeyCode::KeyM) {
                let center_of_map = mapdata.size / 2.0;
                animator.set_tweenable(Tween::new(
                    EaseFunction::QuadraticInOut, 
                    Duration::from_secs(3), 
                    TransformLens {
                        start: *location,
                        end: get_map_camera_position(*location, center_of_map)
                    }
                ));

                Some(CameraType::MapMode{
                    prev_mode: Box::new(cam.clone()),
                    prev_position: *location,
                    mouse_position: Vec2::splat(center_of_map),
                })
            } else {
                None
            }
        },
        CameraType::MapMode{ prev_mode, prev_position , .. } => {
            let mut route = route.into_inner();

            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = true;
            if mouse.just_pressed(MouseButton::Left) {
                let cursor_position = window.cursor_position().unwrap();
                let cursor_position = cam_inner.viewport_to_world(&*global_transform, cursor_position).unwrap();
                let dir = cursor_position.direction.as_vec3();
                let ctx = rapier_context.into_inner();
                let hit = ctx.query_pipeline.cast_ray(
                    &ctx.colliders, 
                    &ctx.rigidbody_set, 
                    location.translation, 
                    dir, 
                    f32::MAX, 
                    false, 
                    QueryFilter::only_fixed()
                );
                if let Some((e, dist)) = hit {
                    let location = location.translation + dist * dir;
                    route.destination = Some(location);
                }
            }
            if mouse.just_pressed(MouseButton::Right){
                route.destination = None;
            }
            if keys.just_pressed(KeyCode::KeyM) {
                animator.set_tweenable(Tween::new(
                    EaseFunction::QuadraticInOut, 
                    Duration::from_secs(3), 
                    TransformLens {
                        start: *location,
                        end: *prev_position
                    }
                ));
                Some(*prev_mode.clone())
            } else {
                None
            }
        }
    };
    if let Some(new_mode) = new_mode {
        *cam = new_mode;
    };
    if keys.just_pressed(KeyCode::KeyP) {
        println!("vec3({}, {}, {}),", location.translation.x, location.translation.y, location.translation.z);
    }
}

pub fn spawn_camera(
    mut commands: Commands
) {
    let initial_pos = Transform::from_xyz(50.0, 30.0, -10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(), // enable atmosphere rendering
        Camera {
            hdr: true,
            ..Default::default()
        },
        CameraType::Flying{ v_pitch: 0.0, v_yaw: 0.0 }, // start the camera as a flying cam

        //Msaa::Off, // disable MSAA
        //ScreenSpaceReflections::default(),
        Msaa::Off,
        //MotionBlur::default(),
        //TemporalAntiAliasing::default(),
        Bloom {
            ..Default::default()
        },
        Tonemapping::AcesFitted,
        DepthPrepass,
        //DepthOfField::default(),
        ScreenSpaceAmbientOcclusion::default(),
        CameraData {
            is_mouse_grabbed: true
        }, // grab the mouse at the start
        VolumetricFog {
            ambient_intensity: 0.0,
            ..Default::default()
        },
        Animator::new(Tween::new(EaseFunction::Linear, Duration::from_millis(1), TransformLens{
            start: initial_pos,
            end: initial_pos
        })),
        initial_pos
    ));
    /*commands.spawn((
        FogVolume{
            absorption: 0.05,
            ..Default::default()
        },
        Transform::from_xyz(50.0, 0.0, 50.0)
            .with_scale(Vec3::splat(100.0))
    ));*/
}