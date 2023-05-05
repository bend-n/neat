use crate::genome::GenomeId;
use crate::Configuration;
use crate::GenomeMap;
use crate::Map;
use crate::{bind, bind_mut};
use godot::prelude::*;

pub type SpeciesMap = Map<u32, Species>;
use distance::GenomicDistanceCache;

mod distance;

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct SpeciesSet {
    configuration: Gd<Configuration>,
    // last_index: Option<usize>,
    species: SpeciesMap,
}

#[godot_api]
impl SpeciesSet {
    #[func]
    fn get_species(&self) -> Dictionary {
        self.species.d()
    }

    pub fn new(configuration: Gd<Configuration>) -> Self {
        SpeciesSet {
            configuration,
            // last_index: None,
            species: Map::new(),
        }
    }

    pub fn species(&self) -> &SpeciesMap {
        &self.species
    }

    pub fn speciate(
        &mut self,
        generation: u32,
        current_genomes: &[GenomeId],
        all_genomes: GenomeMap,
    ) {
        let (compatibility_threshold, stagnation_after, elitism_species) = {
            let config = self.configuration.bind();

            (
                config.compatibility_threshold,
                config.stagnation_after,
                config.elitism_species,
            )
        };

        let mut distances = GenomicDistanceCache::new(self.configuration.share());

        let mut unspeciated_genomes = Dictionary::new();
        for g in current_genomes.iter() {
            unspeciated_genomes.set(*g, Variant::nil())
        }
        let mut new_species = self.species.clone();

        // Find new representatives for existing species
        self.species.iter().for_each(|(species_id, species)| {
            bind!(species);
            let genome_representative = all_genomes.get(species.representative).unwrap();

            let (maybe_new_representative_id, _) = current_genomes
                .iter()
                .map(|genome_id| {
                    let genome = all_genomes.get(*genome_id).unwrap();
                    (
                        genome_id,
                        distances.get(genome, genome_representative.share()),
                    )
                })
                .filter(|(_, distance)| *distance < compatibility_threshold)
                .fold(
                    (None, f64::MAX),
                    |(maybe_closest_genome_id, closest_genome_distance),
                     (genome_id, genome_distance)| {
                        if maybe_closest_genome_id.is_some() {
                            if genome_distance < closest_genome_distance {
                                return (Some(genome_id), genome_distance);
                            }
                        } else {
                            return (Some(genome_id), genome_distance);
                        }

                        (maybe_closest_genome_id, closest_genome_distance)
                    },
                );

            if let Some(new_representative_id) = maybe_new_representative_id {
                let mut species = new_species.get(species_id).unwrap();
                species.bind_mut().representative = *new_representative_id;
                species.bind_mut().members = vec![*new_representative_id];
                new_species.set(species_id, species);

                unspeciated_genomes.remove(*new_representative_id);
            } else {
                new_species.erase(species_id);
            }
        });

        // Put unspeciated genomes into species
        unspeciated_genomes.keys_shared().for_each(|genome_id| {
            let genome_id = GenomeId::from_variant(&genome_id);
            let genome = all_genomes.get(genome_id).unwrap();

            let (maybe_closest_species_id, _) = {
                new_species
                    .iter()
                    .map(|(species_id, species)| {
                        bind!(species);
                        let species_representative_genome =
                            all_genomes.get(species.representative).unwrap();
                        (
                            species_id,
                            distances.get(genome.share(), species_representative_genome),
                        )
                    })
                    .filter(|(_, distance)| *distance < compatibility_threshold)
                    .fold(
                        (None, f64::MAX),
                        |(maybe_closest_species_id, closest_representative_distance),
                         (species_id, representative_distance)| {
                            if maybe_closest_species_id.is_some() {
                                if representative_distance < closest_representative_distance {
                                    return (Some(species_id), representative_distance);
                                }
                            } else {
                                return (Some(species_id), representative_distance);
                            }

                            (maybe_closest_species_id, closest_representative_distance)
                        },
                    )
            };

            if let Some(closest_species_id) = maybe_closest_species_id {
                // Fits into an existing species
                new_species
                    .get(closest_species_id)
                    .unwrap()
                    .bind_mut()
                    .members
                    .push(genome_id);
            } else {
                // Needs to go in a brand new species
                let species = Species::new(generation, genome_id, vec![genome_id]);
                let next_species_id = self.species.keys().max().unwrap_or(0) + 1;

                new_species.set(next_species_id, Gd::new(species));
            }
        });

        // Calculate fitness for every species
        new_species.values().for_each(|mut species| {
            bind_mut!(species);
            let member_fitnesses: Vec<f64> = species
                .members
                .iter()
                .map(|member_genome_id| {
                    all_genomes
                        .get(*member_genome_id)
                        .unwrap()
                        .bind()
                        .fitness
                        .unwrap()
                })
                .collect();

            let species_mean_fitness =
                member_fitnesses.iter().sum::<f64>() / member_fitnesses.len() as f64;
            let best_previous_fitness = species
                .fitness_history
                .iter()
                .cloned()
                .fold(f64::MIN, f64::max);

            if species_mean_fitness > best_previous_fitness {
                species.last_improved = generation;
            }

            species.fitness = Some(species_mean_fitness);
            species.fitness_history.push(species_mean_fitness);
        });

        // Calculate adjusted fitness for every species
        let species_fitnesses: Vec<f64> = new_species
            .values()
            .map(|species| species.bind().fitness.unwrap())
            .collect();
        let exp_sum: f64 = species_fitnesses.iter().map(|fitness| fitness.exp()).sum();
        new_species.values().for_each(|mut species| {
            bind_mut!(species);
            let own_exp = species.fitness.unwrap().exp();
            let adjusted_fitness = own_exp / exp_sum;
            species.adjusted_fitness = Some(adjusted_fitness);
        });

        // Remove stagnated species
        let mut stagnated_ids_and_adjusted_fitnesses: Vec<(u32, f64)> = new_species
            .iter()
            .filter(|(_, species)| generation - species.bind().last_improved >= stagnation_after)
            .map(|(id, species)| (id, species.bind().adjusted_fitness.unwrap()))
            .collect();

        stagnated_ids_and_adjusted_fitnesses.sort_by(|(_, a), (_, b)| {
            use std::cmp::Ordering::*;

            if a > b {
                Less
            } else {
                Greater
            }
        });

        stagnated_ids_and_adjusted_fitnesses
            .into_iter()
            .take(usize::max(new_species.len() - elitism_species as usize, 0))
            .for_each(|(id, _)| {
                new_species.remove(id).unwrap();
            });

        // Finally replace old species
        self.species = new_species;
    }
}

#[derive(Clone, GodotClass)]
#[class(base=RefCounted)]
pub struct Species {
    // created: usize,
    last_improved: u32,
    representative: GenomeId,
    pub members: Vec<GenomeId>,

    fitness: Option<f64>,
    pub adjusted_fitness: Option<f64>,
    fitness_history: Vec<f64>,
}

#[godot_api]
impl RefCountedVirtual for Species {
    fn to_string(&self) -> GodotString {
        GodotString::from(format!("{self:?}"))
    }
}

impl std::fmt::Debug for Species {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Species<{}>", self.representative.uuid_str())
    }
}

impl Species {
    pub fn new(generation: u32, representative: GenomeId, members: Vec<GenomeId>) -> Self {
        Species {
            // created: generation,
            last_improved: generation,
            representative,
            members,
            fitness: None,
            adjusted_fitness: None,
            fitness_history: vec![],
        }
    }
}
