use crate::genome::connection::ConnectionGene;
use godot::prelude::*;
use nanoserde::{DeBin, SerBin};

#[derive(Debug, DeBin, SerBin, GodotClass, Clone)]
#[class(base=RefCounted)]
pub struct Connection {
    #[export(get, set)]
    pub from: u32,
    #[export(get, set)]
    pub to: u32,
    #[export(get, set)]
    pub weight: f64,
}

#[godot_api]
impl Connection {}

impl From<&ConnectionGene> for Connection {
    fn from(g: &ConnectionGene) -> Self {
        Connection {
            from: g.from,
            to: g.to,
            weight: g.weight,
        }
    }
}
