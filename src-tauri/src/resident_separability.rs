use serde::Serialize;

pub const SEPARABILITY_CONTRACT: &str = "vertex.brain.vcell-response-separability.v1";
pub const SEPARABILITY_EXPERIMENT: &str = "VCELL_4C_RESPONSE_SEPARABILITY_000001";
pub const DEFAULT_SEPARATION_THRESHOLD: f64 = 0.08;

#[derive(Debug, Clone)]
pub struct ResponseCaseInput {
    pub onset_step: usize,
    pub end_step: usize,
    pub first_trace_step: usize,
    pub trace_at_return_l1: f64,
    pub peak_trace_l1: f64,
    pub peak_trace_step: usize,
    pub final_trace_l1: f64,
    pub final_max_state_delta: f64,
    pub final_share_distance: f64,
    pub dominant_destination: u32,
    pub geometry_restored: bool,
}

#[derive(Debug, Clone)]
pub struct SeparabilityInput {
    pub baseline_fingerprint: String,
    pub baseline_state: String,
    pub cell_count: usize,
    pub seed_hex: String,
    pub seed_bytes: usize,
    pub response_cases: Vec<ResponseCaseInput>,
    pub time_gate_controls_clean: bool,
    pub geometry_egress_distance: f64,
    pub geometry_equivalence_max_share_delta: f64,
    pub self_reconfiguration_observed: bool,
    pub recovery_observed: bool,
    pub temporal_path_dependence_observed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseSignature {
    pub case_index: usize,
    pub onset_step: usize,
    pub response_fingerprint: String,
    pub first_trace_step: usize,
    pub peak_trace_step: usize,
    pub dominant_destination: u32,
    pub trace_at_return_l1: f64,
    pub peak_trace_l1: f64,
    pub final_trace_l1: f64,
    pub final_max_state_delta: f64,
    pub final_share_distance: f64,
    pub normalized_vector: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairDistance {
    pub a: usize,
    pub b: usize,
    pub onset_a: usize,
    pub onset_b: usize,
    pub distance: f64,
    pub separated: bool,
    pub destination_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AxisObservation {
    pub axis: &'static str,
    pub state: &'static str,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseSeparabilityReport {
    pub schema: &'static str,
    pub experiment_id: &'static str,
    pub state: &'static str,
    pub classification: &'static str,
    pub scope: &'static str,

    pub baseline_fingerprint: String,
    pub baseline_state: String,
    pub cell_count: usize,
    pub seed_hex: String,
    pub seed_bytes: usize,

    pub threshold: f64,
    pub threshold_semantics: &'static str,
    pub signature_count: usize,
    pub pair_count: usize,
    pub separated_pair_count: usize,
    pub separability_ratio_percent: f64,
    pub cluster_count: usize,

    pub minimum_pair_distance: f64,
    pub maximum_pair_distance: f64,
    pub mean_pair_distance: f64,
    pub closest_pair: [usize; 2],
    pub closest_onsets: [usize; 2],

    pub unique_response_fingerprints: usize,
    pub unique_dominant_destinations: usize,
    pub all_geometry_restored: bool,
    pub controls_clean: bool,

    pub axes: Vec<AxisObservation>,
    pub signatures: Vec<ResponseSignature>,
    pub pairwise: Vec<PairDistance>,

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

fn response_fingerprint(c: &ResponseCaseInput) -> String {
    // Deliberately excludes onset/end stimulus descriptors.
    // The identity describes the observed response, not the condition label.
    let material = format!(
        concat!(
            "first={};return={:0.12};peak={:0.12};peak_step={};",
            "final={:0.12};max={:0.12};share={:0.12};dest={};restored={}"
        ),
        c.first_trace_step,
        c.trace_at_return_l1,
        c.peak_trace_l1,
        c.peak_trace_step,
        c.final_trace_l1,
        c.final_max_state_delta,
        c.final_share_distance,
        c.dominant_destination,
        c.geometry_restored,
    );
    format!("VRS1-{:016X}", fnv1a64(material.as_bytes()))
}

fn normalized(value: f64, min: f64, max: f64) -> f64 {
    let range = max - min;
    if range.abs() < 1.0e-15 {
        0.0
    } else {
        (value - min) / range
    }
}

fn numeric_features(c: &ResponseCaseInput) -> [f64; 7] {
    [
        c.first_trace_step as f64,
        c.trace_at_return_l1,
        c.peak_trace_l1,
        c.peak_trace_step as f64,
        c.final_trace_l1,
        c.final_max_state_delta,
        c.final_share_distance,
    ]
}

fn response_distance(a: &ResponseSignature, b: &ResponseSignature) -> f64 {
    let mut sum = 0.0;
    for (x, y) in a.normalized_vector.iter().zip(b.normalized_vector.iter()) {
        let d = x - y;
        sum += d * d;
    }
    let destination_delta = if a.dominant_destination == b.dominant_destination {
        0.0
    } else {
        1.0
    };
    sum += destination_delta * destination_delta;
    (sum / 8.0).sqrt()
}

fn cluster_count(signatures: &[ResponseSignature], pairs: &[PairDistance]) -> usize {
    if signatures.is_empty() {
        return 0;
    }
    let mut parent: Vec<usize> = (0..signatures.len()).collect();

    fn root(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            let r = root(parent, parent[x]);
            parent[x] = r;
        }
        parent[x]
    }

    for p in pairs {
        if !p.separated {
            let ra = root(&mut parent, p.a);
            let rb = root(&mut parent, p.b);
            if ra != rb {
                parent[rb] = ra;
            }
        }
    }

    let mut roots = Vec::new();
    for i in 0..parent.len() {
        let r = root(&mut parent, i);
        if !roots.contains(&r) {
            roots.push(r);
        }
    }
    roots.len()
}

pub fn analyze_response_separability(
    input: SeparabilityInput,
    threshold: f64,
) -> Result<ResponseSeparabilityReport, String> {
    if input.response_cases.len() < 2 {
        return Err("response separability requires at least two response cases".into());
    }
    if !(0.0 < threshold && threshold < 1.0) {
        return Err("separation threshold must be between 0 and 1".into());
    }

    let mut mins = [f64::INFINITY; 7];
    let mut maxs = [f64::NEG_INFINITY; 7];
    for c in &input.response_cases {
        for (i, v) in numeric_features(c).iter().enumerate() {
            mins[i] = mins[i].min(*v);
            maxs[i] = maxs[i].max(*v);
        }
    }

    let signatures: Vec<ResponseSignature> = input
        .response_cases
        .iter()
        .enumerate()
        .map(|(case_index, c)| {
            let raw = numeric_features(c);
            let vector = raw
                .iter()
                .enumerate()
                .map(|(i, v)| normalized(*v, mins[i], maxs[i]))
                .collect();
            ResponseSignature {
                case_index,
                onset_step: c.onset_step,
                response_fingerprint: response_fingerprint(c),
                first_trace_step: c.first_trace_step,
                peak_trace_step: c.peak_trace_step,
                dominant_destination: c.dominant_destination,
                trace_at_return_l1: c.trace_at_return_l1,
                peak_trace_l1: c.peak_trace_l1,
                final_trace_l1: c.final_trace_l1,
                final_max_state_delta: c.final_max_state_delta,
                final_share_distance: c.final_share_distance,
                normalized_vector: vector,
            }
        })
        .collect();

    let mut pairwise = Vec::new();
    for a in 0..signatures.len() {
        for b in (a + 1)..signatures.len() {
            let distance = response_distance(&signatures[a], &signatures[b]);
            pairwise.push(PairDistance {
                a,
                b,
                onset_a: signatures[a].onset_step,
                onset_b: signatures[b].onset_step,
                distance,
                separated: distance >= threshold,
                destination_changed: signatures[a].dominant_destination
                    != signatures[b].dominant_destination,
            });
        }
    }

    let pair_count = pairwise.len();
    let separated_pair_count = pairwise.iter().filter(|p| p.separated).count();
    let separability_ratio_percent = separated_pair_count as f64 * 100.0 / pair_count as f64;

    let minimum_pair = pairwise
        .iter()
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
        .ok_or_else(|| "pair matrix unexpectedly empty".to_string())?;
    let maximum_pair_distance = pairwise.iter().map(|p| p.distance).fold(0.0_f64, f64::max);
    let mean_pair_distance = pairwise.iter().map(|p| p.distance).sum::<f64>() / pair_count as f64;

    let clusters = cluster_count(&signatures, &pairwise);

    let mut fingerprints: Vec<String> = signatures
        .iter()
        .map(|s| s.response_fingerprint.clone())
        .collect();
    fingerprints.sort();
    fingerprints.dedup();

    let mut destinations: Vec<u32> = signatures.iter().map(|s| s.dominant_destination).collect();
    destinations.sort();
    destinations.dedup();

    let all_geometry_restored = input.response_cases.iter().all(|c| c.geometry_restored);
    let geometry_observed = input.geometry_egress_distance > 1.0e-9
        && input.geometry_equivalence_max_share_delta < 1.0e-12;
    let temporal_observed = clusters >= 2
        && separated_pair_count > 0
        && destinations.len() >= 2
        && input.time_gate_controls_clean;
    let history_observed = input.temporal_path_dependence_observed
        && all_geometry_restored
        && input.time_gate_controls_clean;
    let controls_clean = input.time_gate_controls_clean
        && all_geometry_restored
        && input.baseline_state == "CALIBRATED";

    let classification = if separated_pair_count == pair_count {
        "FULLY_SEPARATED"
    } else if clusters >= 2 {
        "PARTIALLY_SEPARATED"
    } else {
        "COLLAPSED"
    };

    let state = if controls_clean && geometry_observed && temporal_observed {
        "OBSERVED"
    } else {
        "OBSERVE"
    };

    Ok(ResponseSeparabilityReport {
        schema: SEPARABILITY_CONTRACT,
        experiment_id: SEPARABILITY_EXPERIMENT,
        state,
        classification,
        scope: "4C_REFERENCE_CORE",

        baseline_fingerprint: input.baseline_fingerprint,
        baseline_state: input.baseline_state,
        cell_count: input.cell_count,
        seed_hex: input.seed_hex,
        seed_bytes: input.seed_bytes,

        threshold,
        threshold_semantics: "ANALYSIS_THRESHOLD / NORMALIZED_RESPONSE_SPACE / NOT_PHYSICAL_LAW",
        signature_count: signatures.len(),
        pair_count,
        separated_pair_count,
        separability_ratio_percent,
        cluster_count: clusters,

        minimum_pair_distance: minimum_pair.distance,
        maximum_pair_distance,
        mean_pair_distance,
        closest_pair: [minimum_pair.a, minimum_pair.b],
        closest_onsets: [minimum_pair.onset_a, minimum_pair.onset_b],

        unique_response_fingerprints: fingerprints.len(),
        unique_dominant_destinations: destinations.len(),
        all_geometry_restored,
        controls_clean,

        axes: vec![
            AxisObservation {
                axis: "TIME",
                state: if temporal_observed { "SEPARABLE_RESPONSE_OBSERVED" } else { "OBSERVE" },
                evidence: format!(
                    "{} signatures / {} clusters / {:0.3}% separated pairs / {} destinations",
                    signatures.len(),
                    clusters,
                    separability_ratio_percent,
                    destinations.len()
                ),
            },
            AxisObservation {
                axis: "GEOMETRY",
                state: if geometry_observed { "RESPONSE_DELTA_OBSERVED" } else { "OBSERVE" },
                evidence: format!(
                    "egress_distance={:0.12}; equivalence_delta={:0.12}",
                    input.geometry_egress_distance,
                    input.geometry_equivalence_max_share_delta
                ),
            },
            AxisObservation {
                axis: "HISTORY",
                state: if history_observed {
                    "PATH_DEPENDENCE_OBSERVED_NOT_MEMORY"
                } else {
                    "OBSERVE"
                },
                evidence: format!(
                    "same_final_geometry={}; deterministic_controls={}; self_reconfiguration={}; recovery={}",
                    all_geometry_restored,
                    input.time_gate_controls_clean,
                    input.self_reconfiguration_observed,
                    input.recovery_observed
                ),
            },
            AxisObservation {
                axis: "CONTACT",
                state: "NOT_ISOLATED_IN_000001",
                evidence: "Contact participates in the resident topology but is not independently perturbed in this experiment.".into(),
            },
        ],
        signatures,
        pairwise,

        llm_ingestion_state: "LOCKED",
        provider_state: "STANDBY_ONLY",
        laa_state: "OBSERVE_ONLY",
        transfer_state: "CANDIDATE_GATED",
        next_gate: "32C_SCALE_EXPANSION",
        interpretation: "Response identity is inferred from observed output trajectories only; stimulus onset is retained as a label but excluded from response fingerprinting. Separability is analytical, thresholded, and not an intelligence score.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cases() -> Vec<ResponseCaseInput> {
        vec![
            ResponseCaseInput {
                onset_step: 4,
                end_step: 10,
                first_trace_step: 4,
                trace_at_return_l1: 0.01,
                peak_trace_l1: 0.02,
                peak_trace_step: 8,
                final_trace_l1: 0.004972,
                final_max_state_delta: 0.001,
                final_share_distance: 0.001,
                dominant_destination: 3,
                geometry_restored: true,
            },
            ResponseCaseInput {
                onset_step: 32,
                end_step: 38,
                first_trace_step: 32,
                trace_at_return_l1: 0.08,
                peak_trace_l1: 0.20,
                peak_trace_step: 49,
                final_trace_l1: 0.179709,
                final_max_state_delta: 0.08,
                final_share_distance: 0.004,
                dominant_destination: 2,
                geometry_restored: true,
            },
            ResponseCaseInput {
                onset_step: 192,
                end_step: 198,
                first_trace_step: 192,
                trace_at_return_l1: 0.04,
                peak_trace_l1: 0.13,
                peak_trace_step: 220,
                final_trace_l1: 0.1045,
                final_max_state_delta: 0.05,
                final_share_distance: 0.003,
                dominant_destination: 4,
                geometry_restored: true,
            },
        ]
    }

    fn input() -> SeparabilityInput {
        SeparabilityInput {
            baseline_fingerprint: "VRB1-TEST".into(),
            baseline_state: "CALIBRATED".into(),
            cell_count: 4,
            seed_hex: "10 FF FF".into(),
            seed_bytes: 3,
            response_cases: cases(),
            time_gate_controls_clean: true,
            geometry_egress_distance: 0.063249479131,
            geometry_equivalence_max_share_delta: 0.0,
            self_reconfiguration_observed: true,
            recovery_observed: true,
            temporal_path_dependence_observed: true,
        }
    }

    #[test]
    fn response_fingerprint_excludes_onset_descriptor() {
        let mut a = cases()[0].clone();
        let mut b = a.clone();
        b.onset_step = 999;
        b.end_step = 1005;
        assert_eq!(response_fingerprint(&a), response_fingerprint(&b));
        a.final_trace_l1 += 0.01;
        assert_ne!(response_fingerprint(&a), response_fingerprint(&b));
    }

    #[test]
    fn pair_matrix_is_complete_and_symmetric_by_construction() {
        let r = analyze_response_separability(input(), DEFAULT_SEPARATION_THRESHOLD).unwrap();
        assert_eq!(r.signature_count, 3);
        assert_eq!(r.pair_count, 3);
        assert!(r.minimum_pair_distance >= 0.0);
        assert!(r.maximum_pair_distance <= 1.0);
    }

    #[test]
    fn categorical_destination_change_is_response_separable() {
        let r = analyze_response_separability(input(), DEFAULT_SEPARATION_THRESHOLD).unwrap();
        assert!(r.cluster_count >= 3);
        assert_eq!(r.unique_dominant_destinations, 3);
        assert!(r.separated_pair_count > 0);
    }

    #[test]
    fn contact_axis_remains_unclaimed() {
        let r = analyze_response_separability(input(), DEFAULT_SEPARATION_THRESHOLD).unwrap();
        let contact = r.axes.iter().find(|a| a.axis == "CONTACT").unwrap();
        assert_eq!(contact.state, "NOT_ISOLATED_IN_000001");
        assert_eq!(r.llm_ingestion_state, "LOCKED");
    }
}
