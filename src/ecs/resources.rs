use crate::code::traits::DebugDisplay;
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_prototype_debug_lines::{DebugShapes};
use std::fmt::Debug;

#[derive(Resource, Clone)]
pub struct SparseSpatialHash<T> {
    pub cell_size: f32,
    pub grid: HashMap<IVec3, Vec<T>>,
}

impl<T: Clone> Default for SparseSpatialHash<T> {
    fn default() -> Self {
        SparseSpatialHash {
            cell_size: 100.,
            grid: HashMap::new(),
        }
    }
}

impl<T: Clone + PartialEq + Copy> SparseSpatialHash<T> {
    
    pub fn insert(&mut self, entity: T, position: Vec3) {
        let index = self.get_index(position);
        let cell = self.grid.entry(index).or_insert(Vec::new());
        cell.push(entity);
    }

    pub fn remove(&mut self, entity: T, position: Vec3) {
        let cell_index = self.get_index(position);

        let cell = self.grid.get_mut(&cell_index).unwrap();
        let index = cell.iter().position(|x| *x == entity).unwrap();

        cell.remove(index);
        //If cell is empty, remove it
        if cell.is_empty() {
            self.grid.remove(&cell_index);
        }
    }

    pub fn update(&mut self, entity: T, old_position: Vec3, new_position: Vec3) {
        let old_index = self.get_index(old_position);
        let new_index = self.get_index(new_position);

        //If the entity has moved to a new cell, remove it from the old cell and add it to the new cell. This check
        //avoids removing and adding the entity to the same cell every frame.
        if old_index != new_index {
            self.remove(entity, old_position);
            self.insert(entity, new_position);
        }
    }

    #[inline]
    pub fn get_index(&self, position: Vec3) -> IVec3 {
        //Round position to nearest cell
        let x = (position.x / self.cell_size).round() as i32;
        let y = (position.y / self.cell_size).round() as i32;
        let z = (position.z / self.cell_size).round() as i32;

        IVec3::new(x, y, z)
    }

    pub fn get_cell(&self, index: IVec3) -> Option<Vec<T>> {
        self.grid.get(&index).cloned()
    }

    //Get the neighbors of a cell
    pub fn get_neighbors(&self, position: Vec3) -> Vec<Vec<T>> {
        let mut neighbors: Vec<Vec<T>> = Vec::new();
        let index = self.get_index(position);
        //Get neighbors at top, bottom, left, and right via loop.
        for x in -1..=1 {
            for y in -1..=1 {
                let neighbor_index = IVec3::new(index.x + x, index.y + y, 0);
                let neighbor = self.get_cell(neighbor_index);
                if neighbor.is_some() {
                    neighbors.push(neighbor.unwrap());
                }
            }
        }
        neighbors
    }
}

impl<T: Debug> DebugDisplay<T> for SparseSpatialHash<T> {
    fn debug_display(&self, shapes: &mut ResMut<DebugShapes>) {
        //Loop through all cells and draw rectangles
        for (key, _value) in &self.grid {
            shapes
                .rect()
                .position(Vec3::new(
                    (key.x * (self.cell_size as i32)) as f32,
                    (key.y * (self.cell_size as i32)) as f32,
                    0.,
                ))
                .size(Vec2::new(self.cell_size, self.cell_size))
                .color(Color::rgba(1.0, 0., 0., 0.5));
        }
    }
}

#[derive(Resource, Clone)]
pub struct BoidWorld {
    pub width: f32,
    pub height: f32,
}

impl Default for BoidWorld {
    fn default() -> Self {
        BoidWorld {
            width: 1000.,
            height: 1000.,
        }
    }
}
