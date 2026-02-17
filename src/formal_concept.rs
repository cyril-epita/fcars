use crate::FormalContext;
use crate::bit_fiddling::*;

use bitvec::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FormalConcept<A = String, B = String> {
    pub context: Arc<FormalContext<A, B>>,
    pub data: RawFormalConcept,
}

/// A RawFormalConcept is optimized for efficiency -- it simply stores a pair of an extent and an intent.
#[derive(Debug, Clone)]
pub struct RawFormalConcept {
    pub extent: BitVec,
    pub intent: BitVec,
}

impl RawFormalConcept {
    /// Checks that the RawFormalConcept is actually a valid concept for the provided context.
    pub fn to_formal_concept<A, B>(self, context: Arc<FormalContext<A, B>>) -> FormalConcept<A, B> {
        assert_eq!(context.objects.len(), self.extent.len());
        assert_eq!(context.attributes.len(), self.intent.len());
        let result = FormalConcept {
            context,
            data: self,
        };
        assert!(result.validate());
        result
    }
}

impl<A: std::fmt::Debug, B: std::fmt::Debug> std::fmt::Display for FormalConcept<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let extent: Vec<_> = self.extent_names_iter().collect();
        let intent: Vec<_> = self.extent_names_iter().collect();
        write!(f, "Extent: {:?}, Intent: {:?}", extent, intent)
    }
}

impl<A, B> FormalConcept<A, B> {
    pub fn validate(&self) -> bool {
        self.data.extent == self.context.induce_l(&self.data.intent)
            && self.data.intent == self.context.induce_r(&self.data.extent)
    }
    pub fn extent_names_iter(&self) -> impl Iterator<Item = &A> {
        self.data
            .extent
            .iter_ones()
            .map(|i| &self.context.objects[i])
    }
    pub fn intent_names_iter(&self) -> impl Iterator<Item = &B> {
        self.data
            .intent
            .iter_ones()
            .map(|j| &self.context.attributes[j])
    }
}

impl<A: PartialEq, B: PartialEq> PartialEq for FormalConcept<A, B> {
    fn eq(&self, other: &Self) -> bool {
        *self.context == *other.context && self.data == other.data
    }
}

impl<A: Eq, B: Eq> Eq for FormalConcept<A, B> {}

impl<A: PartialEq, B: PartialEq> PartialOrd for FormalConcept<A, B> {
    // Concepts are ordered by subset containment of their extents, provided they are from the same context.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if *self.context != *other.context {
            return None; // Cannot compare concepts from different contexts
        }
        self.data.partial_cmp(&other.data)
    }
}

impl PartialEq for RawFormalConcept {
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent
    }
}

impl Eq for RawFormalConcept {}

impl PartialOrd for RawFormalConcept {
    // Concepts are ordered by subset containment of their extents.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.extent == other.extent {
            return Some(std::cmp::Ordering::Equal);
        }
        if is_subset(&self.extent, &other.extent) {
            return Some(std::cmp::Ordering::Less);
        }
        if is_subset(&other.extent, &self.extent) {
            return Some(std::cmp::Ordering::Greater);
        }
        None
    }
}
