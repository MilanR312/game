use std::collections::HashMap;

use bevy::{ecs::component::Component, math::Vec3};
use ordered_float::OrderedFloat;
use pathfinding::prelude::astar;
use slotmap::{new_key_type, SecondaryMap, SlotMap, SparseSecondaryMap};


new_key_type! {
    pub struct NodeKey;
}


#[derive(Component, Debug)]
pub struct Graph {
    linkdata: SlotMap<NodeKey, Vec<NodeKey>>,
    positions: SecondaryMap<NodeKey, Vec3>,
}
impl Graph {
    pub fn new() -> Self {
        Self {
            linkdata: SlotMap::with_key(),
            positions: SecondaryMap::new()
        }
    }
    pub fn positions(&self) -> impl Iterator<Item = &Vec3> {
        self.positions.iter().map(|x| x.1)
    }
    pub fn add_position(&mut self, pos: Vec3) -> NodeKey {
        let key = self.linkdata.insert(vec![]);
        self.positions.insert(key, pos);
        key
    }
    pub fn add_connection(&mut self, a: NodeKey, b: NodeKey) {
        debug_assert!(!self.linkdata.get(a).unwrap().contains(&b));
        debug_assert!(!self.linkdata.get(b).unwrap().contains(&a));
        self.linkdata.get_mut(a).unwrap().push(b);
        self.linkdata.get_mut(b).unwrap().push(a);
    }
    pub fn add_line<I: IntoIterator<Item = Vec3>>(&mut self, start: NodeKey, positions: I) -> NodeKey{
        let mut prev = start;
        for pos in positions.into_iter() {
            let new = self.add_position(pos);
            self.add_connection(prev, new);
            prev = new;
        }
        prev
    }
    fn cost_inner(pos_a: Vec3, pos_b: Vec3) -> f32{
        let delta = ((pos_b[0] - pos_a[0]).abs(), (pos_b[1] - pos_a[1]).abs(), (pos_b[2] - pos_a[2]).abs());
        let out = (delta.0 * delta.0 + delta.1 * delta.1 + delta.2 * delta.2).sqrt();
        out
    }
    pub fn cost(&self, a: NodeKey, b: NodeKey) -> f32 {
        let pos_a = self.positions.get(a).unwrap();
        let pos_b = self.positions.get(b).unwrap();
        Self::cost_inner(*pos_a, *pos_b)
    }
    pub fn successors(&self, key: NodeKey) -> impl Iterator<Item = (NodeKey, OrderedFloat<f32>)>{
        self.linkdata.get(key).unwrap()
            .iter().copied()
            .map(move |x| (x, OrderedFloat(self.cost(key, x))))
    }
    pub fn a_star(&self, source: NodeKey, destination: NodeKey) -> Option<Vec<NodeKey>>{
        let result = astar(
            &source, 
            |p| self.successors(*p), 
            |p| OrderedFloat(self.cost(*p, destination)), 
            |p| *p == destination
        )?;
        Some(result.0)
    }
    pub fn a_star_positions(&self, start: Vec3, destination: Vec3) -> Option<Vec<Vec3>>{
        let closest_node_to_start = self.positions.iter()
            .min_by(|(_, a),(_, b)| {
                let dist_a = start.distance_squared(**a);
                let dist_b = start.distance_squared(**b);
                dist_a.total_cmp(&dist_b)
            })?;
        let closest_node_to_destination = self.positions.iter()
            .min_by(|(_, a),(_, b)| {
                let dist_a = destination.distance_squared(**a);
                let dist_b = destination.distance_squared(**b);
                dist_a.total_cmp(&dist_b)
            })?;
        let nodes_between = self.a_star(closest_node_to_start.0, closest_node_to_destination.0)?;
        Some(
            std::iter::once(start)
                .chain(nodes_between.into_iter().map(|key| *self.positions.get(key).expect("this should never happen")))
                .chain(std::iter::once(destination))
                .collect()
        )
    }
}