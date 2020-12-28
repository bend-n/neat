use crate::activation::ActivationKind;
use crate::aggregations::Aggregation;
use crate::node::NodeKind;
use rand::random;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct NodeGene {
    pub kind: NodeKind,
    pub aggregation: Aggregation,
    pub activation: ActivationKind,
    pub bias: f64,
}

impl NodeGene {
    pub fn new(kind: NodeKind) -> Self {
        let aggregation = random();
        let activation = match kind {
            NodeKind::Input => ActivationKind::Input,
            _ => random(),
        };
        let bias: f64 = match kind {
            NodeKind::Input => 0.,
            _ => random::<f64>() * 2. - 1.,
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

impl Hash for NodeGene {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.aggregation.hash(state);
        self.activation.hash(state);
        self.bias.to_bits().hash(state);
    }
}
