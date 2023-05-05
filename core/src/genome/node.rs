use crate::activation::ActivationKind;
use crate::aggregations::Aggregation;
use crate::mutations::rand;
use crate::node::EnumConversion;
use crate::node::NodeKind;
use godot::prelude::utilities::randf;
use godot::prelude::*;

#[derive(Debug, Clone, GodotClass)]
pub struct NodeGene {
    pub kind: NodeKind,
    pub aggregation: Aggregation,
    pub activation: ActivationKind,
    pub bias: f64,
}

#[godot_api]
impl NodeGene {
    #[func]
    fn get_kind(&self) -> u8 {
        self.kind.clone().to()
    }
    #[func]
    fn get_aggregation(&self) -> u8 {
        self.aggregation.clone().to()
    }
    #[func]
    fn get_activation(&self) -> u8 {
        self.activation.clone().to()
    }
    #[func]
    fn get_bias(&self) -> f64 {
        self.bias
    }
}

impl NodeGene {
    pub fn new(kind: NodeKind) -> Self {
        let activation = match kind {
            NodeKind::Input => ActivationKind::Input,
            _ => rand(),
        };
        let bias: f64 = match kind {
            NodeKind::Input => 0.,
            _ => randf() * 2.9 - 1.0,
        };

        NodeGene {
            aggregation: rand(),
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
