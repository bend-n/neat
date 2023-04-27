use crate::genome::connection::ConnectionGene;
use nanoserde::{DeBin, SerBin};

#[derive(Debug, DeBin, SerBin)]
pub struct Connection {
    pub from: u32,
    pub to: u32,
    pub weight: f64,
}

impl From<&ConnectionGene> for Connection {
    fn from(g: &ConnectionGene) -> Self {
        Connection {
            from: g.from,
            to: g.to,
            weight: g.weight,
        }
    }
}
