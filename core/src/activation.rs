use crate::mutations::Distribution;
use godot::prelude::utilities::randi_range;
use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, PartialEq, DeBin, SerBin)]
pub enum ActivationKind {
    Input,
    Tanh,
    Relu,
    Step,
    Logistic,
    Identity,
    Softsign,
    Sinusoid,
    Gaussian,
    BentIdentity,
    Bipolar,
    Inverse,
    Selu,
}

impl Distribution for ActivationKind {
    fn sample() -> Self {
        match randi_range(0, 11) {
            0 => ActivationKind::Tanh,
            1 => ActivationKind::Relu,
            2 => ActivationKind::Step,
            3 => ActivationKind::Logistic,
            4 => ActivationKind::Identity,
            5 => ActivationKind::Softsign,
            6 => ActivationKind::Sinusoid,
            7 => ActivationKind::Gaussian,
            8 => ActivationKind::BentIdentity,
            9 => ActivationKind::Bipolar,
            10 => ActivationKind::Selu,
            _ => ActivationKind::Inverse,
        }
    }
}
pub fn activate(x: f64, kind: &ActivationKind) -> f64 {
    match kind {
        ActivationKind::Tanh => x.tanh(),
        ActivationKind::Relu => {
            if x > 0. {
                x
            } else {
                0.01 * x
            }
        }
        ActivationKind::Step => {
            if x > 0. {
                1.
            } else {
                0.
            }
        }
        ActivationKind::Logistic => 1. / (1. + (-x).exp()),
        ActivationKind::Identity => x,
        ActivationKind::Softsign => x / (1. + x.abs()),
        ActivationKind::Sinusoid => x.sin(),
        ActivationKind::Gaussian => (-x.powi(2)).exp(),
        ActivationKind::BentIdentity => (((x.powi(2) + 1.).sqrt() - 1.) / 2.) + x,
        ActivationKind::Bipolar => {
            if x > 0. {
                1.
            } else {
                -1.
            }
        }
        ActivationKind::Inverse => 1. - x,
        ActivationKind::Selu => {
            let alpha = 1.6732632423543772;
            let scale = 1.05070098735548;

            let fx = if x > 0. { x } else { alpha * x.exp() - alpha };

            fx * scale
        }
        _ => panic!("Unknown activation function"),
    }
}
