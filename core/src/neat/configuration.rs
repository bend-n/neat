use godot::prelude::*;

use crate::mutations::MutationKind;

/// Holds configuration options of the whole NEAT process
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct Configuration {
    /// The generations limit of for the evolution process
    #[export(get, set)]
    pub max_generations: u32,

    /// The maximum number of genomes in each generation
    #[export(get, set)]
    pub population_size: u32,

    /// The ratio of champion individuals that are copied to the next generation
    #[export(get, set)]
    pub elitism: f64,

    /// The minimum amount of species that need to exist after the removal of stagnated ones
    #[export(get, set)]
    pub elitism_species: u32,

    /// How many generations of not making progress is considered stagnation
    #[export(get, set)]
    pub stagnation_after: u32,

    /// The fitness cost of every node in the gene
    #[export(get, set)]
    pub node_cost: f64,

    /// The fitness cost of every connection in the gene
    #[export(get, set)]
    pub connection_cost: f64,

    /// The mutation rate of offspring
    #[export(get, set)]
    pub mutation_rate: f64,

    /// The ratio of genomes that will survive to the next generation
    #[export(get, set)]
    pub survival_ratio: f64,

    /// The types of mutations available and their sampling weights
    pub mutation_kinds: Vec<(MutationKind, usize)>,

    /// The process will stop if the fitness goal is reached
    pub fitness_goal: Option<f64>,

    /*
     * Genomic distance during speciation
     */
    /// Controls how much connections can affect distance
    #[export(get, set)]
    pub distance_connection_disjoint_coefficient: f64,
    #[export(get, set)]
    pub distance_connection_weight_coeficcient: f64,
    #[export(get, set)]
    pub distance_connection_disabled_coefficient: f64,

    /// Controls how much nodes can affect distance
    #[export(get, set)]
    pub distance_node_bias_coefficient: f64,
    #[export(get, set)]
    pub distance_node_activation_coefficient: f64,
    #[export(get, set)]
    pub distance_node_aggregation_coefficient: f64,

    /// A limit on how distant two genomes can be to belong to the same species
    #[export(get, set)]
    pub compatibility_threshold: f64,
}

#[godot_api]
impl RefCountedVirtual for Configuration {
    fn init(_base: Base<RefCounted>) -> Self {
        Configuration {
            max_generations: 1000,
            population_size: 150,
            elitism: 0.1,
            elitism_species: 3,
            stagnation_after: 50,
            node_cost: 0.,
            connection_cost: 0.,
            mutation_rate: 0.5,
            survival_ratio: 0.5,
            mutation_kinds: default_mutation_kinds(),
            fitness_goal: None,
            distance_connection_disjoint_coefficient: 1.,
            distance_connection_weight_coeficcient: 0.5,
            distance_connection_disabled_coefficient: 0.5,
            distance_node_bias_coefficient: 0.33,
            distance_node_activation_coefficient: 0.33,
            distance_node_aggregation_coefficient: 0.33,
            compatibility_threshold: 3.,
        }
    }
}

pub fn default_mutation_kinds() -> Vec<(MutationKind, usize)> {
    use MutationKind::*;

    vec![
        (AddConnection, 10), // weights currently disabled
        (RemoveConnection, 10),
        (AddNode, 10),
        (RemoveNode, 10),
        (ModifyWeight, 10),
        (ModifyBias, 10),
        (ModifyActivation, 10),
        (ModifyAggregation, 10),
    ]
}

#[godot_api]
impl Configuration {
    #[func]
    fn set_fitness_goal(&mut self, to: Variant) {
        if to.is_nil() {
            self.fitness_goal = None
        } else {
            self.fitness_goal = Some(f64::from_variant(&to))
        }
    }
}
