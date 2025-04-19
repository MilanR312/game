use bevy::prelude::*;

use super::map::{Biome, BiomeCollection, MapData};

#[derive(Component)]
pub struct AudioParams{
    min: f32,
    max: f32
}

pub fn spawn_biome_audio(
    assert_server: Res<AssetServer>,
    mut commands: Commands
) {
    commands.spawn((
        AudioPlayer::new(assert_server.load("crickets.ogg")),
        Biome::Forest | Biome::Plains,
        AudioParams {
            min: 0.0,
            max: 0.5
        }
    ));
}

pub fn set_biome_audio_volume(
    sound_tracks: Query<(&AudioSink, &BiomeCollection, &AudioParams)>,
    world: Res<MapData>,
    player: Single<&Transform, With<Camera>>,
    time: Res<Time>
) {
    let player_position = player.translation.xz();
    let Some(player_biome) = world.biome_at_point(player_position.x, player_position.y) else {
        return;
    };
    let player_biome = player_biome.get_primary_biome().to_collection();
    
    for (sound_track, biomes, params) in sound_tracks.iter() {
        let old_volume = sound_track.volume();
        if !biomes.contains(player_biome) {
            sound_track.set_volume(
                (old_volume - time.delta_secs() * 0.01).clamp(params.min, params.max)
            );
        } else {
            sound_track.set_volume(
                (old_volume + time.delta_secs() * 0.01).clamp(params.min, params.max)
            );
        }
    }
}