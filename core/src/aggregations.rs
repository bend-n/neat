use crate::EnumConversion;
use nanoserde::{DeBin, SerBin};

pub fn aggregate(kind: &Aggregation, components: &[f64]) -> f64 {
    use Aggregation::*;

    match kind {
        Product => components
            .iter()
            .fold(1., |result, current| result * current),
        Sum => components.iter().sum(),
        Max => components.iter().fold(
            f64::MIN,
            |max, current| if *current > max { *current } else { max },
        ),
        Min => components.iter().fold(
            f64::MAX,
            |min, current| if *current < min { *current } else { min },
        ),
        MaxAbs => max(&components
            .iter()
            .map(|component| component.abs())
            .collect::<Vec<f64>>()),
        Median => {
            use std::cmp::Ordering;

            if components.is_empty() {
                return 0.;
            }

            let mut sorted = components.to_vec();
            sorted.sort_by(|a, b| {
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });

            let length = sorted.len();
            let is_length_even = length % 2 == 0;
            let median_index = if is_length_even {
                length / 2 - 1
            } else {
                length / 2
            };

            *sorted.get(median_index).unwrap()
        }
        Mean => sum(components) / components.len() as f64,
    }
}

#[derive(Debug, Clone, PartialEq, DeBin, SerBin)]
pub enum Aggregation {
    Product,
    Sum,
    Max,
    Min,
    MaxAbs,
    Median,
    Mean,
}

impl EnumConversion for Aggregation {
    fn from(i: u8) -> Self {
        use Aggregation::*;
        match i {
            0 => Product,
            1 => Sum,
            2 => Max,
            3 => Min,
            4 => MaxAbs,
            5 => Median,
            _ => Mean,
        }
    }
    fn to(self) -> u8 {
        use Aggregation::*;
        match self {
            Product => 0,
            Sum => 1,
            Max => 2,
            Min => 3,
            MaxAbs => 4,
            Median => 5,
            Mean => 6,
        }
    }
    fn len() -> u8 {
        6
    }
}

#[inline]
fn sum(components: &[f64]) -> f64 {
    components.iter().sum()
}

#[inline]
fn max(components: &[f64]) -> f64 {
    components.iter().fold(
        f64::MIN,
        |max, current| if *current > max { *current } else { max },
    )
}

