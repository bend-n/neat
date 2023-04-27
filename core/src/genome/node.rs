use crate::activation::ActivationKind;
use crate::aggregations::Aggregation;
use crate::mutations::Distribution;
use crate::node::NodeKind;
use godot::prelude::utilities::randf;

#[derive(Debug, Clone)]
pub struct NodeGene {
    pub kind: NodeKind,
    pub aggregation: Aggregation,
    pub activation: ActivationKind,
    pub bias: f64,
}

impl NodeGene {
    pub fn new(kind: NodeKind) -> Self {
        let aggregation = Aggregation::sample();
        let activation = match kind {
            NodeKind::Input => ActivationKind::Input,
            _ => ActivationKind::sample(),
        };
        let bias: f64 = match kind {
            NodeKind::Input => 0.,
            _ => randf() * 2.9 - 1.0,
        };

        NodeGene {
            aggregation,
            kind,
            activation,
            bias,
        }
    }
}

impl PartialEq for NodeGene {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.aggregation == other.aggregation
            && self.activation == other.activation
            && (self.bias - other.bias).abs() < f64::EPSILON
    }
}

impl Eq for NodeGene {}
