use crate::EnumConversion;
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
impl EnumConversion for ActivationKind {
    fn from(i: u8) -> Self {
        match i {
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
            11 => ActivationKind::Inverse,
            _ => ActivationKind::Input,
        }
    }
    fn to(self) -> u8 {
        match self {
            ActivationKind::Tanh => 0,
            ActivationKind::Relu => 1,
            ActivationKind::Step => 2,
            ActivationKind::Logistic => 3,
            ActivationKind::Identity => 4,
            ActivationKind::Softsign => 5,
            ActivationKind::Sinusoid => 6,
            ActivationKind::Gaussian => 7,
            ActivationKind::BentIdentity => 8,
            ActivationKind::Bipolar => 9,
            ActivationKind::Selu => 10,
            ActivationKind::Inverse => 11,
            ActivationKind::Input => 12,
        }
    }
    fn pick_range() -> (i64, i64) {
        (0, 11)
    }
    fn len() -> u8 {
        12
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
        _ => unreachable!(),
    }
}
