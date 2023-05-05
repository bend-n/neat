use crate::activation::ActivationKind;
use crate::aggregations::Aggregation;
use crate::genome::node::NodeGene;
use godot::prelude::*;
use nanoserde::{DeBin, SerBin};

pub trait EnumConversion {
    fn from(i: u8) -> Self;
    fn pick_range() -> (i64, i64) {
        (0, Self::len() as i64)
    }
    fn to(self) -> u8;
    fn len() -> u8;
}

impl EnumConversion for NodeKind {
    fn from(i: u8) -> Self {
        match i {
            0 => Self::Input,
            1 => Self::Hidden,
            2 => Self::Output,
            _ => Self::Constant,
        }
    }
    fn to(self) -> u8 {
        match self {
            Self::Input => 0,
            Self::Hidden => 1,
            Self::Output => 2,
            Self::Constant => 3,
        }
    }
    fn len() -> u8 {
        3
    }
    fn pick_range() -> (i64, i64) {
        (1, 3)
    }
}

#[derive(Debug, Clone, PartialEq, DeBin, SerBin)]
pub enum NodeKind {
    Input,
    Hidden,
    Output,
    Constant,
}

#[derive(Debug, DeBin, SerBin, GodotClass, Clone)]
pub struct NeuralNode {
    pub kind: NodeKind,
    pub aggregation: Aggregation,
    pub activation: ActivationKind,
    pub bias: f64,
    pub value: Option<f64>,
}

#[godot_api]
impl NeuralNode {
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
    /// null till i can get signals
    #[func]
    fn get_value(&self) -> Variant {
        if self.value.is_none() {
            return Variant::nil();
        }
        self.value.unwrap().to_variant()
    }
}

impl From<&NodeGene> for NeuralNode {
    fn from(g: &NodeGene) -> Self {
        NeuralNode {
            kind: g.kind.clone(),
            activation: g.activation.clone(),
            bias: g.bias,
            value: None,
            aggregation: g.aggregation.clone(),
        }
    }
}
