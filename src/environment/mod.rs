use bevy::{app::{Plugin, Startup, Update,PostUpdate}, ecs::schedule::IntoSystemConfigs, gizmos::AppGizmoBuilder, pbr::{experimental::meshlet::MeshletPlugin, DefaultOpaqueRendererMethod}};
use bevy_asset_loader::asset_collection::AssetCollectionApp;
use bevy_hanabi::HanabiPlugin;
use bevy_water::{WaterPlugin, WaterSettings};
use birds::BirdPlugin;
use foxes::FoxPlugin;
use sealife::SeaLifePlugin;
use map::{load_map, Map};
use trees::spawn_trees;
use water::{spawn_ocean, spawn_rivers};
use bevy::prelude::*;
use birds::BirdRules;
use waypoints::WayPointMaterial;
use crate::environment::fire::FirePlugin;

pub mod fire;
mod trees;
pub mod map;
mod water;
mod birds;
pub mod waypoints;
mod audio;
mod fireflies;
mod grass;
mod wheat;
mod pois;
mod sealife;
pub mod foxes;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(WaterSettings { // setup the water plugin
                amplitude: 0.2,
                height: 1.9,
                ..Default::default()
            })
            .insert_resource(BirdRules::default())
            .init_collection::<Map>()
            .add_plugins(WaterPlugin)
            .add_plugins(HanabiPlugin)
            .add_plugins(grass::GrassPlugin)
            .add_plugins(wheat::WheatPlugin)
            //.add_plugins(FoxPlugin)
            //.add_plugins(BirdPlugin)
            //.add_plugins(SeaLifePlugin)
            //.add_plugins(FirePlugin)
            .add_plugins(MaterialPlugin::<WayPointMaterial>{
                prepass_enabled: false,
                ..Default::default()
            })
            /*.add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, WayPointMaterial>
                >{
                    prepass_enabled: false,
                    ..Default::default()
                })*/
            .init_gizmo_group::<waypoints::InWorldPathGizmos>()
            .add_systems(Startup, (
                load_map,
                waypoints::spawn_waypoints.after(load_map),
                pois::spawn.after(load_map),
                spawn_ocean.after(load_map),
                //spawn_trees.after(load_map),
                spawn_rivers.after(load_map),
                //fireflies::setup_fireflies,
            ))                     
            .add_systems(Update, (
                waypoints::draw_path_v2, 
                waypoints::calculate_path,
                trees::set_visibility_and_merge_meshes,
                map::generate_chunks,
                map::insert_colliders,
            ));
    }
}
