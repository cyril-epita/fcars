use crate::FormalConcept;
use crate::FormalContext;
use crate::RawFormalConcept;
use rayon::iter::walk_tree;
use rayon::prelude::*;

impl<A: Sync, B: Sync> FormalContext<A, B> {
    /// Returns a parallel iterator over all formal concepts for this formal context.
    pub fn all_concepts_raw_par_iter(&self) -> impl ParallelIterator<Item = RawFormalConcept> {
        walk_tree((self.max_concept_raw(), 0), |(concept, y)| {
            // This is the CbO algorithm, as expressed in the PCbO paper.
            let mut result = vec![];
            for j in concept.intent.iter_zeros().filter(|&j| j >= *y) {
                let c = concept.extent.clone() & self.get_attribute_extent(j);
                let d = self.induce_r(&c);
                if concept.intent[0..j] == d[0..j] {
                    result.push((
                        RawFormalConcept {
                            extent: c,
                            intent: d,
                        },
                        j + 1,
                    ));
                }
            }
            result
        })
        .map(|(c, _)| c)
    }
    pub fn all_concepts_raw(&self) -> Vec<RawFormalConcept> {
        self.all_concepts_raw_par_iter().collect()
    }
    pub fn num_concepts(&self) -> usize {
        self.all_concepts_raw_par_iter().count()
    }
}

impl<A: Clone + Send + Sync, B: Clone + Send + Sync> FormalContext<A, B> {
    pub fn all_concepts_par_iter(&self) -> impl ParallelIterator<Item = FormalConcept<A, B>> {
        let arc = self.arc();
        self.all_concepts_raw_par_iter()
            .map(move |c| c.to_formal_concept(arc.clone()))
    }
    pub fn all_concepts(&self) -> Vec<FormalConcept<A, B>> {
        self.all_concepts_par_iter().collect()
    }
}
