use godot::prelude::utilities::randf;

use super::{ConnectionGene, Genome, NodeGene};

pub fn crossover(a: (&Genome, f64), b: (&Genome, f64)) -> Option<Genome> {
    if (a.0.inputs != b.0.inputs) || (a.0.outputs != b.0.outputs) {
        return None;
    }

    let mut parent_a = a.0.clone();
    let mut fitness_a = a.1;

    let mut parent_b = b.0.clone();
    let mut fitness_b = b.1;

    // Parent A will always be the fitter one
    if fitness_a < fitness_b {
        std::mem::swap(&mut parent_a, &mut parent_b);
        std::mem::swap(&mut fitness_a, &mut fitness_b);
    }

    let mut child = Genome::empty(parent_a.inputs, parent_a.outputs);

    let child_connection_genes: Vec<ConnectionGene> = parent_a
        .connection_genes
        .iter()
        .map(|connection| {
            let maybe_counterpart_connection = parent_b
                .connection_genes
                .iter()
                .find(|cb| cb.innovation_number() == connection.innovation_number());

            // Chooses connection from one of the parents
            let chosen_connection =
                if let Some(counterpart_connection) = maybe_counterpart_connection {
                    if randf() < 0.5 {
                        connection
                    } else {
                        counterpart_connection
                    }
                } else {
                    connection
                };

            /*
             * Chooses will the new connection be disabled
             * - disabled in both parents, 75% chance it will be disabled
             * - enabled in both parents, it will be enabled
             * - disabled in one parent, 50% chance it will stay disabled
             */
            let new_disabled = if let Some(counterpart_connection) = maybe_counterpart_connection {
                match (connection.disabled, counterpart_connection.disabled) {
                    (true, true) => randf() < 0.75,
                    (false, false) => false,
                    _ => randf() < 0.5,
                }
            } else {
                connection.disabled
            };

            let mut new_connection = chosen_connection.clone();
            new_connection.disabled = new_disabled;

            new_connection
        })
        .collect();

    let required_node_count = 1 + child_connection_genes
        .iter()
        .fold(0, |max, c| u32::max(u32::max(max, c.from), c.to));

    let child_node_genes: Vec<NodeGene> = (0..required_node_count as usize)
        .map(
            |i| match (parent_a.node_genes.get(i), parent_b.node_genes.get(i)) {
                (Some(a), Some(b)) => {
                    if randf() < 0.5 {
                        a
                    } else {
                        b
                    }
                }
                (Some(a), None) => a,
                (None, Some(b)) => b,
                _ => unreachable!(),
            },
        )
        .cloned()
        .collect();

    child.connection_genes = child_connection_genes;
    child.node_genes = child_node_genes;

    child.node_order().and(Some(child))
}
