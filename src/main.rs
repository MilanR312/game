
use std::{fs::File, io::Read, path::Path};

use bevy::{asset::{io::Reader, AssetLoader, LoadContext, LoadState, RenderAssetUsages}, core_pipeline::{experimental::taa::TemporalAntiAliasPlugin, prepass::{DeferredPrepass, DepthPrepass}}, diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, gltf::GltfMesh, image::ImageLoader, pbr::{experimental::meshlet::MeshletPlugin, wireframe::{Wireframe, WireframeConfig, WireframePlugin}, ExtendedMaterial, MaterialExtension, ScreenSpaceReflections}, prelude::*, render::{batching::no_gpu_preprocessing::BatchedInstanceBuffer, diagnostic::RenderDiagnosticsPlugin, mesh::{Indices, PlaneMeshBuilder, PrimitiveTopology}, render_resource::AsBindGroup, settings::{RenderCreation, WgpuFeatures, WgpuSettings}, RenderPlugin}, scene::{InstanceId, SceneInstance}, utils::hashbrown::HashMap};
use bevy_atmosphere::{model::AtmosphereModel, plugin::{AtmosphereCamera, AtmospherePlugin}, prelude::{AtmosphereMut, Nishita}};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_obj::ObjPlugin;
use bevy_tweening::TweeningPlugin;
use bevy_water::{material::{StandardWaterMaterial, WaterMaterial}, WaterPlugin, WaterSettings};
use camera::CameraPlugin;
use environment::EnvironmentPlugin;
use iyes_perf_ui::{entries::{PerfUiFixedTimeEntries, PerfUiFramerateEntries, PerfUiSystemEntries}, prelude::{PerfUiAllEntries, PerfUiDefaultEntries}, PerfUiPlugin};
use material::{TerrainMaterialExtension, TestMaterialExtension};
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
mod pathfinding;
mod material;

fn main() -> eyre::Result<()> {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings.features.set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);
    App::new().add_plugins(DefaultPlugins.set(
        RenderPlugin{
            render_creation: wgpu_settings.into(),
            synchronous_pipeline_compilation: false
        }
    )/*.set(ImagePlugin::default_nearest())*/)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default()) // load the physics sym for the birds
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>
            >::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TestMaterialExtension>
        >::default()) // register the custom material as an extension to the base material
        .add_plugins(TweeningPlugin)
        .add_plugins(ObjPlugin) // load obj file loading
        .add_plugins(UiPlugin) // load our ui
        .add_plugins(SkyPlugin) // load sky related assets
        .add_plugins(CameraPlugin) // load the camera
        .add_plugins(EnvironmentPlugin) // load the environment
        .run();
    Ok(())
}



