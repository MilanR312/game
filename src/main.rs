
use std::{fs::File, io::Read, path::Path};

use bevy::{asset::{io::Reader, AssetLoader, LoadContext, LoadState, RenderAssetUsages}, core_pipeline::prepass::{DeferredPrepass, DepthPrepass}, diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, gltf::GltfMesh, image::ImageLoader, pbr::{experimental::meshlet::MeshletPlugin, wireframe::{Wireframe, WireframeConfig, WireframePlugin}, ExtendedMaterial, MaterialExtension, ScreenSpaceReflections}, prelude::*, render::{batching::no_gpu_preprocessing::BatchedInstanceBuffer, diagnostic::RenderDiagnosticsPlugin, mesh::{Indices, PlaneMeshBuilder, PrimitiveTopology}, render_resource::AsBindGroup, settings::{RenderCreation, WgpuFeatures, WgpuSettings}, RenderPlugin}, scene::{InstanceId, SceneInstance}, utils::hashbrown::HashMap};
use bevy_atmosphere::{model::AtmosphereModel, plugin::{AtmosphereCamera, AtmospherePlugin}, prelude::{AtmosphereMut, Nishita}};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_obj::ObjPlugin;
use bevy_water::{material::{StandardWaterMaterial, WaterMaterial}, WaterPlugin, WaterSettings};
use camera::CameraPlugin;
use environment::EnvironmentPlugin;
use iyes_perf_ui::{entries::{PerfUiFixedTimeEntries, PerfUiFramerateEntries, PerfUiSystemEntries}, prelude::{PerfUiAllEntries, PerfUiDefaultEntries}, PerfUiPlugin};
use noise::NoiseFn;
use rand::{rng, rngs::ThreadRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use sky::SkyPlugin;
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransform, LookTransformPlugin};
use spade::{DelaunayTriangulation, Triangulation};

use bevy_rapier3d::prelude::*;
use thiserror::Error;
use ui::UiPlugin;


mod mask;
mod ui;
mod sky;
mod environment;
mod camera;


fn main() -> eyre::Result<()> {
    App::new().add_plugins(DefaultPlugins)

        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(ObjPlugin) // load obj file loading
        .add_plugins(UiPlugin) // load our ui
        .add_plugins(SkyPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(EnvironmentPlugin)

        .run();
    Ok(())
}



