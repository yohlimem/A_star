use std::hash::{Hash, Hasher};

use nannou::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Square {
    pub position: Vec2,
    pub solid: bool,
    pub index: (usize, usize),
    pub potential: f32,
}

impl Eq for Square {}
impl Hash for Square {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.solid.hash(state);
        self.index.hash(state);
    }
}
