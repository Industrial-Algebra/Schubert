// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Schubert routing — geometric network path computation.
//!
//! Models network routing where route advertisements are capability grants
//! and forwarding decisions are Schubert intersections. The intersection
//! number gives the exact count of valid routes, and codimension excess
//! detects congestion.
//!
//! # How It Works
//!
//! 1. Each node advertises capabilities (Schubert conditions)
//! 2. A path from A→B requires intersecting A's outgoing caps with B's incoming caps
//! 3. The intersection number = number of valid paths
//! 4. If total codimension exceeds the Grassmannian dimension, the path is congested
//!
//! # Example
//!
//! ```
//! use schubert::routing::{RouteTable, RouteAdvertisement};
//!
//! let mut rt = RouteTable::new(2, 4)?;
//! rt.advertise("nodeA", vec![1])?;
//! rt.advertise("nodeB", vec![1])?;
//!
//! let result = rt.check_route("nodeA", "nodeB")?;
//! // result.path_count — number of valid routes
//! // result.is_congested — whether total codimension exceeds dimension
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use std::collections::HashMap;

use crate::{
    error::{Result, SchubertError},
    ComputationPath,
};
use amari_enumerative::{IntersectionResult, SchubertCalculus, SchubertClass};

/// A route advertisement — a node announcing a capability.
#[derive(Debug, Clone)]
pub struct RouteAdvertisement {
    /// The node making the announcement.
    pub node_id: String,
    /// Schubert partition defining the route condition.
    pub partition: Vec<usize>,
    /// Number of hops (for path cost).
    pub hop_count: u32,
}

/// A routing table managing route advertisements and path computation.
///
/// Operates in a fixed Grassmannian Gr(k,n). Each node advertises
/// capabilities (Schubert conditions). Path computation uses Schubert
/// intersection to determine reachability and path count.
#[derive(Debug)]
pub struct RouteTable {
    grassmannian: (usize, usize),
    /// Node → list of advertised partitions.
    advertisements: HashMap<String, Vec<Vec<usize>>>,
}

/// Result of a route check between two nodes.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// Number of valid paths (intersection number).
    pub path_count: u64,
    /// Total codimension of all route conditions.
    pub total_codimension: usize,
    /// Whether the path is overconstrained (codim > dim).
    pub is_congested: bool,
    /// Dimension of the solution variety if underconstrained.
    pub solution_dimension: Option<usize>,
    /// Which computation path was used.
    pub path: ComputationPath,
}

impl RouteTable {
    /// Create a new routing table for Gr(k,n).
    pub fn new(k: usize, n: usize) -> Result<Self> {
        if k == 0 || n < 2 || k >= n {
            return Err(SchubertError::InvalidGrassmannian {
                k,
                n,
                reason: "require k ≥ 1, n ≥ 2, k < n".into(),
            });
        }
        Ok(Self {
            grassmannian: (k, n),
            advertisements: HashMap::new(),
        })
    }

    /// Return the Grassmannian parameters.
    pub fn grassmannian(&self) -> (usize, usize) {
        self.grassmannian
    }

    /// Advertise a route from a node.
    ///
    /// Each advertisement is a Schubert condition. Multiple advertisements
    /// from the same node are combined via intersection.
    pub fn advertise(&mut self, node_id: impl Into<String>, partition: Vec<usize>) -> Result<()> {
        let node_id = node_id.into();
        // Validate partition
        SchubertClass::new(partition.clone(), self.grassmannian).map_err(|e| {
            SchubertError::InvalidPartition {
                partition: partition.clone(),
                k: self.grassmannian.0,
                n: self.grassmannian.1,
                reason: e.to_string(),
            }
        })?;

        self.advertisements
            .entry(node_id)
            .or_default()
            .push(partition);
        Ok(())
    }

    /// Advertise multiple partitions at once.
    pub fn advertise_many(
        &mut self,
        node_id: impl Into<String>,
        partitions: Vec<Vec<usize>>,
    ) -> Result<()> {
        let node_id = node_id.into();
        for p in partitions {
            self.advertise(&node_id, p)?;
        }
        Ok(())
    }

    /// Get the partitions advertised by a node.
    pub fn advertisements_for(&self, node_id: &str) -> Option<&[Vec<usize>]> {
        self.advertisements.get(node_id).map(|v| v.as_slice())
    }

    /// Check if a route exists from source to destination.
    ///
    /// Computes the Schubert intersection of the source's outgoing
    /// advertisements with the destination's incoming advertisements.
    /// The intersection number is the count of valid paths.
    pub fn check_route(&self, source: &str, destination: &str) -> Result<RouteResult> {
        let source_caps = self
            .advertisements
            .get(source)
            .ok_or_else(|| SchubertError::Generic(format!("source '{source}' not found")))?;

        let dest_caps = self.advertisements.get(destination).ok_or_else(|| {
            SchubertError::Generic(format!("destination '{destination}' not found"))
        })?;

        // Build Schubert classes for source
        let mut all_classes = Vec::new();
        for partition in source_caps {
            all_classes.push(
                SchubertClass::new(partition.clone(), self.grassmannian).map_err(|e| {
                    SchubertError::InvalidPartition {
                        partition: partition.clone(),
                        k: self.grassmannian.0,
                        n: self.grassmannian.1,
                        reason: e.to_string(),
                    }
                })?,
            );
        }

        // Add destination classes
        for partition in dest_caps {
            all_classes.push(
                SchubertClass::new(partition.clone(), self.grassmannian).map_err(|e| {
                    SchubertError::InvalidPartition {
                        partition: partition.clone(),
                        k: self.grassmannian.0,
                        n: self.grassmannian.1,
                        reason: e.to_string(),
                    }
                })?,
            );
        }

        let total_codim: usize = all_classes.iter().map(|c| c.codimension()).sum();
        let dim = self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0);
        let is_congested = total_codim > dim;

        // Compute intersection
        let mut calc = SchubertCalculus::new(self.grassmannian);
        let result = calc.multi_intersect(&all_classes);

        match result {
            IntersectionResult::Finite(n) => Ok(RouteResult {
                path_count: n,
                total_codimension: total_codim,
                is_congested,
                solution_dimension: None,
                path: ComputationPath::LittlewoodRichardson,
            }),
            IntersectionResult::PositiveDimensional { dimension, .. } => Ok(RouteResult {
                path_count: 0, // infinite paths technically
                total_codimension: total_codim,
                is_congested,
                solution_dimension: Some(dimension),
                path: ComputationPath::LittlewoodRichardson,
            }),
            IntersectionResult::Empty => Ok(RouteResult {
                path_count: 0,
                total_codimension: total_codim,
                is_congested: true,
                solution_dimension: None,
                path: ComputationPath::LittlewoodRichardson,
            }),
        }
    }

    /// Check if a multi-hop path exists through intermediate nodes.
    ///
    /// Computes the intersection of all advertised partitions along the
    /// path: source → hop1 → hop2 → ... → destination.
    pub fn check_path(&self, nodes: &[&str]) -> Result<RouteResult> {
        if nodes.len() < 2 {
            return Err(SchubertError::Generic(
                "path must have at least 2 nodes".into(),
            ));
        }

        let mut all_classes = Vec::new();
        for node in nodes {
            let caps = self
                .advertisements
                .get(*node)
                .ok_or_else(|| SchubertError::Generic(format!("node '{node}' not found")))?;

            for partition in caps {
                all_classes.push(
                    SchubertClass::new(partition.clone(), self.grassmannian).map_err(|e| {
                        SchubertError::InvalidPartition {
                            partition: partition.clone(),
                            k: self.grassmannian.0,
                            n: self.grassmannian.1,
                            reason: e.to_string(),
                        }
                    })?,
                );
            }
        }

        let total_codim: usize = all_classes.iter().map(|c| c.codimension()).sum();
        let dim = self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0);
        let is_congested = total_codim > dim;

        let mut calc = SchubertCalculus::new(self.grassmannian);
        let result = calc.multi_intersect(&all_classes);

        match result {
            IntersectionResult::Finite(n) => Ok(RouteResult {
                path_count: n,
                total_codimension: total_codim,
                is_congested,
                solution_dimension: None,
                path: ComputationPath::LittlewoodRichardson,
            }),
            IntersectionResult::PositiveDimensional { dimension, .. } => Ok(RouteResult {
                path_count: 0,
                total_codimension: total_codim,
                is_congested,
                solution_dimension: Some(dimension),
                path: ComputationPath::LittlewoodRichardson,
            }),
            IntersectionResult::Empty => Ok(RouteResult {
                path_count: 0,
                total_codimension: total_codim,
                is_congested: true,
                solution_dimension: None,
                path: ComputationPath::LittlewoodRichardson,
            }),
        }
    }

    /// List all nodes in the routing table.
    pub fn nodes(&self) -> Vec<&str> {
        self.advertisements.keys().map(|s| s.as_str()).collect()
    }

    /// Get the congestion level for a node (codimension relative to dimension).
    ///
    /// Returns a value > 1.0 if congested, < 1.0 if underconstrained.
    pub fn congestion_level(&self, node_id: &str) -> Option<f64> {
        let caps = self.advertisements.get(node_id)?;
        let total_codim: usize = caps.iter().map(|p| p.iter().sum::<usize>()).sum();
        let dim = self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0);
        if dim == 0 {
            None
        } else {
            Some(total_codim as f64 / dim as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_hop_route() {
        let mut rt = RouteTable::new(2, 4).unwrap();
        rt.advertise("A", vec![1]).unwrap();
        rt.advertise("B", vec![1]).unwrap();

        let result = rt.check_route("A", "B").unwrap();
        // σ₁ · σ₁ = σ₂ + σ₁₁ → positive dimensional
        assert!(result.solution_dimension.is_some());
        assert!(!result.is_congested);
    }

    #[test]
    fn congested_route() {
        let mut rt = RouteTable::new(2, 4).unwrap();
        rt.advertise("A", vec![2]).unwrap();
        rt.advertise("A", vec![2, 1]).unwrap();
        rt.advertise("A", vec![2, 2]).unwrap();
        rt.advertise("B", vec![1]).unwrap();

        // codim: 2 + 3 + 4 + 1 = 10 > 4 → congested
        let result = rt.check_route("A", "B").unwrap();
        assert!(result.is_congested);
        assert_eq!(result.path_count, 0);
    }

    #[test]
    fn impossible_route() {
        let mut rt = RouteTable::new(2, 4).unwrap();
        rt.advertise("A", vec![2]).unwrap(); // σ₂
        rt.advertise("B", vec![1, 1]).unwrap(); // σ₁₁

        // σ₂ · σ₁₁ = 0 in Gr(2,4) → impossible
        let result = rt.check_route("A", "B").unwrap();
        assert_eq!(result.path_count, 0);
        assert!(!result.is_congested);
    }

    #[test]
    fn multi_hop_path() {
        let mut rt = RouteTable::new(2, 4).unwrap();
        rt.advertise("A", vec![1]).unwrap();
        rt.advertise("B", vec![1]).unwrap();
        rt.advertise("C", vec![1]).unwrap();

        let result = rt.check_path(&["A", "B", "C"]).unwrap();
        // σ₁³ — multiple σ₁ intersections
        assert!(result.solution_dimension.is_some());
    }

    #[test]
    fn congestion_level_check() {
        let mut rt = RouteTable::new(2, 4).unwrap();
        rt.advertise("A", vec![1]).unwrap(); // codim 1, dim 4 → 0.25

        let level = rt.congestion_level("A").unwrap();
        assert!((level - 0.25).abs() < 0.01);

        rt.advertise("A", vec![2, 2]).unwrap(); // +4 = 5 total → 1.25
        let level = rt.congestion_level("A").unwrap();
        assert!((level - 1.25).abs() < 0.01);
    }

    #[test]
    fn missing_node_error() {
        let rt = RouteTable::new(2, 4).unwrap();
        assert!(rt.check_route("A", "B").is_err());
    }

    #[test]
    fn nodes_list() {
        let mut rt = RouteTable::new(2, 4).unwrap();
        rt.advertise("A", vec![1]).unwrap();
        rt.advertise("B", vec![1]).unwrap();
        rt.advertise("C", vec![1]).unwrap();

        let nodes = rt.nodes();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains(&"A"));
        assert!(nodes.contains(&"B"));
        assert!(nodes.contains(&"C"));
    }
}
