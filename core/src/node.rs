use crate::activation::ActivationKind;
use crate::aggregations::Aggregation;
use crate::genome::node::NodeGene;
use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, PartialEq, DeBin, SerBin)]
pub enum NodeKind {
    Input,
    Hidden,
    Output,
    Constant,
}

#[derive(Debug, DeBin, SerBin)]
pub struct Node {
    pub kind: NodeKind,
    pub aggregation: Aggregation,
    pub activation: ActivationKind,
    pub bias: f64,
    pub value: Option<f64>,
}

impl From<&NodeGene> for Node {
    fn from(g: &NodeGene) -> Self {
        Node {
            kind: g.kind.clone(),
            activation: g.activation.clone(),
            bias: g.bias,
            value: None,
            aggregation: g.aggregation.clone(),
        }
    }
}
