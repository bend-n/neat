use godot::prelude::utilities::randf;
use godot::prelude::*;

use crate::bind;
use crate::genome::{crossover, Genome, GenomeId, GenomeMap};
use crate::mutations::{MutationKind, Pick};
use crate::network::Network;
use crate::speciation::SpeciesSet;
pub use configuration::Configuration;
use speciation::GenomeBank;

mod configuration;
mod speciation;

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct NEAT {
    #[export(get, set)]
    inputs: u32,
    #[export(get, set)]
    outputs: u32,
    #[export(get, set)]
    fitness_fn: Callable,
    #[export(get, set)]
    reporter_fn: Callable,
    #[export(get, set)]
    genomes: Gd<GenomeBank>,
    #[export(get, set)]
    species_set: Gd<SpeciesSet>,
    #[export(get, set)]
    configuration: Gd<Configuration>,
}

#[godot_api]
impl RefCountedVirtual for NEAT {
    fn init(_base: Base<Self::Base>) -> Self {
        Self::default()
    }
}

impl Default for NEAT {
    fn default() -> Self {
        let configuration: Gd<Configuration> = Gd::new_default();
        NEAT {
            inputs: 0,
            outputs: 0,
            fitness_fn: Callable::default(),
            reporter_fn: Callable::default(),
            genomes: Gd::new_default(),
            species_set: Gd::new(SpeciesSet::new(configuration.share())),
            configuration,
        }
    }
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct StartResult {
    #[export(get, set)]
    pub network: Gd<Network>,
    #[export(get, set)]
    pub best_fitness: f64,
}

#[godot_api]
impl StartResult {}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct BestResult {
    #[export(get, set)]
    pub best_id: i64,
    #[export(get, set)]
    pub best_fitness: f64,
}

#[godot_api]
impl BestResult {}

#[godot_api]
impl NEAT {
    #[func]
    pub fn start(&mut self) -> Gd<StartResult> {
        let (population_size, max_generations) = {
            let config = self.configuration.bind();
            (config.population_size, config.max_generations)
        };

        // Create initial genomes
        (0..population_size).for_each(|_| {
            self.genomes
                .bind_mut()
                .add_genome(Gd::new(Genome::new(self.inputs, self.outputs)))
        });

        self.test_fitness();

        for i in 1..=max_generations {
            let current_genome_ids: Vec<GenomeId> = self.genomes.bind().genomes().keys().collect();
            let previous_and_current_genomes = GenomeMap::from_vec(
                self.genomes
                    .bind()
                    .genomes()
                    .iter()
                    .chain(self.genomes.bind().previous_genomes().iter())
                    .collect(),
            );
            self.species_set.bind_mut().speciate(
                i,
                &current_genome_ids,
                previous_and_current_genomes,
            );

            let (elitism, population_size, mutation_rate, survival_ratio) = {
                let config = self.configuration.bind();

                (
                    config.elitism,
                    config.population_size,
                    config.mutation_rate,
                    config.survival_ratio,
                )
            };
            assert_ne!(self.species_set.bind().species().len(), 0);
            let offspring: Vec<Gd<Genome>> = self
                .species_set
                .bind()
                .species()
                .values()
                .flat_map(|species| {
                    bind!(species);
                    let offspring_count: usize = (species.adjusted_fitness.unwrap()
                        * population_size as f64)
                        .ceil() as usize;
                    let elites_count: usize = (offspring_count as f64 * elitism).ceil() as usize;
                    let nonelites_count: usize = offspring_count - elites_count;

                    let mut member_ids_and_fitnesses: Vec<(GenomeId, f64)> = species
                        .members
                        .iter()
                        .map(|member_id| {
                            (
                                *member_id,
                                self.genomes
                                    .bind()
                                    .get(*member_id)
                                    .unwrap()
                                    .bind()
                                    .fitness
                                    .unwrap(),
                            )
                        })
                        .collect();

                    member_ids_and_fitnesses.sort_by(|a, b| {
                        use std::cmp::Ordering::*;

                        let fitness_a = a.1;
                        let fitness_b = b.1;

                        if fitness_a > fitness_b {
                            Less
                        } else {
                            Greater
                        }
                    });

                    // Pick survivors
                    let surviving_count: usize =
                        (member_ids_and_fitnesses.len() as f64 * survival_ratio).ceil() as usize;
                    member_ids_and_fitnesses.truncate(surviving_count);

                    let elite_children: Vec<Gd<Genome>> =
                        (0..usize::min(elites_count, member_ids_and_fitnesses.len()))
                            .map(|elite_index| {
                                let (elite_genome_id, _) =
                                    member_ids_and_fitnesses.get(elite_index).unwrap();
                                self.genomes.bind().get(*elite_genome_id).unwrap()
                            })
                            .collect();

                    let crossover_data: Vec<(Gd<Genome>, f64, Gd<Genome>, f64)> = (0
                        ..nonelites_count)
                        .map(|_| {
                            let (parent_a_id, parent_a_fitness) = member_ids_and_fitnesses.rande();
                            let (parent_b_id, parent_b_fitness) = member_ids_and_fitnesses.rande();

                            (
                                self.genomes.bind().get(*parent_a_id).unwrap(),
                                *parent_a_fitness,
                                self.genomes.bind().get(*parent_b_id).unwrap(),
                                *parent_b_fitness,
                            )
                        })
                        .collect();

                    // TODO: rayon here
                    let mut crossover_children: Vec<Gd<Genome>> = crossover_data
                        .into_iter()
                        .map(|(parent_a, fitness_a, parent_b, fitness_b)| {
                            crossover((&parent_a.bind(), fitness_a), (&parent_b.bind(), fitness_b))
                        })
                        .filter(|maybe_genome| maybe_genome.is_some())
                        .map(|maybe_genome| Gd::new(maybe_genome.unwrap()))
                        .collect();

                    let mutations_for_children: Vec<Option<MutationKind>> = crossover_children
                        .iter()
                        .map(|_| {
                            if randf() < mutation_rate {
                                Some(self.pick_mutation())
                            } else {
                                None
                            }
                        })
                        .collect();

                    // TODO: use par iter here
                    crossover_children
                        .iter_mut()
                        .zip(mutations_for_children)
                        .for_each(|(child, maybe_mutation)| {
                            if let Some(mutation) = maybe_mutation {
                                child.bind_mut().mutate(&mutation);
                            }
                        });

                    elite_children
                        .into_iter()
                        .chain(crossover_children)
                        .collect::<Vec<Gd<Genome>>>()
                })
                .collect();

            self.genomes.bind_mut().clear();
            assert_ne!(offspring.len(), 0);
            offspring
                .into_iter()
                .for_each(|genome| self.genomes.bind_mut().add_genome(genome));
            self.test_fitness();
            self.reporter_fn.callv(varray![i]);
            let goal_reached = {
                if let Some(goal) = self.configuration.bind().fitness_goal {
                    self.get_best().bind().best_fitness >= goal
                } else {
                    false
                }
            };

            if goal_reached {
                break;
            }
        }

        let best = self.get_best();
        let res = Gd::new(StartResult {
            network: Network::from_genome(
                &self
                    .genomes
                    .bind()
                    .get(best.bind().best_id.into())
                    .unwrap()
                    .bind(),
            ),
            best_fitness: best.bind().best_fitness,
        });
        res
    }

    fn test_fitness(&mut self) {
        let id_and_fitness: Vec<(GenomeId, f64)> = self
            .genomes
            .bind()
            .genomes()
            .iter()
            .map(|(genome_id, genome)| {
                bind!(genome);
                let net = Network::from_genome(&genome);
                let mut fitness = f64::from_variant(&self.fitness_fn.callv(varray![net]));

                fitness -= self.configuration.bind().node_cost * genome.nodes().len() as f64;
                fitness -=
                    self.configuration.bind().connection_cost * genome.connections().len() as f64;
                (genome_id, fitness)
            })
            .collect();
        id_and_fitness
            .into_iter()
            .for_each(|(genome_id, genome_fitness)| {
                self.genomes
                    .bind_mut()
                    .mark_fitness(genome_id, genome_fitness);
            });
    }

    #[func]
    pub fn get_best(&self) -> Gd<BestResult> {
        assert!(!self.genomes.bind().genomes().is_empty());
        let (best_id, best_fit) = self
            .genomes
            .bind()
            .genomes()
            .iter()
            .map(|(gid, g)| (gid, g.bind().fitness.unwrap()))
            .fold(
                (GenomeId::default(), f64::MIN),
                |(best_id, best_fitness), (genome_id, genome_fitness)| {
                    if genome_fitness > best_fitness {
                        (genome_id, genome_fitness)
                    } else {
                        (best_id, best_fitness)
                    }
                },
            );
        // println!("{:#?}", self.genomes.genomes().len()); the problem is it goes to 0 at the end sometimes
        Gd::new(BestResult {
            best_id: best_id.into(),
            best_fitness: best_fit,
        })
    }

    fn pick_mutation(&self) -> MutationKind {
        self.configuration.bind().mutation_kinds.rande().0
    }
}
