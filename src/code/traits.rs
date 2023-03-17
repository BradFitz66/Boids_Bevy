use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugShapes;

pub trait DebugDisplay<T> {
    fn debug_display(&self, shapes: &mut ResMut<DebugShapes>);
}
