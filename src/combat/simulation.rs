use rayon::prelude::*;

use crate::combat::engine::resolve_combat;
use crate::combat::types::CombatResult;
use crate::data::models::{Unit, Weapon};

#[derive(Debug, Clone)]
pub struct Percentiles {
    pub p10: usize,
    pub p25: usize,
    pub p50: usize,
    pub p75: usize,
    pub p90: usize,
    pub mean: f64,
}

#[derive(Debug, Clone)]
pub struct PhaseSimulation {
    pub actual_value: usize,
    pub percentile: f64,
    #[allow(dead_code)]
    pub samples: Vec<usize>,
    pub percentiles: Percentiles,
}

#[derive(Debug, Clone)]
pub struct HistogramBin {
    pub value: usize,
    pub count: usize,
    #[allow(dead_code)]
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub hits_stats: PhaseSimulation,
    pub wounds_stats: PhaseSimulation,
    pub damage_stats: PhaseSimulation,
    pub histogram_bins: Vec<HistogramBin>,
}

/// Run `n_runs` Monte Carlo simulations and compute statistics.
#[allow(clippy::too_many_arguments)]
pub fn run_simulation(
    attacker: &Unit,
    defender: &Unit,
    weapon: &Weapon,
    num_models: usize,
    has_champion: bool,
    use_attack_override: bool,
    attack_override: usize,
    include_ward: bool,
    actual_result: &CombatResult,
    n_runs: usize,
) -> SimulationResult {
    // Run simulations in parallel
    let samples: Vec<(usize, usize, usize)> = (0..n_runs)
        .into_par_iter()
        .map(|_| {
            let result = resolve_combat(
                attacker,
                defender,
                weapon,
                num_models,
                has_champion,
                use_attack_override,
                attack_override,
                include_ward,
                false, // never stop early in simulation
                None,
            );
            (
                result.phases[0].total_output, // hits
                result.phases[1].total_output, // wounds
                result.final_damage,
            )
        })
        .collect();

    let mut hit_samples = Vec::with_capacity(n_runs);
    let mut wound_samples = Vec::with_capacity(n_runs);
    let mut damage_samples = Vec::with_capacity(n_runs);
    for (h, w, d) in samples {
        hit_samples.push(h);
        wound_samples.push(w);
        damage_samples.push(d);
    }

    SimulationResult {
        hits_stats: build_phase_simulation(
            &hit_samples,
            actual_result.phases[0].total_output,
        ),
        wounds_stats: build_phase_simulation(
            &wound_samples,
            actual_result.phases[1].total_output,
        ),
        damage_stats: build_phase_simulation(
            &damage_samples,
            actual_result.final_damage,
        ),
        histogram_bins: compute_histogram(&damage_samples),
    }
}

fn build_phase_simulation(samples: &[usize], actual_value: usize) -> PhaseSimulation {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    PhaseSimulation {
        actual_value,
        percentile: percentile_of_value(&sorted, actual_value),
        percentiles: compute_percentiles(&sorted),
        samples: sorted,
    }
}

fn compute_percentiles(sorted_samples: &[usize]) -> Percentiles {
    let len = sorted_samples.len();
    if len == 0 {
        return Percentiles {
            p10: 0,
            p25: 0,
            p50: 0,
            p75: 0,
            p90: 0,
            mean: 0.0,
        };
    }
    let mean = sorted_samples.iter().sum::<usize>() as f64 / len as f64;
    Percentiles {
        p10: percentile_value(sorted_samples, 0.10),
        p25: percentile_value(sorted_samples, 0.25),
        p50: percentile_value(sorted_samples, 0.50),
        p75: percentile_value(sorted_samples, 0.75),
        p90: percentile_value(sorted_samples, 0.90),
        mean,
    }
}

/// Get the value at a given percentile (e.g., 0.50 = median).
fn percentile_value(sorted_samples: &[usize], p: f64) -> usize {
    if sorted_samples.is_empty() {
        return 0;
    }
    let idx = ((sorted_samples.len() - 1) as f64 * p).round() as usize;
    sorted_samples[idx.clamp(0, sorted_samples.len() - 1)]
}

/// Compute what percentile a specific value falls into (0.0 to 1.0).
fn percentile_of_value(sorted_samples: &[usize], value: usize) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }
    let count_below = sorted_samples.iter().filter(|&&v| v < value).count();
    let count_equal = sorted_samples.iter().filter(|&&v| v == value).count();
    // Use midpoint of ties
    let rank = count_below + count_equal / 2;
    rank as f64 / sorted_samples.len() as f64
}

fn compute_histogram(samples: &[usize]) -> Vec<HistogramBin> {
    if samples.is_empty() {
        return Vec::new();
    }
    let max_val = *samples.iter().max().unwrap();
    let _min_val = *samples.iter().min().unwrap();
    let bin_size = if max_val > 30 {
        5
    } else if max_val > 15 {
        2
    } else {
        1
    };

    let mut counts = std::collections::BTreeMap::new();
    for &v in samples {
        let bin = (v / bin_size) * bin_size;
        *counts.entry(bin).or_insert(0) += 1;
    }

    let total = samples.len() as f64;
    counts
        .into_iter()
        .map(|(value, count)| HistogramBin {
            value,
            count,
            percentage: (count as f64 / total) * 100.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_value_basic() {
        let samples = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(percentile_value(&samples, 0.0), 1);
        assert_eq!(percentile_value(&samples, 0.5), 6);
        assert_eq!(percentile_value(&samples, 1.0), 10);
    }

    #[test]
    fn percentile_of_value_basic() {
        let samples = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert!(percentile_of_value(&samples, 5) >= 0.4);
        assert!(percentile_of_value(&samples, 5) <= 0.5);
    }

    #[test]
    fn histogram_bins_group_when_large() {
        let samples: Vec<usize> = (0..50).collect();
        let bins = compute_histogram(&samples);
        assert!(!bins.is_empty());
        // With max > 30, bin size should be 5
        assert!(bins.iter().all(|b| b.value % 5 == 0));
    }
}
