use serde::Serialize;
use std::collections::BTreeSet;

use vcell_region_flow_000009::{
    fingerprint_distance, run_region_experiment, top_cells, RegionFingerprint, RegionMedium,
    VxnMicroSeed, DEFAULT_EXPERIMENT_STEPS,
};

pub const SCALE_CONTRACT: &str = "vertex.brain.vcell-scale-expansion.v1";
pub const SCALE_EXPERIMENT: &str = "VCELL_32C_SCALE_EXPANSION_000001";
pub const SCALE_GATE_TILTS: [f64; 7] = [-12.0, -8.0, -4.0, 0.0, 4.0, 8.0, 12.0];

#[derive(Debug, Clone, Serialize)]
pub struct ScaleCase {
    pub case_index: usize,
    pub gate_tilt_deg: f64,
    pub response_fingerprint: String,
    pub reached_cells: usize,
    pub reached_percent: f64,
    pub dominant_cell: u32,
    pub dominant_share: f64,
    pub top_cells: Vec<(u32, f64)>,
    pub shell_peak: [f64; 3],
    pub first_arrival_step: Option<u64>,
    pub last_arrival_step: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalePair {
    pub a: usize,
    pub b: usize,
    pub tilt_a: f64,
    pub tilt_b: f64,
    pub response_distance: f64,
    pub dominant_cell_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScaleExpansionReport {
    pub schema: &'static str,
    pub experiment_id: &'static str,
    pub state: &'static str,
    pub classification: &'static str,
    pub current_resident_scope: &'static str,
    pub candidate_scope: &'static str,
    pub resident_promotion_state: &'static str,
    pub four_c_reference_fingerprint: String,
    pub seed_hex: String,
    pub seed_bytes: usize,
    pub cells: usize,
    pub scale_multiplier_vs_4c: f64,
    pub vertices: usize,
    pub intra_cell_bonds: usize,
    pub links: usize,
    pub contact_points: usize,
    pub root_degree: usize,
    pub shell_counts: [usize; 3],
    pub experiment_steps: usize,
    pub case_count: usize,
    pub pair_count: usize,
    pub unique_response_fingerprints: usize,
    pub unique_dominant_cells: usize,
    pub minimum_response_distance: f64,
    pub maximum_response_distance: f64,
    pub mean_response_distance: f64,
    pub baseline_to_plus8_distance: f64,
    pub minus8_to_plus8_distance: f64,
    pub min_reached_cells: usize,
    pub max_reached_cells: usize,
    pub all_cases_reach_region: bool,
    pub response_diversity_observed: bool,
    pub cases: Vec<ScaleCase>,
    pub pairwise: Vec<ScalePair>,
    pub cross_scale_numeric_equivalence_claimed: bool,
    pub carrier_note: &'static str,
    pub llm_ingestion_state: &'static str,
    pub provider_state: &'static str,
    pub laa_state: &'static str,
    pub transfer_state: &'static str,
    pub next_gate: &'static str,
    pub interpretation: &'static str,
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn response_fingerprint(f: &RegionFingerprint) -> String {
    // Stimulus geometry is not encoded here.
    // Fingerprint identity comes from the observed 32C response only.
    let mut material = String::new();

    for value in &f.normalized_share {
        material.push_str(&format!("{value:.12};"));
    }
    material.push('|');

    for value in &f.shell_peak {
        material.push_str(&format!("{value:.12};"));
    }
    material.push('|');

    for arrival in &f.arrival_step {
        match arrival {
            Some(step) => material.push_str(&format!("{step};")),
            None => material.push_str("N;"),
        }
    }

    format!("V32S1-{:016X}", fnv1a64(material.as_bytes()))
}

fn arrival_bounds(f: &RegionFingerprint) -> (Option<u64>, Option<u64>) {
    let mut steps: Vec<u64> = f.arrival_step.iter().flatten().copied().collect();
    if steps.is_empty() {
        return (None, None);
    }
    steps.sort_unstable();
    (steps.first().copied(), steps.last().copied())
}

fn build_case(case_index: usize, gate_tilt_deg: f64, f: &RegionFingerprint) -> ScaleCase {
    let top = top_cells(f, 8);
    let (dominant_cell, dominant_share) = top.first().copied().unwrap_or((0, 0.0));
    let (first_arrival_step, last_arrival_step) = arrival_bounds(f);

    ScaleCase {
        case_index,
        gate_tilt_deg,
        response_fingerprint: response_fingerprint(f),
        reached_cells: f.reached_cells,
        reached_percent: f.reached_cells as f64 * 100.0 / 32.0,
        dominant_cell,
        dominant_share,
        top_cells: top,
        shell_peak: f.shell_peak,
        first_arrival_step,
        last_arrival_step,
    }
}

pub fn run_scale_expansion(
    four_c_reference_fingerprint: String,
) -> Result<ScaleExpansionReport, String> {
    let seed = VxnMicroSeed::default();
    let medium = RegionMedium::new(0.0)?;
    let metrics = medium.region.metrics()?;

    if metrics.cell_count != 32 {
        return Err(format!(
            "32C scale candidate expected 32 cells, found {}",
            metrics.cell_count
        ));
    }
    if medium.nodes.len() != 768 {
        return Err(format!(
            "32C scale candidate expected 768 vertices, found {}",
            medium.nodes.len()
        ));
    }
    if seed.encode().len() != 3 {
        return Err("32C scale candidate must retain the 3-byte VXN micro-seed".into());
    }

    let mut fingerprints = Vec::with_capacity(SCALE_GATE_TILTS.len());
    let mut cases = Vec::with_capacity(SCALE_GATE_TILTS.len());

    for (case_index, tilt) in SCALE_GATE_TILTS.iter().copied().enumerate() {
        let f = run_region_experiment(tilt, seed, DEFAULT_EXPERIMENT_STEPS)?;
        cases.push(build_case(case_index, tilt, &f));
        fingerprints.push(f);
    }

    let mut pairwise = Vec::new();
    for a in 0..fingerprints.len() {
        for b in (a + 1)..fingerprints.len() {
            pairwise.push(ScalePair {
                a,
                b,
                tilt_a: SCALE_GATE_TILTS[a],
                tilt_b: SCALE_GATE_TILTS[b],
                response_distance: fingerprint_distance(&fingerprints[a], &fingerprints[b]),
                dominant_cell_changed: cases[a].dominant_cell != cases[b].dominant_cell,
            });
        }
    }

    let pair_count = pairwise.len();
    let minimum_response_distance = pairwise
        .iter()
        .map(|p| p.response_distance)
        .fold(f64::INFINITY, f64::min);
    let maximum_response_distance = pairwise
        .iter()
        .map(|p| p.response_distance)
        .fold(0.0_f64, f64::max);
    let mean_response_distance =
        pairwise.iter().map(|p| p.response_distance).sum::<f64>() / pair_count as f64;

    let mut response_ids: Vec<String> = cases
        .iter()
        .map(|c| c.response_fingerprint.clone())
        .collect();
    response_ids.sort();
    response_ids.dedup();

    let dominant_cells: BTreeSet<u32> = cases.iter().map(|c| c.dominant_cell).collect();
    let min_reached_cells = cases.iter().map(|c| c.reached_cells).min().unwrap_or(0);
    let max_reached_cells = cases.iter().map(|c| c.reached_cells).max().unwrap_or(0);

    let zero = SCALE_GATE_TILTS
        .iter()
        .position(|v| v.abs() < 1.0e-12)
        .ok_or_else(|| "zero gate case missing".to_string())?;
    let plus8 = SCALE_GATE_TILTS
        .iter()
        .position(|v| (*v - 8.0).abs() < 1.0e-12)
        .ok_or_else(|| "+8 gate case missing".to_string())?;
    let minus8 = SCALE_GATE_TILTS
        .iter()
        .position(|v| (*v + 8.0).abs() < 1.0e-12)
        .ok_or_else(|| "-8 gate case missing".to_string())?;

    let baseline_to_plus8_distance =
        fingerprint_distance(&fingerprints[zero], &fingerprints[plus8]);
    let minus8_to_plus8_distance =
        fingerprint_distance(&fingerprints[minus8], &fingerprints[plus8]);

    let all_cases_reach_region = min_reached_cells == 32;
    let response_diversity_observed = response_ids.len() > 1 && maximum_response_distance > 1.0e-12;

    let state = if response_diversity_observed
        && metrics.cell_count == 32
        && medium.nodes.len() == 768
        && pair_count == 21
    {
        "OBSERVED"
    } else {
        "OBSERVE"
    };

    let classification = if all_cases_reach_region && response_diversity_observed {
        "32C_GLOBAL_FLOW_WITH_GEOMETRY_RESPONSE"
    } else if response_diversity_observed {
        "32C_PARTIAL_REACH_WITH_GEOMETRY_RESPONSE"
    } else {
        "32C_RESPONSE_COLLAPSED"
    };

    Ok(ScaleExpansionReport {
        schema: SCALE_CONTRACT,
        experiment_id: SCALE_EXPERIMENT,
        state,
        classification,
        current_resident_scope: "4C_REFERENCE_CORE",
        candidate_scope: "32C_REGION_CANDIDATE",
        resident_promotion_state: "CANDIDATE_ONLY",
        four_c_reference_fingerprint,
        seed_hex: seed.hex(),
        seed_bytes: seed.encode().len(),
        cells: metrics.cell_count,
        scale_multiplier_vs_4c: metrics.cell_count as f64 / 4.0,
        vertices: medium.nodes.len(),
        intra_cell_bonds: metrics.cell_count * 36,
        links: metrics.link_count,
        contact_points: metrics.contact_points,
        root_degree: metrics.root_degree,
        shell_counts: metrics.shell_counts,
        experiment_steps: DEFAULT_EXPERIMENT_STEPS,
        case_count: cases.len(),
        pair_count,
        unique_response_fingerprints: response_ids.len(),
        unique_dominant_cells: dominant_cells.len(),
        minimum_response_distance,
        maximum_response_distance,
        mean_response_distance,
        baseline_to_plus8_distance,
        minus8_to_plus8_distance,
        min_reached_cells,
        max_reached_cells,
        all_cases_reach_region,
        response_diversity_observed,
        cases,
        pairwise,
        cross_scale_numeric_equivalence_claimed: false,
        carrier_note: "32C REGION FLOW 000009 uses the FIRST_WAVE carrier marked provisional; 4C and 32C response vectors are therefore not declared numerically equivalent in this experiment.",
        llm_ingestion_state: "LOCKED",
        provider_state: "STANDBY_ONLY",
        laa_state: "OBSERVE_ONLY",
        transfer_state: "CANDIDATE_GATED",
        next_gate: "32C_RESIDENT_PROMOTION",
        interpretation: "Scale Expansion 000001 verifies that the existing 32C real-contact region can be observed from BrainSystem with the same 3-byte VXN and can express geometry-dependent response diversity. It does not yet replace the verified 4C resident core.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_fingerprint_is_output_only() {
        let seed = VxnMicroSeed::default();
        let f = run_region_experiment(0.0, seed, 64).unwrap();
        let a = response_fingerprint(&f);
        let b = response_fingerprint(&f);
        assert_eq!(a, b);
        assert!(a.starts_with("V32S1-"));
    }

    #[test]
    fn real_region_is_32c_768_vertices_and_three_byte_vxn() {
        let seed = VxnMicroSeed::default();
        let medium = RegionMedium::new(0.0).unwrap();
        assert_eq!(medium.cells.len(), 32);
        assert_eq!(medium.nodes.len(), 768);
        assert_eq!(seed.encode().len(), 3);
    }

    #[test]
    fn local_gate_changes_32c_response_fingerprint() {
        let seed = VxnMicroSeed::default();
        let a = run_region_experiment(0.0, seed, DEFAULT_EXPERIMENT_STEPS).unwrap();
        let b = run_region_experiment(8.0, seed, DEFAULT_EXPERIMENT_STEPS).unwrap();
        assert!(fingerprint_distance(&a, &b) > 1.0e-12);
        assert_ne!(response_fingerprint(&a), response_fingerprint(&b));
    }

    #[test]
    fn same_32c_condition_replays_exactly() {
        let seed = VxnMicroSeed::default();
        let a = run_region_experiment(4.0, seed, 256).unwrap();
        let b = run_region_experiment(4.0, seed, 256).unwrap();
        assert_eq!(response_fingerprint(&a), response_fingerprint(&b));
        assert_eq!(a.normalized_share, b.normalized_share);
        assert_eq!(a.arrival_step, b.arrival_step);
    }

    #[test]
    fn scale_report_keeps_4c_resident_and_llm_locked() {
        let r = run_scale_expansion("VRB1-REFERENCE".into()).unwrap();
        assert_eq!(r.state, "OBSERVED");
        assert_eq!(r.current_resident_scope, "4C_REFERENCE_CORE");
        assert_eq!(r.candidate_scope, "32C_REGION_CANDIDATE");
        assert_eq!(r.resident_promotion_state, "CANDIDATE_ONLY");
        assert_eq!(r.cells, 32);
        assert_eq!(r.vertices, 768);
        assert_eq!(r.case_count, 7);
        assert_eq!(r.pair_count, 21);
        assert!(r.response_diversity_observed);
        assert!(!r.cross_scale_numeric_equivalence_claimed);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
        assert_eq!(r.next_gate, "32C_RESIDENT_PROMOTION");
    }
}
