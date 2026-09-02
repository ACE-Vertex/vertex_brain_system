use serde::Serialize;

pub const BASELINE_CONTRACT: &str = "vertex.brain.vcell-resident-baseline.v1";
pub const BASELINE_EXPERIMENT: &str = "VCELL_RESIDENT_RESPONSE_BASELINE_000001";

#[derive(Debug, Clone)]
pub struct BaselineInput {
    pub cell_count: usize,
    pub contact_count: usize,
    pub seed_hex: String,
    pub seed_bytes: usize,
    pub geometry: [f64; 3],
    pub resident_phase: String,

    pub control_replay_egress_l1: f64,
    pub control_replay_error_delta: f64,

    pub geometry_equivalence_max_share_delta: f64,
    pub geometry_delta_egress_distance: f64,
    pub geometry_self_reconfiguration_observed: bool,
    pub geometry_recovery_observed: bool,

    pub first_delta_step: usize,
    pub first_delta_step_by_cell: [usize; 4],
    pub temporal_locality_fraction_percent: f64,
    pub temporal_peak_delta_step: usize,
    pub temporal_peak_delta_l1: f64,

    pub timing_trace_span_l1: f64,
    pub timing_trace_span_percent_of_max: f64,
    pub timing_unique_dominant_destinations: usize,
    pub timing_strongest_onset_step: usize,
    pub timing_weakest_onset_step: usize,
    pub timing_sensitive_state_observed: bool,
    pub timing_same_final_geometry: bool,
    pub timing_deterministic_replay: bool,
    pub timing_zero_pulse_control_clean: bool,
    pub timing_temporal_crosscheck_clean: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResidentResponseBaseline {
    pub schema: &'static str,
    pub experiment_id: &'static str,
    pub state: &'static str,
    pub resident_scope: &'static str,

    pub cell_count: usize,
    pub contact_count: usize,
    pub seed_hex: String,
    pub seed_bytes: usize,
    pub geometry: [f64; 3],
    pub resident_phase_at_capture: String,

    pub control_replay_egress_l1: f64,
    pub control_replay_error_delta: f64,
    pub control_replay_clean: bool,

    pub geometry_equivalence_max_share_delta: f64,
    pub geometry_delta_egress_distance: f64,
    pub self_reconfiguration_observed: bool,
    pub recovery_observed: bool,

    pub first_delta_step: usize,
    pub first_delta_step_by_cell: [usize; 4],
    pub temporal_locality_fraction_percent: f64,
    pub temporal_peak_delta_step: usize,
    pub temporal_peak_delta_l1: f64,

    pub timing_trace_span_l1: f64,
    pub timing_trace_span_percent_of_max: f64,
    pub timing_unique_dominant_destinations: usize,
    pub timing_strongest_onset_step: usize,
    pub timing_weakest_onset_step: usize,
    pub timing_sensitive_state_observed: bool,

    pub timing_controls_clean: bool,
    pub response_axes_observed: usize,
    pub baseline_fingerprint: String,

    pub laa_state: &'static str,
    pub llm_ingestion_state: &'static str,
    pub provider_state: &'static str,
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

fn canonical_fingerprint_material(i: &BaselineInput) -> String {
    format!(
        concat!(
            "cells={};contacts={};seed={};seed_bytes={};",
            "geometry={:.9},{:.9},{:.9};",
            "ctrl_e={:.12};ctrl_err={:.12};",
            "geo_eq={:.12};geo_delta={:.12};self={};recovery={};",
            "first={};first_cells={},{},{},{};",
            "temporal_locality={:.9};peak_step={};peak_l1={:.12};",
            "time_span={:.12};time_span_pct={:.9};dest={};",
            "strong={};weak={};timing={};same_geo={};det={};zero={};cross={}"
        ),
        i.cell_count,
        i.contact_count,
        i.seed_hex,
        i.seed_bytes,
        i.geometry[0],
        i.geometry[1],
        i.geometry[2],
        i.control_replay_egress_l1,
        i.control_replay_error_delta,
        i.geometry_equivalence_max_share_delta,
        i.geometry_delta_egress_distance,
        i.geometry_self_reconfiguration_observed,
        i.geometry_recovery_observed,
        i.first_delta_step,
        i.first_delta_step_by_cell[0],
        i.first_delta_step_by_cell[1],
        i.first_delta_step_by_cell[2],
        i.first_delta_step_by_cell[3],
        i.temporal_locality_fraction_percent,
        i.temporal_peak_delta_step,
        i.temporal_peak_delta_l1,
        i.timing_trace_span_l1,
        i.timing_trace_span_percent_of_max,
        i.timing_unique_dominant_destinations,
        i.timing_strongest_onset_step,
        i.timing_weakest_onset_step,
        i.timing_sensitive_state_observed,
        i.timing_same_final_geometry,
        i.timing_deterministic_replay,
        i.timing_zero_pulse_control_clean,
        i.timing_temporal_crosscheck_clean,
    )
}

pub fn build_resident_response_baseline(i: BaselineInput) -> ResidentResponseBaseline {
    let control_replay_clean =
        i.control_replay_egress_l1 < 1.0e-12 && i.control_replay_error_delta < 1.0e-12;

    let timing_controls_clean = i.timing_same_final_geometry
        && i.timing_deterministic_replay
        && i.timing_zero_pulse_control_clean
        && i.timing_temporal_crosscheck_clean;

    let calibrated = i.cell_count == 4
        && i.seed_bytes == 3
        && control_replay_clean
        && i.geometry_equivalence_max_share_delta < 1.0e-12
        && i.geometry_self_reconfiguration_observed
        && i.geometry_recovery_observed
        && i.first_delta_step > 0
        && i.temporal_peak_delta_step > i.first_delta_step
        && i.timing_sensitive_state_observed
        && timing_controls_clean
        && i.timing_unique_dominant_destinations >= 2;

    let material = canonical_fingerprint_material(&i);
    let fingerprint = format!("VRB1-{:016X}", fnv1a64(material.as_bytes()));

    ResidentResponseBaseline {
        schema: BASELINE_CONTRACT,
        experiment_id: BASELINE_EXPERIMENT,
        state: if calibrated { "CALIBRATED" } else { "OBSERVE" },
        resident_scope: "4C_REFERENCE_CORE",

        cell_count: i.cell_count,
        contact_count: i.contact_count,
        seed_hex: i.seed_hex,
        seed_bytes: i.seed_bytes,
        geometry: i.geometry,
        resident_phase_at_capture: i.resident_phase,

        control_replay_egress_l1: i.control_replay_egress_l1,
        control_replay_error_delta: i.control_replay_error_delta,
        control_replay_clean,

        geometry_equivalence_max_share_delta: i.geometry_equivalence_max_share_delta,
        geometry_delta_egress_distance: i.geometry_delta_egress_distance,
        self_reconfiguration_observed: i.geometry_self_reconfiguration_observed,
        recovery_observed: i.geometry_recovery_observed,

        first_delta_step: i.first_delta_step,
        first_delta_step_by_cell: i.first_delta_step_by_cell,
        temporal_locality_fraction_percent: i.temporal_locality_fraction_percent,
        temporal_peak_delta_step: i.temporal_peak_delta_step,
        temporal_peak_delta_l1: i.temporal_peak_delta_l1,

        timing_trace_span_l1: i.timing_trace_span_l1,
        timing_trace_span_percent_of_max: i.timing_trace_span_percent_of_max,
        timing_unique_dominant_destinations: i.timing_unique_dominant_destinations,
        timing_strongest_onset_step: i.timing_strongest_onset_step,
        timing_weakest_onset_step: i.timing_weakest_onset_step,
        timing_sensitive_state_observed: i.timing_sensitive_state_observed,

        timing_controls_clean,
        response_axes_observed: 5,
        baseline_fingerprint: fingerprint,

        laa_state: "OBSERVE_ONLY",
        llm_ingestion_state: "LOCKED_UNTIL_POST_BASELINE_GATE",
        provider_state: "STANDBY_ONLY",
        transfer_state: "CANDIDATE_GATED",
        next_gate: "RESIDENT_SCALE_AND_RESPONSE_SEPARABILITY",
        interpretation: "4C resident response calibration; not an intelligence score, not an LLM transfer map, and not permission to ingest a model.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> BaselineInput {
        BaselineInput {
            cell_count: 4,
            contact_count: 4,
            seed_hex: "10 FF FF".into(),
            seed_bytes: 3,
            geometry: [0.0, 0.0, 0.0],
            resident_phase: "READY".into(),
            control_replay_egress_l1: 0.0,
            control_replay_error_delta: 0.0,
            geometry_equivalence_max_share_delta: 0.0,
            geometry_delta_egress_distance: 0.063249479131,
            geometry_self_reconfiguration_observed: true,
            geometry_recovery_observed: true,
            first_delta_step: 3,
            first_delta_step_by_cell: [3, 4, 3, 7],
            temporal_locality_fraction_percent: 7.1875,
            temporal_peak_delta_step: 249,
            temporal_peak_delta_l1: 8.389589138745,
            timing_trace_span_l1: 0.1747365525477,
            timing_trace_span_percent_of_max: 97.233,
            timing_unique_dominant_destinations: 3,
            timing_strongest_onset_step: 32,
            timing_weakest_onset_step: 4,
            timing_sensitive_state_observed: true,
            timing_same_final_geometry: true,
            timing_deterministic_replay: true,
            timing_zero_pulse_control_clean: true,
            timing_temporal_crosscheck_clean: true,
        }
    }

    #[test]
    fn reference_baseline_calibrates_without_llm_ingestion() {
        let r = build_resident_response_baseline(fixture());
        assert_eq!(r.state, "CALIBRATED");
        assert_eq!(r.resident_scope, "4C_REFERENCE_CORE");
        assert_eq!(r.llm_ingestion_state, "LOCKED_UNTIL_POST_BASELINE_GATE");
        assert_eq!(r.provider_state, "STANDBY_ONLY");
        assert_eq!(r.transfer_state, "CANDIDATE_GATED");
    }

    #[test]
    fn baseline_fingerprint_is_deterministic() {
        let a = build_resident_response_baseline(fixture());
        let b = build_resident_response_baseline(fixture());
        assert_eq!(a.baseline_fingerprint, b.baseline_fingerprint);
        assert!(a.baseline_fingerprint.starts_with("VRB1-"));
    }

    #[test]
    fn dirty_replay_cannot_report_calibrated() {
        let mut f = fixture();
        f.control_replay_egress_l1 = 1.0e-5;
        let r = build_resident_response_baseline(f);
        assert_eq!(r.state, "OBSERVE");
        assert!(!r.control_replay_clean);
    }
}
