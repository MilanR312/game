use bevy::{app::{Plugin, Startup, Update}, ecs::schedule::IntoSystemConfigs, gizmos::AppGizmoBuilder, pbr::DefaultOpaqueRendererMethod};
use bevy_hanabi::HanabiPlugin;
use bevy_water::{WaterPlugin, WaterSettings};
use birds::{bird_boids, calculate_area_speed, spawn_birds};
use map::load_map;
use trees::spawn_trees;
use water::{spawn_ocean, spawn_rivers};

mod trees;
mod map;
mod water;
mod birds;
mod waypoints;
mod audio;
mod fireflies;
mod grass;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(WaterSettings { // setup the water plugin
                amplitude: 0.2,
                height: 1.9,
                ..Default::default()
            })
            .add_plugins(WaterPlugin)
            .add_plugins(HanabiPlugin)
            .add_plugins(grass::GrassPlugin)
            .init_gizmo_group::<waypoints::PathGizmos>()
            .add_systems(Startup, (
                load_map,
                waypoints::spawn_waypoints.after(load_map),
                spawn_ocean.after(load_map), // all these functions require the heightmap that the load_map registers so it can only be ran after load_map
                spawn_trees.after(load_map), // they can still run in parallel so no chain is used
                //spawn_rivers.after(load_map),
                //spawn_birds.after(load_map),
                //fireflies::setup_fireflies
            ))
            .add_systems(Update, (waypoints::draw_path, waypoints::calculate_path))
            //.add_systems(Update, (calculate_area_speed, bird_boids).chain()) // we need to calculate the area speed first so we chain the 2 updates so they run back to back
        ;
    }
}