use godot::prelude::*;
use std::collections::VecDeque;
use std::fmt::Debug;

use crate::mutations::MutationKind;
use crate::node::NodeKind;
use crate::Map;
pub use connection::ConnectionGene;
pub use crossover::*;
pub use gid::GenomeId;
pub use node::NodeGene;

pub type GenomeMap = Map<GenomeId, Genome>;

pub mod connection;
pub mod crossover;
pub mod gid;
pub mod node;

#[derive(Clone, PartialEq, GodotClass)]
#[class(base=RefCounted)]
pub struct Genome {
    id: GenomeId,
    #[export(get, set)]
    inputs: u32,
    #[export(get, set)]
    outputs: u32,
    pub fitness: Option<f64>,
    connection_genes: Vec<ConnectionGene>,
    node_genes: Vec<NodeGene>,
}

impl Debug for Genome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Genome<{}>", self.id.uuid_str())
    }
}

#[godot_api]
impl RefCountedVirtual for Genome {
    fn to_string(&self) -> GodotString {
        format!("{self:?}").into()
    }
}

#[godot_api]
impl Genome {
    #[func]
    pub fn fitness(&self) -> f64 {
        self.fitness.unwrap_or(f64::NEG_INFINITY)
    }

    pub fn new(inputs: u32, outputs: u32) -> Self {
        let mut node_genes = vec![];

        (0..inputs).for_each(|_| node_genes.push(NodeGene::new(NodeKind::Input)));
        (0..outputs).for_each(|_| node_genes.push(NodeGene::new(NodeKind::Output)));

        let connection_genes: Vec<ConnectionGene> = (0..inputs)
            .flat_map(|i| {
                (inputs..inputs + outputs)
                    .map(|o| ConnectionGene::new(i, o))
                    .collect::<Vec<ConnectionGene>>()
            })
            .collect();

        Genome {
            id: GenomeId::default(),
            fitness: None,
            inputs,
            outputs,
            connection_genes,
            node_genes,
        }
    }

    pub fn empty(inputs: u32, outputs: u32) -> Self {
        Genome {
            id: GenomeId::default(),
            inputs,
            outputs,
            fitness: None,
            connection_genes: vec![],
            node_genes: vec![],
        }
    }

    pub fn id(&self) -> GenomeId {
        self.id
    }

    pub fn input_count(&self) -> u32 {
        self.inputs
    }

    pub fn output_count(&self) -> u32 {
        self.outputs
    }

    pub fn nodes(&self) -> &[NodeGene] {
        &self.node_genes
    }

    pub fn node_mut(&mut self, index: usize) -> Option<&mut NodeGene> {
        self.node_genes.get_mut(index)
    }

    pub fn connections(&self) -> &[ConnectionGene] {
        &self.connection_genes
    }

    pub fn connection_mut(&mut self, index: usize) -> Option<&mut ConnectionGene> {
        self.connection_genes.get_mut(index)
    }

    fn calculate_node_order(
        &self,
        additional_connections: Option<Vec<ConnectionGene>>,
    ) -> Option<Vec<u32>> {
        let mut connections: Vec<ConnectionGene> = self
            .connection_genes
            .iter()
            .filter(|c| !c.disabled)
            .cloned()
            .collect();

        if let Some(mut conns) = additional_connections {
            connections.append(&mut conns);
        }

        if connections.is_empty() {
            return None;
        }

        let mut visited: Vec<u32> = vec![];

        // Input nodes are automatically visited as they get their values from inputs
        self.node_genes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, NodeKind::Input))
            .for_each(|(i, _)| {
                visited.push(i as u32);
            });

        let mut newly_visited = 1;
        while newly_visited != 0 {
            newly_visited = 0;

            let mut nodes_to_visit: Vec<u32> = (0..self.node_genes.len())
                .map(|x| x as u32)
                .filter(|i| {
                    // The node is not visited but all prerequisite nodes are visited
                    !visited.contains(i)
                        && connections
                            .iter()
                            .filter(|c| c.to == *i)
                            .map(|c| c.from)
                            .all(|node_index| visited.contains(&node_index))
                })
                .collect();

            newly_visited += nodes_to_visit.len();
            visited.append(&mut nodes_to_visit);
        }

        if visited.len() != self.node_genes.len() {
            return None;
        }

        Some(visited)
    }

    pub fn node_order(&self) -> Option<Vec<u32>> {
        self.calculate_node_order(None)
    }

    pub fn node_order_with(&self, additional_connections: Vec<ConnectionGene>) -> Option<Vec<u32>> {
        self.calculate_node_order(Some(additional_connections))
    }

    /// Dictionary<u32,u32>
    fn calculate_node_distance_from_inputs(&self) -> Dictionary {
        // Inputs are immediately added with distance of 0
        let mut distances = Dictionary::new();
        self.nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, NodeKind::Input))
            .map(|(i, _)| i as u32)
            .for_each(|i| {
                distances.insert(i, 0);
            });

        // Inputs need to be visited first
        let mut to_visit: VecDeque<u32> = self
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, NodeKind::Input))
            .map(|(i, _)| i as u32)
            .collect();

        while let Some(i) = to_visit.pop_front() {
            let source_distance = u32::from_variant(&distances.get(i).unwrap_or(0.to_variant()));

            self.connections()
                .iter()
                .filter(|c| c.from == i)
                .for_each(|c| {
                    let node_index = c.to;
                    let potential_distance = source_distance + 1;

                    let maybe_change = if let Some(distance) = distances.get(node_index) {
                        let distance = u32::from_variant(&distance);
                        if potential_distance > distance {
                            to_visit.push_back(node_index);
                            Some(potential_distance)
                        } else {
                            None
                        }
                    } else {
                        to_visit.push_back(node_index);
                        Some(potential_distance)
                    };

                    if let Some(new_distance) = maybe_change {
                        distances.insert(node_index, new_distance);
                    }
                });
        }

        distances
    }

    fn is_projecting_directly(&self, source: u32, target: u32) -> bool {
        self.connection_genes
            .iter()
            .filter(|c| !c.disabled)
            .any(|c| c.from == source && c.to == target)
    }

    // fn is_projected_directly(&self, target: usize, source: usize) -> bool {
    //     self.is_projecting_directly(source, target)
    // }

    fn is_projecting(&self, source: u32, target: u32) -> bool {
        let mut visited_nodes = Dictionary::new();
        let mut nodes_to_visit: VecDeque<u32> = VecDeque::new();

        nodes_to_visit.push_back(source);

        let mut projecting = false;
        while let Some(i) = nodes_to_visit.pop_front() {
            visited_nodes.insert(i, Variant::nil());
            if self.is_projecting_directly(i, target) {
                projecting = true;
                break;
            } else {
                self.connection_genes
                    .iter()
                    .filter(|c| c.from == i && !c.disabled && !visited_nodes.contains_key(i))
                    .for_each(|c| nodes_to_visit.push_back(c.to));
            }
        }

        projecting
    }

    // fn is_projected(&self, target: usize, source: usize) -> bool {
    //     self.is_projecting(source, target)
    // }

    pub fn can_connect(&self, from: u32, to: u32) -> bool {
        let from_node = self.node_genes.get(from as usize).unwrap();
        let to_node = self.node_genes.get(to as usize).unwrap();

        let is_from_output = matches!(from_node.kind, NodeKind::Output);
        let is_to_input = matches!(to_node.kind, NodeKind::Input);

        let distances = self.calculate_node_distance_from_inputs();
        let from_distance = u32::from_variant(&distances.get(from).unwrap());
        let to_distance = u32::from_variant(&distances.get(to).unwrap_or(u32::MAX.to_variant()));
        let is_recurrent = from_distance > to_distance;

        if is_from_output || is_to_input || is_recurrent {
            false
        } else {
            !self.is_projecting(from, to)
        }
    }

    pub fn add_connection(&mut self, from: u32, to: u32) -> Option<usize> {
        if !self.can_connect(from, to) {
            return None;
        }

        let maybe_connection = self
            .connection_genes
            .iter_mut()
            .find(|c| c.from == from && c.to == to);

        if let Some(mut conn) = maybe_connection {
            conn.disabled = false;
        } else {
            self.connection_genes.push(ConnectionGene::new(from, to));
        }

        Some(self.connection_genes.len() - 1)
    }

    pub fn add_many_connections(&mut self, params: &[(u32, u32)]) -> Vec<Option<usize>> {
        let results = params
            .iter()
            .map(|(from, to)| self.add_connection(*from, *to))
            .collect();

        results
    }

    pub fn disable_connection(&mut self, index: usize) {
        self.connection_genes.get_mut(index).unwrap().disabled = true;
    }

    pub fn disable_many_connections(&mut self, indexes: &[usize]) {
        indexes.iter().for_each(|i| self.disable_connection(*i));
    }

    /// Add a new hidden node to the genome
    pub fn add_node(&mut self) -> usize {
        let index = self.node_genes.len();
        self.node_genes.push(NodeGene::new(NodeKind::Hidden));

        index
    }

    pub fn mutate(&mut self, kind: &MutationKind) {
        crate::mutations::mutate(kind, self);
    }
}
