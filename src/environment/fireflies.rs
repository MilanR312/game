use std::f32::consts::PI;

use bevy::{math::vec3, prelude::*};
use bevy_hanabi::prelude::*;


fn create_firefly_effect(
    mut effects: ResMut<Assets<EffectAsset>>
) -> Handle<EffectAsset> {
  // Define a color gradient from red to transparent black
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.9, 0.9, 0.2, 0.0)); // Fade in
    color_gradient.add_key(0.2, Vec4::new(1.0, 1.0, 0.5, 0.8)); // Bright yellow
    color_gradient.add_key(0.8, Vec4::new(0.9, 0.8, 0.3, 0.6)); // Warm glow
    color_gradient.add_key(1.0, Vec4::new(0.8, 0.7, 0.2, 0.0)); // Fade out

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.01));
    size_gradient.add_key(0.5, Vec3::splat(0.05)); // Gentle pulse
    size_gradient.add_key(1.0, Vec3::splat(0.02));

    let writer = ExprWriter::new();

    

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Surface
    };

    let radius = writer.lit(0.2).uniform(writer.lit(0.4));
    let theta = writer.lit(0.0).uniform(writer.lit(PI * 2.0));
    let phi = writer.lit(-PI).uniform(writer.lit(PI));

    let init_angles = SetAttributeModifier::new(Attribute::F32X3_0, 
        radius.vec3(theta, phi).expr()
    );
    let set_angles = SetAttributeModifier::new(Attribute::F32X3_0, 
        writer.attr(Attribute::F32X3_0).x().vec3(
            writer.attr(Attribute::F32X3_0).y() + (writer.lit(0.05) * writer.time()).sin() * writer.lit(0.02), 
            writer.attr(Attribute::F32X3_0).z() + (writer.lit(0.05) * writer.time()).sin() * writer.lit(0.02), 
        ).expr()
    );

    let velocity = SetAttributeModifier::new(Attribute::VELOCITY, 
        (writer.attr(Attribute::F32X3_0).x() * writer.attr(Attribute::F32X3_0).y().sin() * writer.attr(Attribute::F32X3_0).z().cos())
        .vec3(
            writer.attr(Attribute::F32X3_0).x() * writer.attr(Attribute::F32X3_0).y().sin() * writer.attr(Attribute::F32X3_0).z().sin(),
            writer.attr(Attribute::F32X3_0).x() * writer.attr(Attribute::F32X3_0).y().cos() 
        ).expr()
    );

    let age = writer.lit(0.0).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let average_lifetime = 10.0;
    let lifetime = writer.lit(average_lifetime * 0.8).uniform(writer.lit(average_lifetime * 1.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);


    let asset = EffectAsset::new(
        128,
        SpawnerSettings::rate((4.).into()), 
        writer.finish()
    ).with_name("fireflies")
        .init(init_pos)
        .init(init_angles)
        .init(init_age)
        .init(init_lifetime)
        .render(ColorOverLifetimeModifier::new(color_gradient))
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false
        })
        .render(OrientModifier {
            mode: OrientMode::FaceCameraPosition,
            rotation: None
        })
        .update(set_angles)
        .update(velocity)
        ;

    effects.add(asset)
}

pub fn setup_fireflies(
    mut commands: Commands,
    effects: ResMut<Assets<EffectAsset>>
) {
    let positions = [
        vec3(81.85775, 4.7008815, 23.67252),
        vec3(81.77121, 4.6228757, 25.904512),
        vec3(80.03963, 4.8517885, 28.639233),
        vec3(78.249825, 5.0461464, 27.238514),
        vec3(80.09793, 4.4813976, 25.126257),
    ];
    let effect = create_firefly_effect(effects);
    for position in positions {
        commands.spawn((
            ParticleEffect::new(effect.clone()),
            Transform::from_translation(position)
        ));
    }
}