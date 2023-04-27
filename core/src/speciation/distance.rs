use godot::prelude::*;

use crate::Configuration;
use crate::{ConnectionGene, Genome};

pub struct GenomicDistanceCache {
    configuration: Gd<Configuration>,
    cache: Dictionary,
}

impl GenomicDistanceCache {
    pub fn new(configuration: Gd<Configuration>) -> Self {
        GenomicDistanceCache {
            configuration,
            cache: Dictionary::new(),
        }
    }

    pub fn get(&mut self, a: Gd<Genome>, b: Gd<Genome>) -> f64 {
        let distance_key = GodotString::from(GenomicDistanceCache::make_key(&a.bind(), &b.bind()));

        if let Some(distance) = self.cache.get(distance_key.clone()) {
            f64::from_variant(&distance)
        } else {
            let distance = self.distance(&a.bind(), &b.bind());
            self.cache.insert(distance_key, distance);

            distance
        }
    }

    fn distance(&self, a: &Genome, b: &Genome) -> f64 {
        let (
            distance_connection_disjoint_coefficient,
            distance_connection_weight_coeficcient,
            distance_connection_disabled_coefficient,
            distance_node_bias_coefficient,
            distance_node_activation_coefficient,
            distance_node_aggregation_coefficient,
        ) = {
            let conf = self.configuration.bind();

            (
                conf.distance_connection_disjoint_coefficient,
                conf.distance_connection_weight_coeficcient,
                conf.distance_connection_disabled_coefficient,
                conf.distance_node_bias_coefficient,
                conf.distance_node_activation_coefficient,
                conf.distance_node_aggregation_coefficient,
            )
        };

        let mut distance = 0.;

        let max_connection_genes = usize::max(a.connections().len(), b.connections().len());

        let mut disjoint_connections: Vec<&ConnectionGene> = vec![];
        let mut common_connections: Vec<(&ConnectionGene, &ConnectionGene)> = vec![];

        let mut disjoint_map = Dictionary::new(); // Dictionary<usize, bool>
        a.connections()
            .iter()
            .chain(b.connections().iter())
            .map(|connection| connection.innovation_number())
            .for_each(|innovation_number| {
                disjoint_map.insert(
                    innovation_number,
                    !disjoint_map.contains_key(innovation_number),
                );
            });

        disjoint_map
            .iter_shared()
            .map(|(x, y)| (u32::from_variant(&x), bool::from_variant(&y)))
            .for_each(|(innovation_number, is_disjoint)| {
                if is_disjoint {
                    let disjoint_connection = a
                        .connections()
                        .iter()
                        .chain(b.connections().iter())
                        .find(|connection| connection.innovation_number() == innovation_number)
                        .unwrap();

                    disjoint_connections.push(disjoint_connection);
                } else {
                    let common_connection_a = a
                        .connections()
                        .iter()
                        .find(|connection| connection.innovation_number() == innovation_number)
                        .unwrap();
                    let common_connection_b = b
                        .connections()
                        .iter()
                        .find(|connection| connection.innovation_number() == innovation_number)
                        .unwrap();

                    common_connections.push((common_connection_a, common_connection_b));
                }
            });

        let disjoint_factor =
            disjoint_connections.len() as f64 * distance_connection_disjoint_coefficient;

        let connections_difference_factor: f64 = common_connections
            .iter()
            .map(|(connection_a, connection_b)| {
                let mut connection_distance = 0.;

                if connection_a.disabled != connection_b.disabled {
                    connection_distance += 1. * distance_connection_disabled_coefficient;
                }

                connection_distance += (connection_a.weight - connection_b.weight).abs()
                    * distance_connection_weight_coeficcient;

                connection_distance
            })
            .sum::<f64>();

        let nodes_difference_factor: f64 = a
            .nodes()
            .iter()
            .zip(b.nodes())
            .map(|(node_a, node_b)| {
                let mut node_distance = 0.;

                if node_a.activation != node_b.activation {
                    node_distance += 1. * distance_node_activation_coefficient;
                }

                if node_a.aggregation != node_b.aggregation {
                    node_distance += 1. * distance_node_aggregation_coefficient;
                }

                node_distance += (node_a.bias - node_b.bias).abs() * distance_node_bias_coefficient;

                node_distance
            })
            .sum();

        distance += nodes_difference_factor;
        distance += (connections_difference_factor + disjoint_factor) / max_connection_genes as f64;

        distance
    }

    // pub fn mean(&self) -> f64 {
    //     self.cache.values().sum::<f64>() / self.cache.len() as f64
    // }

    fn make_key(a: &Genome, b: &Genome) -> String {
        let a = a.id();
        let b = b.id();
        if i64::from(a) > i64::from(b) {
            a.full_str() + b.full_str().as_str()
        } else {
            b.full_str() + a.full_str().as_str()
        }
    }
}
