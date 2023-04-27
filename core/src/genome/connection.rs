use godot::prelude::utilities::randf;

#[derive(Debug, Clone)]
pub struct ConnectionGene {
    pub from: u32,
    pub to: u32,
    pub weight: f64,
    pub disabled: bool,
}

impl ConnectionGene {
    pub fn new(from: u32, to: u32) -> Self {
        ConnectionGene {
            from,
            to,
            weight: randf() * 2.0 - 1.0,
            disabled: false,
        }
    }

    pub fn innovation_number(&self) -> u32 {
        let a = self.from;
        let b = self.to;

        let first_part = (a + b) * (a + b + 1);
        let second_part = b;

        first_part.checked_div(2).unwrap() + second_part
    }
}

impl PartialEq for ConnectionGene {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from
            && self.to == other.to
            && self.disabled == other.disabled
            && (self.weight - other.weight).abs() < f64::EPSILON
    }
}

impl Eq for ConnectionGene {}
