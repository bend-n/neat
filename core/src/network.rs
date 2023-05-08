use godot::prelude::*;
use nanoserde::{DeBin, SerBin};

use crate::activation::*;
use crate::aggregations::aggregate;
use crate::connection::*;
use crate::genome::Genome;
use crate::node::*;

#[derive(Debug, GodotClass, DeBin, SerBin)]
#[class(base=RefCounted)]
pub struct Network {
    #[export(get, set)]
    pub input_count: u32,
    #[export(get, set)]
    pub output_count: u32,
    pub nodes: Vec<NeuralNode>,
    pub connections: Vec<Connection>,
    node_calculation_order: Vec<u32>,
}
trait Pack {
    fn pack(self) -> PackedByteArray
    where
        Self: Sized;
}

impl Pack for Vec<u8> {
    fn pack(self) -> PackedByteArray {
        let mut arr = PackedByteArray::new();
        for i in self.into_iter() {
            arr.push(i)
        }
        arr
    }
}

#[godot_api]
impl RefCountedVirtual for Network {
    fn init(_base: Base<RefCounted>) -> Self {
        Network {
            input_count: 0,
            output_count: 0,
            nodes: vec![],
            connections: vec![],
            node_calculation_order: vec![],
        }
    }
}

#[godot_api]
impl Network {
    /// look if it was a proper graph then this would be hard
    #[func]
    fn get_nodes(&self) -> VariantArray {
        let mut nodes: Array<Gd<NeuralNode>> = Array::new();
        let mut connections: Array<Gd<Connection>> = Array::new();
        for node in &self.nodes {
            nodes.push(Gd::new(node.clone()))
        }
        for connection in &self.connections {
            connections.push(Gd::new(connection.clone()))
        }
        varray![nodes, connections]
    }

    #[func]
    pub fn to_bytes(&self) -> PackedByteArray {
        self.serialize_bin().pack()
    }

    #[func]
    pub fn from_bytes(bytes: PackedByteArray) -> Variant {
        match Self::deserialize_bin(&bytes.to_vec()) {
            Ok(new) => Gd::new(new).to_variant(),
            Err(e) => {
                godot_error!("deserializing failed: {e:#?}");
                Variant::nil()
            }
        }
    }

    #[func]
    pub fn is_node_ready(&self, index: u32) -> bool {
        let node = self.nodes.get(index as usize).unwrap();

        let requirements_fullfilled = self.connections.iter().filter(|c| c.to == index).all(|c| {
            let from_index = c.from as usize;
            let from_node = &self.nodes[from_index];

            from_node.value.is_some()
        });
        let has_no_value = node.value.is_none();

        requirements_fullfilled && has_no_value
    }

    #[func]
    pub fn forward_pass(&mut self, inputs: PackedFloat64Array) -> PackedFloat64Array {
        for i in &self.node_calculation_order {
            let node = self.nodes.get(*i as usize).unwrap();

            if matches!(node.kind, NodeKind::Input) {
                self.nodes.get_mut(*i as usize).unwrap().value = Some(inputs.get(*i as usize));
            } else {
                let components: Vec<f64> = self
                    .connections
                    .iter()
                    .filter(|c| c.to == *i)
                    .map(|c| {
                        let incoming_value =
                            self.nodes.get(c.from as usize).unwrap().value.unwrap();
                        incoming_value * c.weight
                    })
                    .collect();

                let aggregated = aggregate(&node.aggregation, &components);
                let aggregated_with_bias = aggregated + node.bias;

                self.nodes.get_mut(*i as usize).unwrap().value =
                    Some(activate(aggregated_with_bias, &node.activation));
            }
        }

        let mut result = PackedFloat64Array::new();
        self.nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Output))
            .map(|n| n.value.unwrap())
            .for_each(|f| result.push(f));
        result // note that result can be f32::NAN
    }

    #[func]
    pub fn clear_values(&mut self) {
        self.nodes.iter_mut().for_each(|n| n.value = None);
    }
}

impl Network {
    pub fn from_genome(g: &Genome) -> Gd<Self> {
        let nodes: Vec<NeuralNode> = g.nodes().iter().map(From::from).collect();
        let connections: Vec<Connection> = g
            .connections()
            .iter()
            .filter(|c| !c.disabled)
            .map(From::from)
            .collect();

        Gd::new(Network {
            input_count: g.input_count(),
            output_count: g.output_count(),
            nodes,
            connections,
            node_calculation_order: g.node_order().unwrap(),
        })
    }
}
