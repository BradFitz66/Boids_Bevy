use crate::code::traits::DebugDisplay;
use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugShapes;

#[derive(Component, Default, Clone, Debug)]
pub struct Boid {
    pub neighbours: Vec<Entity>,
}

#[derive(Component)]
pub struct Kinematic {
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub last_position: Vec3,
}

impl Kinematic {
    pub fn new(velocity: Vec3) -> Self {
        Kinematic {
            velocity,
            acceleration: Vec3::new(0., 0., 0.),
            last_position: Vec3::new(0., 0., 0.),
        }
    }
}
impl DebugDisplay<Kinematic> for Kinematic {
    fn debug_display(&self, shapes: &mut ResMut<DebugShapes>) {
        shapes
            .line()
            .start(self.last_position)
            .end(self.last_position + (self.velocity.normalize() * 30.))
            .color(Color::GREEN);
    }
}

#[derive(Component)]
pub struct KinematicConstraint {
    pub max_speed: f32,
    pub max_force: f32,
}
