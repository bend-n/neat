use godot::prelude::utilities::{randf, randfn, randi, randi_range};

use crate::genome::Genome;
use crate::node::NodeKind;
use crate::{EnumConversion, NodeGene};

pub fn mutate(kind: &MutationKind, g: &mut Genome) {
    use MutationKind::*;

    match kind {
        AddConnection => add_connection(g),
        RemoveConnection => disable_connection(g),
        AddNode => add_node(g),
        RemoveNode => remove_node(g),
        ModifyWeight => change_weight(g),
        ModifyBias => change_bias(g),
        ModifyActivation => change_activation(g),
        ModifyAggregation => change_aggregation(g),
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum MutationKind {
    AddConnection,
    RemoveConnection,
    AddNode,
    RemoveNode,
    ModifyWeight,
    ModifyBias,
    ModifyActivation,
    ModifyAggregation,
}

#[inline]
pub fn rand<T: EnumConversion>() -> T {
    let (from, to) = T::pick_range();
    T::from(randi_range(from, to) as u8)
}

pub trait Pick<T> {
    fn randi(&self) -> usize;
    fn rande(&self) -> &T;
}
impl<T> Pick<T> for [T] {
    #[inline]
    fn rande(&self) -> &T {
        self.get(self.randi()).unwrap()
    }
    #[inline]
    fn randi(&self) -> usize {
        randi() as usize % self.len()
    }
}

fn get_node_gene(g: &mut Genome) -> &mut NodeGene {
    let eligible: Vec<usize> = g
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| !matches!(n.kind, NodeKind::Input))
        .map(|(i, _)| i)
        .collect();
    g.node_mut(*eligible.rande()).unwrap()
}

/// Adds a new random connection
pub fn add_connection(g: &mut Genome) {
    let existing_connections: Vec<(u32, u32, bool)> = g
        .connections()
        .iter()
        .map(|c| (c.from, c.to, c.disabled))
        .collect();

    let mut possible_connections: Vec<(u32, u32)> = (0..g.nodes().len() as u32)
        .flat_map(|i| {
            let mut inner = vec![];

            (0..g.nodes().len() as u32).for_each(|j| {
                if i != j {
                    if !existing_connections.contains(&(i, j, false)) {
                        inner.push((i, j));
                    };
                    if !existing_connections.contains(&(j, i, false)) {
                        inner.push((j, i));
                    };
                }
            });

            inner
        })
        .collect();

    possible_connections.sort_unstable();
    possible_connections.dedup();

    possible_connections.retain(|(i, j)| g.can_connect(*i, *j));

    if possible_connections.is_empty() {
        return;
    }

    let picked_connection = possible_connections.rande();

    g.add_connection(picked_connection.0, picked_connection.1)
        .unwrap();
}

/// Removes a random connection if it's not the only one
fn disable_connection(g: &mut Genome) {
    let eligible_indexes: Vec<usize> = g
        .connections()
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            if c.disabled {
                return false;
            }

            let from_index = c.from;
            let to_index = c.to;

            // Number of outgoing connections for the `from` node
            let from_connections_count = g
                .connections()
                .iter()
                .filter(|c| c.from == from_index && !c.disabled)
                .count();
            // Number of incoming connections for the `to` node
            let to_connections_count = g
                .connections()
                .iter()
                .filter(|c| c.to == to_index && !c.disabled)
                .count();

            from_connections_count > 1 && to_connections_count > 1
        })
        .map(|(i, _)| i)
        .collect();

    if eligible_indexes.is_empty() {
        return;
    }

    let index = eligible_indexes.rande();

    g.disable_connection(*index);
}

/// Adds a random hidden node to the genome and its connections
pub fn add_node(g: &mut Genome) {
    let new_node_index = g.add_node();

    // Only enabled connections can be disabled
    let enabled_connections: Vec<usize> = g
        .connections()
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.disabled)
        .map(|(i, _)| i)
        .collect();

    let (picked_index, picked_from, picked_to, picked_weight) = {
        let picked_index = enabled_connections.rande();
        let picked_connection = g.connections().get(*picked_index).unwrap();

        (
            picked_index,
            picked_connection.from,
            picked_connection.to,
            picked_connection.weight,
        )
    };

    g.disable_connection(*picked_index);

    let connection_index = g
        .add_connection(picked_from, new_node_index as u32)
        .unwrap();
    g.add_connection(new_node_index as u32, picked_to).unwrap();

    // Reuse the weight from the removed connection
    g.connection_mut(connection_index).unwrap().weight = picked_weight;
}

/// Removes a random hidden node from the genome and rewires connected nodes
fn remove_node(g: &mut Genome) {
    let hidden_nodes: Vec<u32> = g
        .nodes()
        .iter()
        .enumerate()
        .filter(|(i, n)| {
            let i = *i as u32;
            let incoming_count = g
                .connections()
                .iter()
                .filter(|c| c.to == i && !c.disabled)
                .count();
            let outgoing_count = g
                .connections()
                .iter()
                .filter(|c| c.from == i && !c.disabled)
                .count();

            matches!(n.kind, NodeKind::Hidden) && incoming_count > 0 && outgoing_count > 0
        })
        .map(|(i, _)| i as u32)
        .collect();

    if hidden_nodes.is_empty() {
        return;
    }

    let picked_node_index = hidden_nodes.rande();

    let incoming_connections_and_from_indexes: Vec<(u32, u32)> = g
        .connections()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.to == *picked_node_index && !c.disabled)
        .map(|(i, c)| (i as u32, c.from))
        .collect();
    let outgoing_connections_and_to_indexes: Vec<(u32, u32)> = g
        .connections()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.from == *picked_node_index && !c.disabled)
        .map(|(i, c)| (i as u32, c.to))
        .collect();

    let new_from_to_pairs: Vec<(u32, u32)> = incoming_connections_and_from_indexes
        .iter()
        .flat_map(|(_, from)| {
            outgoing_connections_and_to_indexes
                .iter()
                .map(|(_, to)| (*from, *to))
                .collect::<Vec<(u32, u32)>>()
        })
        .filter(|(from, to)| {
            !g.connections()
                .iter()
                .any(|c| c.from == *from && c.to == *to && !c.disabled)
        })
        .collect();

    g.add_many_connections(&new_from_to_pairs);

    let connection_indexes_to_delete: Vec<usize> = g
        .connections()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.from == *picked_node_index || c.to == *picked_node_index)
        .map(|(i, _)| i)
        .collect();

    g.disable_many_connections(&connection_indexes_to_delete);
}

/// Changes the weight of a random connection
fn change_weight(g: &mut Genome) {
    let index = g.connections().randi();
    let picked_connection = g.connection_mut(index).unwrap();

    let new_weight = if randf() < 0.1 {
        picked_connection.weight + randfn(0.5, 0.2)
    } else {
        randf() * 2.0 - 1.0
    };

    picked_connection.weight = new_weight.max(-1.).min(1.);
}

/// Changes the bias of a random non input node
fn change_bias(g: &mut Genome) {
    let picked_node = get_node_gene(g);

    let new_bias = if randf() < 0.1 {
        picked_node.bias + randfn(0.5, 0.2)
    } else {
        randf() * 2. - 1.
    };

    picked_node.bias = new_bias.max(-1.).min(1.);
}

/// Changes the activation function of a random non input node
fn change_activation(g: &mut Genome) {
    let picked_node = get_node_gene(g);

    picked_node.activation = rand();
}

fn change_aggregation(g: &mut Genome) {
    let picked_node = get_node_gene(g);

    picked_node.aggregation = rand();
}
