use std::collections::HashMap;

use ordered_float::OrderedFloat;
use pathfinding::prelude::astar;
use slotmap::{new_key_type, SecondaryMap, SlotMap, SparseSecondaryMap};


new_key_type! {
    pub struct NodeKey;
}

pub struct Graph {
    linkdata: SlotMap<NodeKey, Vec<NodeKey>>,
    positions: SecondaryMap<NodeKey, [OrderedFloat<f32>; 2]>,
}
impl Graph {
    pub fn add_position(&mut self, pos: [f32; 2]) -> NodeKey {
        let key = self.linkdata.insert(vec![]);
        self.positions.insert(key, [OrderedFloat(pos[0]), OrderedFloat(pos[1])]);
        key
    }
    pub fn add_connection(&mut self, a: NodeKey, b: NodeKey) {
        debug_assert!(!self.linkdata.get(a).unwrap().contains(&b));
        debug_assert!(!self.linkdata.get(b).unwrap().contains(&a));
        self.linkdata.get_mut(a).unwrap().push(b);
        self.linkdata.get_mut(b).unwrap().push(a);
    }
    pub fn cost(&self, a: NodeKey, b: NodeKey) -> OrderedFloat<f32> {
        let pos_a = self.positions.get(a).unwrap();
        let pos_b = self.positions.get(b).unwrap();
        let delta = ((pos_b[0] - pos_a[0]).abs(), (pos_b[1] - pos_a[1]).abs());
        let out = (delta.0 * delta.0 + delta.1 * delta.1).sqrt();
        OrderedFloat(out)
    }
    pub fn successors(&self, key: NodeKey) -> impl Iterator<Item = (NodeKey, OrderedFloat<f32>)>{
        self.linkdata.get(key).unwrap()
            .iter().copied()
            .map(move |x| (x, self.cost(key, x)))
    }
    pub fn a_star(&self, source: NodeKey, destination: NodeKey) -> Option<Vec<NodeKey>>{
        let result = astar(
            &source, 
            |p| self.successors(*p), 
            |p| self.cost(*p, destination), 
            |p| *p == destination
        )?;
        Some(result.0)
    }
}