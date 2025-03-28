use bevy::{app::{Plugin, Startup, Update}, ecs::schedule::IntoSystemConfigs, pbr::DefaultOpaqueRendererMethod};
use bevy_water::{WaterPlugin, WaterSettings};
use birds::{bird_boids, calculate_area_speed, spawn_birds};
use map::load_map;
use trees::spawn_trees;
use water::{spawn_ocean, spawn_rivers};

mod trees;
mod map;
mod water;
mod birds;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(WaterSettings {
                amplitude: 0.2,
                height: 1.9,
                ..Default::default()
            })
            .insert_resource(DefaultOpaqueRendererMethod::deferred())
            .add_plugins(WaterPlugin)
            .add_systems(Startup, (
                load_map,
                spawn_ocean.after(load_map), // all these functions require the heightmap that the load_map registers
                spawn_trees.after(load_map),
                //spawn_rivers.after(load_map),
                //spawn_birds.after(load_map)
            ))
            //.add_systems(Update, (calculate_area_speed, bird_boids).chain())
        ;
    }
}