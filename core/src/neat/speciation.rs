use godot::prelude::*;

use crate::genome::{Genome, GenomeId, GenomeMap};

/// Holds all genomes and species, does the process of speciation
#[derive(Debug, Default)]
pub struct GenomeBank {
    genomes: GenomeMap,
    previous_genomes: GenomeMap,
}

impl GenomeBank {
    /// Adds a new genome
    pub fn add_genome(&mut self, genome: Gd<Genome>) {
        let id = genome.bind().id();
        self.genomes.set(id, genome);
    }

    /// Clear genomes
    pub fn clear(&mut self) {
        self.previous_genomes = std::mem::take(&mut self.genomes);
    }

    pub fn get(&self, genome_id: GenomeId) -> Option<Gd<Genome>> {
        self.genomes.get(genome_id)
    }

    /// Returns a reference to the genomes
    pub fn genomes(&self) -> &GenomeMap {
        &self.genomes
    }

    pub fn previous_genomes(&self) -> &GenomeMap {
        &self.previous_genomes
    }

    /// Tracks the fitness of a particular genome
    pub fn mark_fitness(&mut self, genome_id: GenomeId, fitness: f64) -> Option<()> {
        self.get(genome_id)?.bind_mut().fitness = Some(fitness);
        Some(())
    }
}
