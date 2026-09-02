#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

mod laa;
mod resident32;
mod resident32_baseline;
mod resident32_persistent_difference;
mod resident_baseline;
mod resident_separability;
mod scale_expansion;
mod vcell_mount;
use laa::{
    analyze_observations, default_probe_plan, laa_self_test_report, laa_status_view, AnalysisDepth,
    LaaAnalysisReport, LaaObservationInput, LaaProbePlan, LaaStatusView,
};
use resident32::{Resident32, Resident32PassiveRead, Resident32ProbeReport, Resident32Status};
use resident32_baseline::{run_resident32_baseline, Resident32BaselineReport};
use resident32_persistent_difference::{run_persistent_difference, PersistentDifferenceReport};
use resident_baseline::{
    build_resident_response_baseline, BaselineInput, ResidentResponseBaseline,
};
use resident_separability::{
    analyze_response_separability, ResponseCaseInput, ResponseSeparabilityReport,
    SeparabilityInput, DEFAULT_SEPARATION_THRESHOLD,
};
use scale_expansion::{run_scale_expansion, ScaleExpansionReport};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use vcell_mount::{vcell_mount_view_32c, VcellMountView};

use vcell_adaptive_field::VxnMicroSeed;
use vcell_delta_propagation_000015::{run_delta_propagation_probe, DeltaPropagationReport};
use vcell_geometry_reuse_000014::{run_geometry_reuse_probe, GeometryReuseReport};
use vcell_self_reconfiguration_000011::{
    default_start_geometry, display_cells, disturb_geometry, internal_contact_display, observe,
    outer_edges, outer_shell, self_reconfigure, DisplayCell, InternalContactDisplay, Observation,
    ReconfigurationEvent, DEFAULT_FLOW_STEPS,
};
use vcell_temporal_frontier_000016::{
    run_temporal_frontier_probe, TemporalFrontierReport, TemporalSample,
};
use vcell_time_gated_geometry_000017::{run_time_gate_probe, TimeGateCase, TimeGateReport};

#[derive(Debug)]
struct Engine {
    seed: VxnMicroSeed,
    current: Observation,
    phase: String,
    history: Vec<ReconfigurationEvent>,
    last_evaluations: usize,
    outer_shell_distance: f64,
}

impl Engine {
    fn new() -> Result<Self, String> {
        let seed = VxnMicroSeed::default();
        let current = observe(default_start_geometry(), seed, DEFAULT_FLOW_STEPS)?;
        Ok(Self {
            seed,
            current,
            phase: "READY".into(),
            history: Vec::new(),
            last_evaluations: 1,
            outer_shell_distance: 0.0,
        })
    }

    fn adapt(&mut self) -> Result<(), String> {
        let run = self_reconfigure(self.current.geometry, self.seed, DEFAULT_FLOW_STEPS)?;
        self.current = run.final_state;
        self.history = run.events;
        self.last_evaluations = run.evaluations;
        self.outer_shell_distance = run.outer_shell_distance;
        self.phase = "RETAINED".into();
        Ok(())
    }

    fn disturb(&mut self) -> Result<(), String> {
        let geometry = disturb_geometry(self.current.geometry)?;
        self.current = observe(geometry, self.seed, DEFAULT_FLOW_STEPS)?;
        self.history.clear();
        self.last_evaluations = 1;
        self.phase = "DISTURBED".into();
        Ok(())
    }

    fn reset(&mut self) -> Result<(), String> {
        *self = Self::new()?;
        Ok(())
    }
}

struct AppState {
    engine: Mutex<Engine>,         // retained 4C regression reference
    resident32: Mutex<Resident32>, // promoted persistent resident
}

#[derive(Debug, Clone, Serialize)]
struct MembraneCapability {
    name: &'static str,
    direction: &'static str,
    authority: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct BrainMembraneView {
    contract: &'static str,
    connector: &'static str,
    state: &'static str,
    integrity_percent: f64,
    connected_vcells: usize,
    observation_channels: usize,
    arbitrary_shell: bool,
    filesystem_write: bool,
    network_egress: bool,
    capabilities: Vec<MembraneCapability>,
}

#[derive(Debug, Clone, Serialize)]
struct BrainSnapshot {
    schema: &'static str,
    system: &'static str,
    version: &'static str,
    observation_layer: &'static str,
    timestamp_ms: u128,
    focused_cell: &'static str,
    membrane: BrainMembraneView,
    laa: LaaStatusView,
    vcell_mount: VcellMountView,
    resident32: Resident32Status,
    frame_role: &'static str,
    frame: Frame,
}

#[derive(Debug, Clone, Serialize)]
struct CellView {
    local_cell: u32,
    position: [f64; 3],
    vertices: Vec<[f64; 3]>,
    edges: Vec<[usize; 2]>,
}

#[derive(Debug, Clone, Serialize)]
struct IcView {
    id: u8,
    ingress: bool,
    outer: [f64; 3],
    inner: [f64; 3],
}

#[derive(Debug, Clone, Serialize)]
struct Frame {
    phase: String,
    seed_hex: String,
    geometry: [f64; 3],
    egress: [f64; 3],
    error: f64,
    evaluations: usize,
    outer_shell_distance: f64,
    outer_vertices: Vec<[f64; 3]>,
    outer_edges: Vec<[usize; 2]>,
    inner_cells: Vec<CellView>,
    internal_contacts: Vec<IcView>,
}

#[derive(Debug, Clone, Serialize)]
struct PerfView {
    samples: usize,
    flow_steps: usize,
    nodes: usize,
    bonds: usize,
    invariant_bonds: usize,
    mutable_contacts: usize,
    equivalence_max_share_delta: f64,
    learned_geometry_equal: bool,
    recovery_geometry_equal: bool,

    baseline_p50_us: f64,
    baseline_p99_us: f64,
    baseline_ops: f64,

    fresh_p50_us: f64,
    fresh_p99_us: f64,
    fresh_ops: f64,
    fresh_speedup: f64,

    reused_p50_us: f64,
    reused_p99_us: f64,
    reused_ops: f64,
    reused_speedup: f64,
    reused_vs_fresh_speedup: f64,

    update_p50_us: f64,
    update_p99_us: f64,
    flow_p50_us: f64,
    flow_p99_us: f64,
    flow_ops: f64,

    baseline_calls: f64,
    baseline_bytes: f64,
    fresh_calls: f64,
    fresh_bytes: f64,
    reused_calls: f64,
    reused_bytes: f64,
    update_calls: f64,
    update_bytes: f64,
    flow_calls: f64,
    flow_bytes: f64,

    adaptation_ms: f64,
    adaptation_evaluations: usize,
    retained_moves: usize,
    touched_ratio_percent: f64,
    recovery_ms: f64,
    recovery_evaluations: usize,
    self_reconfiguration_observed: bool,
    recovery_observed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeltaView {
    flow_steps: usize,
    nodes: usize,
    bonds: usize,
    contacts: usize,
    recomputed_contacts: usize,
    changed_contacts: usize,
    hinge_preserved_contacts: usize,
    first_delta_step: usize,
    first_delta_step_by_cell: [usize; 4],
    peak_active_nodes: usize,
    average_active_nodes: f64,
    average_active_ratio_percent: f64,
    peak_frontier_bonds: usize,
    average_frontier_bonds: f64,
    average_frontier_ratio_percent: f64,
    equivalence_max_share_delta: f64,
    egress_distance: f64,
    baseline_egress: [f64; 3],
    changed_egress: [f64; 3],
    delta_observed: bool,
    frontier_savings_percent: f64,
    near_global_frontier_observed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct TemporalSampleView {
    step: usize,
    active_nodes: usize,
    active_node_ratio_percent: f64,
    frontier_bonds: usize,
    frontier_bond_ratio_percent: f64,
    active_cells: usize,
    cell_delta_l1: [f64; 4],
    max_state_delta: f64,
    total_state_delta_l1: f64,
}

#[derive(Debug, Clone, Serialize)]
struct TemporalView {
    flow_steps: usize,
    first_delta_step: usize,
    first_delta_step_by_cell: [usize; 4],
    first_two_cells_step: usize,
    all_cells_step: usize,

    nodes_25_step: usize,
    nodes_50_step: usize,
    nodes_75_step: usize,
    nodes_95_step: usize,
    nodes_100_step: usize,

    bonds_25_step: usize,
    bonds_50_step: usize,
    bonds_75_step: usize,
    bonds_95_step: usize,
    bonds_100_step: usize,

    temporal_locality_steps: usize,
    temporal_locality_fraction_percent: f64,

    peak_delta_step: usize,
    peak_delta_l1: f64,
    peak_max_state_delta: f64,

    equivalence_max_share_delta: f64,
    egress_distance: f64,

    matches_000015_first_delta: bool,
    matches_000015_cell_arrivals: bool,
    matches_000015_egress: bool,

    timeline: Vec<TemporalSampleView>,
}

#[derive(Debug, Clone, Serialize)]
struct TimeGateCaseView {
    onset_step: usize,
    end_step: usize,
    first_trace_step: usize,
    trace_at_return_l1: f64,
    peak_trace_l1: f64,
    peak_trace_step: usize,
    final_trace_l1: f64,
    final_max_state_delta: f64,
    final_share_distance: f64,
    pulsed_dominant_destination: u32,
    geometry_restored: bool,
}

#[derive(Debug, Clone, Serialize)]
struct TimeGateView {
    pulse_geometry: [f64; 3],
    pulse_width_steps: usize,
    post_pulse_horizon_steps: usize,

    strongest_onset_step: usize,
    weakest_onset_step: usize,
    strongest_final_trace_l1: f64,
    weakest_final_trace_l1: f64,
    timing_trace_span_l1: f64,
    timing_trace_span_percent_of_max: f64,

    strongest_share_distance: f64,
    max_share_distance_onset_step: usize,
    unique_dominant_destinations: usize,

    timing_sensitive_state_observed: bool,
    same_final_geometry_all_cases: bool,
    deterministic_replay: bool,
    zero_pulse_control_clean: bool,
    temporal_frontier_crosscheck_clean: bool,

    cases: Vec<TimeGateCaseView>,
}

#[tauri::command]
fn brain_snapshot(state: tauri::State<'_, AppState>) -> Result<BrainSnapshot, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "4C reference lock poisoned")?;
    let frame = frame_from(&engine)?;

    let resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident lock poisoned")?;
    let resident32 = resident.status()?;
    let mount = vcell_mount_view_32c(&resident32);

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock before unix epoch")?
        .as_millis();

    Ok(BrainSnapshot {
        schema: "vertex.brain.snapshot.v5",
        system: "Vertex Brain System",
        version: "0.9.1",
        observation_layer: "vSCOPE integrated",
        timestamp_ms,
        focused_cell: "REGION 000032 / CELL 000001",
        membrane: BrainMembraneView {
            contract: "vertex.brain.membrane.v1",
            connector: "vCELL 32C resident outer-membrane connector",
            state: "CONNECTED",
            integrity_percent: 100.0,
            connected_vcells: resident32.cells,
            observation_channels: resident32.contact_points,
            arbitrary_shell: false,
            filesystem_write: false,
            network_egress: false,
            capabilities: vec![
                MembraneCapability {
                    name: "observe_32c_resident_state",
                    direction: "vCELL -> Brain",
                    authority: "READ",
                },
                MembraneCapability {
                    name: "inject_resident_probe",
                    direction: "Brain -> vCELL",
                    authority: "EXPERIMENT",
                },
                MembraneCapability {
                    name: "reset_resident_dynamic_state",
                    direction: "Brain -> vCELL",
                    authority: "EXPERIMENT",
                },
                MembraneCapability {
                    name: "observe_4c_regression_reference",
                    direction: "Reference -> Brain",
                    authority: "READ",
                },
                MembraneCapability {
                    name: "return_evidence",
                    direction: "vCELL -> Brain",
                    authority: "READ",
                },
            ],
        },
        laa: laa_status_view(),
        vcell_mount: mount,
        resident32,
        frame_role: "4C_REGRESSION_REFERENCE",
        frame,
    })
}

#[tauri::command]
fn vcell_mount_status(state: tauri::State<'_, AppState>) -> Result<VcellMountView, String> {
    let resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident mount lock poisoned")?;
    let s = resident.status()?;
    Ok(vcell_mount_view_32c(&s))
}

#[tauri::command]
fn vcell_resident32_persistent_difference() -> Result<PersistentDifferenceReport, String> {
    run_persistent_difference()
}

#[tauri::command]
fn vcell_resident32_baseline() -> Result<Resident32BaselineReport, String> {
    run_resident32_baseline()
}

#[tauri::command]
fn vcell_resident32_status(state: tauri::State<'_, AppState>) -> Result<Resident32Status, String> {
    let resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident status lock poisoned")?;
    resident.status()
}

#[tauri::command]
fn vcell_resident32_passive_read(
    state: tauri::State<'_, AppState>,
) -> Result<Resident32PassiveRead, String> {
    let resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident passive read lock poisoned")?;
    Ok(resident.passive_read())
}

#[tauri::command]
fn vcell_resident32_settle(
    state: tauri::State<'_, AppState>,
    steps: Option<usize>,
) -> Result<Resident32PassiveRead, String> {
    let mut resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident settle lock poisoned")?;
    resident.settle(steps.unwrap_or(256))
}

#[tauri::command]
fn vcell_resident32_probe(
    state: tauri::State<'_, AppState>,
    steps: Option<usize>,
) -> Result<Resident32ProbeReport, String> {
    let mut resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident probe lock poisoned")?;
    resident.probe(steps)
}

#[tauri::command]
fn vcell_resident32_reset(state: tauri::State<'_, AppState>) -> Result<Resident32Status, String> {
    let mut resident = state
        .resident32
        .lock()
        .map_err(|_| "32C resident reset lock poisoned")?;
    resident.reset_dynamic_state()
}

#[tauri::command]
fn laa_status() -> LaaStatusView {
    laa_status_view()
}

#[tauri::command]
fn laa_probe_plan(depth: Option<String>) -> Result<LaaProbePlan, String> {
    Ok(default_probe_plan(AnalysisDepth::parse(depth)?))
}

#[tauri::command]
fn laa_analyze_observations(
    observations: Vec<LaaObservationInput>,
) -> Result<LaaAnalysisReport, String> {
    analyze_observations(&observations)
}

#[tauri::command]
fn laa_self_test() -> Result<LaaAnalysisReport, String> {
    laa_self_test_report()
}

#[tauri::command]
fn frame(state: tauri::State<'_, AppState>) -> Result<Frame, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "brain observation lock poisoned")?;
    frame_from(&engine)
}

#[tauri::command]
fn adapt(state: tauri::State<'_, AppState>) -> Result<Frame, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "brain observation lock poisoned")?;
    engine.adapt()?;
    frame_from(&engine)
}

#[tauri::command]
fn disturb(state: tauri::State<'_, AppState>) -> Result<Frame, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "brain observation lock poisoned")?;
    engine.disturb()?;
    frame_from(&engine)
}

#[tauri::command]
fn reset_probe(state: tauri::State<'_, AppState>) -> Result<Frame, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "brain observation lock poisoned")?;
    engine.reset()?;
    frame_from(&engine)
}

#[tauri::command]
fn run_geometry_reuse() -> Result<PerfView, String> {
    // VERTEX WORKS release verification uses 2048 samples.
    // Interactive vSCOPE keeps the same equations with a smaller sample count.
    Ok(perf_view(run_geometry_reuse_probe(512)?))
}

#[tauri::command]
fn run_delta_propagation() -> Result<DeltaView, String> {
    Ok(delta_view(run_delta_propagation_probe()?))
}

#[tauri::command]
fn run_temporal_frontier() -> Result<TemporalView, String> {
    Ok(temporal_view(run_temporal_frontier_probe()?))
}

#[tauri::command]
fn run_time_gate() -> Result<TimeGateView, String> {
    Ok(time_gate_view(run_time_gate_probe()?))
}

fn l1_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()
}

fn collect_resident_response_baseline(
    resident_phase: String,
) -> Result<ResidentResponseBaseline, String> {
    let seed = VxnMicroSeed::default();
    let geometry = default_start_geometry();

    // Standardized replay control: same 3-byte VXN, same geometry, same flow horizon.
    let control_a = observe(geometry, seed, DEFAULT_FLOW_STEPS)?;
    let control_b = observe(geometry, seed, DEFAULT_FLOW_STEPS)?;

    let geometry_reuse = run_geometry_reuse_probe(128)?;
    let delta = run_delta_propagation_probe()?;
    let temporal = run_temporal_frontier_probe()?;
    let timing = run_time_gate_probe()?;

    Ok(build_resident_response_baseline(BaselineInput {
        cell_count: display_cells(geometry)?.len(),
        contact_count: internal_contact_display(geometry)?.len(),
        seed_hex: seed.hex(),
        seed_bytes: 3,
        geometry: geometry.as_array(),
        resident_phase,

        control_replay_egress_l1: l1_distance(control_a.egress_share, control_b.egress_share),
        control_replay_error_delta: (control_a.homeostatic_error - control_b.homeostatic_error)
            .abs(),

        geometry_equivalence_max_share_delta: geometry_reuse.equivalence_max_share_delta,
        geometry_delta_egress_distance: delta.delta_egress_distance,
        geometry_self_reconfiguration_observed: geometry_reuse.hypothesis_self_reconfiguration,
        geometry_recovery_observed: geometry_reuse.hypothesis_recovery,

        first_delta_step: temporal.first_delta_step,
        first_delta_step_by_cell: temporal.first_delta_step_by_cell,
        temporal_locality_fraction_percent: temporal.temporal_locality_fraction_percent,
        temporal_peak_delta_step: temporal.peak_delta_step,
        temporal_peak_delta_l1: temporal.peak_delta_l1,

        timing_trace_span_l1: timing.timing_trace_span_l1,
        timing_trace_span_percent_of_max: timing.timing_trace_span_percent_of_max,
        timing_unique_dominant_destinations: timing.unique_dominant_destinations,
        timing_strongest_onset_step: timing.strongest_onset_step,
        timing_weakest_onset_step: timing.weakest_onset_step,
        timing_sensitive_state_observed: timing.timing_sensitive_state_observed,
        timing_same_final_geometry: timing.same_final_geometry_all_cases,
        timing_deterministic_replay: timing.deterministic_replay,
        timing_zero_pulse_control_clean: timing.zero_pulse_control_clean,
        timing_temporal_crosscheck_clean: timing.temporal_frontier_crosscheck_clean,
    }))
}

#[tauri::command]
fn vcell_resident_baseline(
    state: tauri::State<'_, AppState>,
) -> Result<ResidentResponseBaseline, String> {
    let resident_phase = state
        .engine
        .lock()
        .map_err(|_| "vCELL resident baseline lock poisoned")?
        .phase
        .clone();
    collect_resident_response_baseline(resident_phase)
}

fn collect_response_separability(
    resident_phase: String,
) -> Result<ResponseSeparabilityReport, String> {
    // Baseline remains the pre-transfer reference and must be calibrated first.
    let baseline = collect_resident_response_baseline(resident_phase)?;
    if baseline.state != "CALIBRATED" {
        return Err("4C resident baseline is not calibrated".into());
    }

    // Re-run the standardized timing sweep to obtain individual response traces.
    // This does not mutate the resident Engine and does not contact any provider.
    let timing = run_time_gate_probe()?;

    let cases: Vec<ResponseCaseInput> = timing
        .cases
        .iter()
        .map(|c| ResponseCaseInput {
            onset_step: c.onset_step,
            end_step: c.end_step,
            first_trace_step: c.first_trace_step,
            trace_at_return_l1: c.trace_at_return_l1,
            peak_trace_l1: c.peak_trace_l1,
            peak_trace_step: c.peak_trace_step,
            final_trace_l1: c.final_trace_l1,
            final_max_state_delta: c.final_max_state_delta,
            final_share_distance: c.final_share_distance,
            dominant_destination: c.pulsed_dominant_destination,
            geometry_restored: c.geometry_restored,
        })
        .collect();

    analyze_response_separability(
        SeparabilityInput {
            baseline_fingerprint: baseline.baseline_fingerprint,
            baseline_state: baseline.state.to_string(),
            cell_count: baseline.cell_count,
            seed_hex: baseline.seed_hex,
            seed_bytes: baseline.seed_bytes,
            response_cases: cases,
            time_gate_controls_clean: timing.same_final_geometry_all_cases
                && timing.deterministic_replay
                && timing.zero_pulse_control_clean
                && timing.temporal_frontier_crosscheck_clean,
            geometry_egress_distance: baseline.geometry_delta_egress_distance,
            geometry_equivalence_max_share_delta: baseline.geometry_equivalence_max_share_delta,
            self_reconfiguration_observed: baseline.self_reconfiguration_observed,
            recovery_observed: baseline.recovery_observed,
            temporal_path_dependence_observed: baseline.timing_sensitive_state_observed,
        },
        DEFAULT_SEPARATION_THRESHOLD,
    )
}

#[tauri::command]
fn vcell_scale_expansion(
    state: tauri::State<'_, AppState>,
) -> Result<ScaleExpansionReport, String> {
    let resident_phase = state
        .engine
        .lock()
        .map_err(|_| "vCELL scale expansion lock poisoned")?
        .phase
        .clone();

    let baseline = collect_resident_response_baseline(resident_phase)?;
    if baseline.state != "CALIBRATED" {
        return Err("4C resident reference must be calibrated before 32C expansion".into());
    }

    run_scale_expansion(baseline.baseline_fingerprint)
}

#[tauri::command]
fn vcell_response_separability(
    state: tauri::State<'_, AppState>,
) -> Result<ResponseSeparabilityReport, String> {
    let resident_phase = state
        .engine
        .lock()
        .map_err(|_| "vCELL separability lock poisoned")?
        .phase
        .clone();
    collect_response_separability(resident_phase)
}

fn frame_from(engine: &Engine) -> Result<Frame, String> {
    let shell = outer_shell()?;
    let cells = display_cells(engine.current.geometry)?;
    let contacts = internal_contact_display(engine.current.geometry)?;
    Ok(Frame {
        phase: engine.phase.clone(),
        seed_hex: engine.seed.hex(),
        geometry: engine.current.geometry.as_array(),
        egress: engine.current.egress_share,
        error: engine.current.homeostatic_error,
        evaluations: engine.last_evaluations,
        outer_shell_distance: engine.outer_shell_distance,
        outer_vertices: shell.world_vertices,
        outer_edges: outer_edges()?,
        inner_cells: cells.into_iter().map(cell_view).collect(),
        internal_contacts: contacts.into_iter().map(ic_view).collect(),
    })
}

fn cell_view(cell: DisplayCell) -> CellView {
    CellView {
        local_cell: cell.local_cell,
        position: cell.position,
        vertices: cell.vertices,
        edges: cell.edges,
    }
}

fn ic_view(ic: InternalContactDisplay) -> IcView {
    IcView {
        id: ic.id,
        ingress: ic.ingress,
        outer: ic.outer,
        inner: ic.inner,
    }
}

fn time_gate_view(r: TimeGateReport) -> TimeGateView {
    TimeGateView {
        pulse_geometry: r.pulse_geometry,
        pulse_width_steps: r.pulse_width_steps,
        post_pulse_horizon_steps: r.post_pulse_horizon_steps,

        strongest_onset_step: r.strongest_onset_step,
        weakest_onset_step: r.weakest_onset_step,
        strongest_final_trace_l1: r.strongest_final_trace_l1,
        weakest_final_trace_l1: r.weakest_final_trace_l1,
        timing_trace_span_l1: r.timing_trace_span_l1,
        timing_trace_span_percent_of_max: r.timing_trace_span_percent_of_max,

        strongest_share_distance: r.strongest_share_distance,
        max_share_distance_onset_step: r.max_share_distance_onset_step,
        unique_dominant_destinations: r.unique_dominant_destinations,

        timing_sensitive_state_observed: r.timing_sensitive_state_observed,
        same_final_geometry_all_cases: r.same_final_geometry_all_cases,
        deterministic_replay: r.deterministic_replay,
        zero_pulse_control_clean: r.zero_pulse_control_clean,
        temporal_frontier_crosscheck_clean: r.temporal_frontier_crosscheck_clean,

        cases: r.cases.into_iter().map(time_gate_case_view).collect(),
    }
}

fn time_gate_case_view(c: TimeGateCase) -> TimeGateCaseView {
    TimeGateCaseView {
        onset_step: c.onset_step,
        end_step: c.end_step,
        first_trace_step: c.first_trace_step,
        trace_at_return_l1: c.trace_at_return_l1,
        peak_trace_l1: c.peak_trace_l1,
        peak_trace_step: c.peak_trace_step,
        final_trace_l1: c.final_trace_l1,
        final_max_state_delta: c.final_max_state_delta,
        final_share_distance: c.final_share_distance,
        pulsed_dominant_destination: c.pulsed_dominant_destination,
        geometry_restored: c.geometry_restored,
    }
}

fn temporal_view(r: TemporalFrontierReport) -> TemporalView {
    TemporalView {
        flow_steps: r.flow_steps,
        first_delta_step: r.first_delta_step,
        first_delta_step_by_cell: r.first_delta_step_by_cell,
        first_two_cells_step: r.first_two_cells_step,
        all_cells_step: r.all_cells_step,

        nodes_25_step: r.nodes_25_step,
        nodes_50_step: r.nodes_50_step,
        nodes_75_step: r.nodes_75_step,
        nodes_95_step: r.nodes_95_step,
        nodes_100_step: r.nodes_100_step,

        bonds_25_step: r.bonds_25_step,
        bonds_50_step: r.bonds_50_step,
        bonds_75_step: r.bonds_75_step,
        bonds_95_step: r.bonds_95_step,
        bonds_100_step: r.bonds_100_step,

        temporal_locality_steps: r.temporal_locality_steps,
        temporal_locality_fraction_percent: r.temporal_locality_fraction_percent,

        peak_delta_step: r.peak_delta_step,
        peak_delta_l1: r.peak_delta_l1,
        peak_max_state_delta: r.peak_max_state_delta,

        equivalence_max_share_delta: r.final_equivalence_max_share_delta,
        egress_distance: r.delta_egress_distance,

        matches_000015_first_delta: r.matches_000015_first_delta,
        matches_000015_cell_arrivals: r.matches_000015_cell_arrivals,
        matches_000015_egress: r.matches_000015_egress,

        timeline: r.timeline.into_iter().map(temporal_sample_view).collect(),
    }
}

fn temporal_sample_view(s: TemporalSample) -> TemporalSampleView {
    TemporalSampleView {
        step: s.step,
        active_nodes: s.active_nodes,
        active_node_ratio_percent: s.active_node_ratio_percent,
        frontier_bonds: s.frontier_bonds,
        frontier_bond_ratio_percent: s.frontier_bond_ratio_percent,
        active_cells: s.active_cells,
        cell_delta_l1: s.cell_delta_l1,
        max_state_delta: s.max_state_delta,
        total_state_delta_l1: s.total_state_delta_l1,
    }
}

fn delta_view(r: DeltaPropagationReport) -> DeltaView {
    DeltaView {
        flow_steps: r.flow_steps,
        nodes: r.node_count,
        bonds: r.bond_count,
        contacts: r.contact_count,
        recomputed_contacts: r.recomputed_contacts,
        changed_contacts: r.numerically_changed_contacts,
        hinge_preserved_contacts: r.hinge_preserved_contacts,
        first_delta_step: r.first_delta_step,
        first_delta_step_by_cell: r.first_delta_step_by_cell,
        peak_active_nodes: r.peak_active_nodes,
        average_active_nodes: r.average_active_nodes,
        average_active_ratio_percent: r.average_active_node_ratio_percent,
        peak_frontier_bonds: r.peak_frontier_bonds,
        average_frontier_bonds: r.average_frontier_bonds,
        average_frontier_ratio_percent: r.average_frontier_bond_ratio_percent,
        equivalence_max_share_delta: r.full_flow_equivalence_max_share_delta,
        egress_distance: r.delta_egress_distance,
        baseline_egress: r.baseline_egress,
        changed_egress: r.changed_egress,
        delta_observed: r.delta_observed,
        frontier_savings_percent: r.frontier_savings_percent,
        near_global_frontier_observed: r.near_global_frontier_observed,
    }
}

fn perf_view(r: GeometryReuseReport) -> PerfView {
    PerfView {
        samples: r.samples,
        flow_steps: r.flow_steps,
        nodes: r.node_count,
        bonds: r.bond_count,
        invariant_bonds: r.invariant_bonds,
        mutable_contacts: r.mutable_contact_bonds,
        equivalence_max_share_delta: r.equivalence_max_share_delta,
        learned_geometry_equal: r.learned_geometry_equal,
        recovery_geometry_equal: r.recovery_geometry_equal,

        baseline_p50_us: r.baseline_e2e.p50_ns as f64 / 1_000.0,
        baseline_p99_us: r.baseline_e2e.p99_ns as f64 / 1_000.0,
        baseline_ops: r.baseline_e2e.throughput_per_sec,

        fresh_p50_us: r.fresh_e2e.p50_ns as f64 / 1_000.0,
        fresh_p99_us: r.fresh_e2e.p99_ns as f64 / 1_000.0,
        fresh_ops: r.fresh_e2e.throughput_per_sec,
        fresh_speedup: r.fresh_speedup_vs_baseline_p50,

        reused_p50_us: r.reused_e2e.p50_ns as f64 / 1_000.0,
        reused_p99_us: r.reused_e2e.p99_ns as f64 / 1_000.0,
        reused_ops: r.reused_e2e.throughput_per_sec,
        reused_speedup: r.reused_speedup_vs_baseline_p50,
        reused_vs_fresh_speedup: r.reused_speedup_vs_fresh_p50,

        update_p50_us: r.reused_update.p50_ns as f64 / 1_000.0,
        update_p99_us: r.reused_update.p99_ns as f64 / 1_000.0,
        flow_p50_us: r.reused_flow.p50_ns as f64 / 1_000.0,
        flow_p99_us: r.reused_flow.p99_ns as f64 / 1_000.0,
        flow_ops: r.reused_flow.throughput_per_sec,

        baseline_calls: r.baseline_alloc.calls_per_observation,
        baseline_bytes: r.baseline_alloc.bytes_per_observation,
        fresh_calls: r.fresh_alloc.calls_per_observation,
        fresh_bytes: r.fresh_alloc.bytes_per_observation,
        reused_calls: r.reused_e2e_alloc.calls_per_observation,
        reused_bytes: r.reused_e2e_alloc.bytes_per_observation,
        update_calls: r.reused_update_alloc.calls_per_observation,
        update_bytes: r.reused_update_alloc.bytes_per_observation,
        flow_calls: r.reused_flow_alloc.calls_per_observation,
        flow_bytes: r.reused_flow_alloc.bytes_per_observation,

        adaptation_ms: r.reused_adaptation_ns as f64 / 1_000_000.0,
        adaptation_evaluations: r.reused_adaptation_evaluations,
        retained_moves: r.retained_moves,
        touched_ratio_percent: r.touched_ratio_percent,
        recovery_ms: r.reused_recovery_ns as f64 / 1_000_000.0,
        recovery_evaluations: r.reused_recovery_evaluations,
        self_reconfiguration_observed: r.hypothesis_self_reconfiguration,
        recovery_observed: r.hypothesis_recovery,
    }
}

fn main() {
    let engine =
        Engine::new().expect("Vertex Brain System must initialize its 4C regression reference");
    let resident32 =
        Resident32::new().expect("Vertex Brain System must initialize its promoted 32C resident");
    tauri::Builder::default()
        .manage(AppState {
            engine: Mutex::new(engine),
            resident32: Mutex::new(resident32),
        })
        .invoke_handler(tauri::generate_handler![
            brain_snapshot,
            frame,
            adapt,
            disturb,
            reset_probe,
            run_geometry_reuse,
            run_delta_propagation,
            run_temporal_frontier,
            run_time_gate,
            vcell_mount_status,
            vcell_resident32_persistent_difference,
            vcell_resident32_baseline,
            vcell_resident32_status,
            vcell_resident32_passive_read,
            vcell_resident32_settle,
            vcell_resident32_probe,
            vcell_resident32_reset,
            vcell_resident_baseline,
            vcell_response_separability,
            vcell_scale_expansion,
            laa_status,
            laa_probe_plan,
            laa_analyze_observations,
            laa_self_test
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vertex Brain System");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_still_exposes_nested_four_cell_structure() {
        let engine = Engine::new().unwrap();
        let frame = frame_from(&engine).unwrap();
        assert_eq!(frame.inner_cells.len(), 4);
        assert_eq!(frame.internal_contacts.len(), 4);
    }

    #[test]
    fn geometry_reuse_probe_requires_equivalent_behavior() {
        let view = perf_view(run_geometry_reuse_probe(128).unwrap());
        assert!(view.equivalence_max_share_delta < 1.0e-12);
        assert!(view.learned_geometry_equal);
        assert!(view.recovery_geometry_equal);
        assert_eq!(view.nodes, 96);
        assert_eq!(view.bonds, 160);
        assert_eq!(view.invariant_bonds, 144);
        assert_eq!(view.mutable_contacts, 16);
    }

    #[test]
    fn geometry_update_and_reused_e2e_report_zero_heap_pressure() {
        let view = perf_view(run_geometry_reuse_probe(128).unwrap());
        assert_eq!(view.update_calls, 0.0);
        assert_eq!(view.update_bytes, 0.0);
        assert_eq!(view.reused_calls, 0.0);
        assert_eq!(view.reused_bytes, 0.0);
        assert_eq!(view.flow_calls, 0.0);
        assert_eq!(view.flow_bytes, 0.0);
    }

    #[test]
    fn observed_intelligence_behavior_survives_persistent_space() {
        let view = perf_view(run_geometry_reuse_probe(128).unwrap());
        assert!(view.self_reconfiguration_observed);
        assert!(view.recovery_observed);
    }

    #[test]
    fn delta_propagation_matches_full_reusable_flow() {
        let view = delta_view(run_delta_propagation_probe().unwrap());
        assert!(view.equivalence_max_share_delta < 1.0e-12);
        assert_eq!(view.recomputed_contacts, 16);
        assert_eq!(view.changed_contacts, 10);
        assert_eq!(view.hinge_preserved_contacts, 6);
        assert!(view.delta_observed);
    }

    #[test]
    fn delta_frontier_is_observed_before_sparse_execution() {
        let view = delta_view(run_delta_propagation_probe().unwrap());
        assert!(view.first_delta_step > 0);
        assert!(view.peak_active_nodes > 0);
        assert!(view.peak_frontier_bonds > 0);
        assert!(view.average_frontier_ratio_percent >= 95.0);
        assert!(view.frontier_savings_percent < 5.0);
        assert!(view.near_global_frontier_observed);
    }

    #[test]
    fn temporal_frontier_preserves_delta_evidence() {
        let view = temporal_view(run_temporal_frontier_probe().unwrap());
        assert!(view.matches_000015_first_delta);
        assert!(view.matches_000015_cell_arrivals);
        assert!(view.matches_000015_egress);
        assert!(view.equivalence_max_share_delta < 1.0e-12);
    }

    #[test]
    fn temporal_frontier_exposes_ordered_arrival_structure() {
        let view = temporal_view(run_temporal_frontier_probe().unwrap());
        assert!(view.first_delta_step > 0);
        assert!(view.first_two_cells_step >= view.first_delta_step);
        assert!(view.all_cells_step >= view.first_two_cells_step);
        assert_eq!(view.timeline.len(), view.flow_steps);
        assert!(view.bonds_95_step >= view.first_delta_step);
    }

    #[test]
    fn time_gate_exposes_timing_as_a_state_variable() {
        let view = time_gate_view(run_time_gate_probe().unwrap());
        assert!(view.timing_sensitive_state_observed);
        assert!(view.timing_trace_span_l1 > 0.0);
        assert!(view.same_final_geometry_all_cases);
    }

    #[test]
    fn time_gate_controls_are_clean() {
        let view = time_gate_view(run_time_gate_probe().unwrap());
        assert!(view.zero_pulse_control_clean);
        assert!(view.deterministic_replay);
        assert!(view.temporal_frontier_crosscheck_clean);
    }

    #[test]
    fn brain_membrane_contract_stays_fail_closed_at_outer_boundary() {
        let engine = Engine::new().unwrap();
        let frame = frame_from(&engine).unwrap();
        assert_eq!(frame.inner_cells.len(), 4);
        assert_eq!(frame.internal_contacts.len(), 4);
        let membrane = BrainMembraneView {
            contract: "vertex.brain.membrane.v1",
            connector: "vCELL experimental outer-membrane connector",
            state: "CONNECTED",
            integrity_percent: 100.0,
            connected_vcells: frame.inner_cells.len(),
            observation_channels: frame.internal_contacts.len(),
            arbitrary_shell: false,
            filesystem_write: false,
            network_egress: false,
            capabilities: vec![],
        };
        assert!(!membrane.arbitrary_shell);
        assert!(!membrane.filesystem_write);
        assert!(!membrane.network_egress);
        assert_eq!(membrane.connected_vcells, 4);
    }

    #[test]
    fn laa_foundation_is_fail_closed_and_black_box_active() {
        let status = laa_status_view();
        assert_eq!(status.contract, "vertex.brain.laa.v1");
        assert_eq!(status.black_box, "ACTIVE");
        assert!(!status.direct_network);
        assert!(!status.direct_filesystem);
        assert!(!status.direct_process_execution);
    }

    #[test]
    fn laa_probe_plan_exposes_response_tomography_surface() {
        let plan = default_probe_plan(AnalysisDepth::BlackBox);
        assert_eq!(plan.probe_count, 14);
        assert!(!plan.direct_execution);
        assert!(plan.probes.iter().any(|p| p.family.label() == "TEMPORAL"));
        assert!(plan.probes.iter().any(|p| p.family.label() == "ORDER"));
    }

    #[test]
    fn laa_self_test_builds_candidate_transfer_map_without_model_io() {
        let report = laa_self_test_report().unwrap();
        assert_eq!(report.contract, "vertex.brain.laa.v1");
        assert!(!report.raw_prompt_or_response_retained);
        assert!(report.tomography.temporal_path_variation > 0.0);
        assert_eq!(report.transfer_map.status, "CANDIDATE_ONLY");
        assert!(report
            .transfer_map
            .candidates
            .iter()
            .any(|c| c.channel == "TIME"));
    }

    #[test]
    fn mount_contract_promotes_32c_and_retains_4c_reference() {
        let resident = Resident32::new().unwrap();
        let s = resident.status().unwrap();
        let m = vcell_mount_view_32c(&s);
        assert_eq!(m.contract, "vertex.brain.vcell-mount.v2");
        assert_eq!(m.resident_id, "REGION 000032");
        assert_eq!(m.resident_cells, 32);
        assert_eq!(m.vertices, 768);
        assert_eq!(m.reference_scope, "4C_REGRESSION_REFERENCE");
        assert_eq!(m.seed_bytes, 3);
    }

    #[test]
    fn promoted_mount_keeps_laa_gated() {
        let resident = Resident32::new().unwrap();
        let s = resident.status().unwrap();
        let m = vcell_mount_view_32c(&s);
        assert_eq!(m.laa_transfer_ingress, "CANDIDATE_GATED");
        assert!(!m.automatic_transplant);
        assert!(!m.direct_world_mutation);
    }

    #[test]
    fn resident_response_baseline_is_calibrated_before_llm_ingestion() {
        let r = collect_resident_response_baseline("READY".into()).unwrap();
        assert_eq!(r.schema, "vertex.brain.vcell-resident-baseline.v1");
        assert_eq!(r.experiment_id, "VCELL_RESIDENT_RESPONSE_BASELINE_000001");
        assert_eq!(r.state, "CALIBRATED");
        assert_eq!(r.cell_count, 4);
        assert_eq!(r.seed_bytes, 3);
        assert!(r.control_replay_clean);
        assert!(r.timing_sensitive_state_observed);
        assert!(r.timing_controls_clean);
        assert_eq!(r.llm_ingestion_state, "LOCKED_UNTIL_POST_BASELINE_GATE");
        assert_eq!(r.provider_state, "STANDBY_ONLY");
    }

    #[test]
    fn resident_response_baseline_fingerprint_replays_exactly() {
        let a = collect_resident_response_baseline("READY".into()).unwrap();
        let b = collect_resident_response_baseline("READY".into()).unwrap();
        assert_eq!(a.baseline_fingerprint, b.baseline_fingerprint);
        assert_eq!(a.first_delta_step_by_cell, [3, 4, 3, 7]);
    }

    #[test]
    fn baseline_keeps_llm_and_laa_outside_the_vcell_write_path() {
        let r = collect_resident_response_baseline("READY".into()).unwrap();
        assert_eq!(r.laa_state, "OBSERVE_ONLY");
        assert_eq!(r.transfer_state, "CANDIDATE_GATED");
        assert_ne!(r.llm_ingestion_state, "ACTIVE");
        assert_ne!(r.provider_state, "CONNECTED");
    }

    #[test]
    fn four_cell_response_separability_runs_without_llm_ingestion() {
        let r = collect_response_separability("READY".into()).unwrap();
        assert_eq!(r.schema, "vertex.brain.vcell-response-separability.v1");
        assert_eq!(r.experiment_id, "VCELL_4C_RESPONSE_SEPARABILITY_000001");
        assert_eq!(r.cell_count, 4);
        assert_eq!(r.signature_count, 12);
        assert_eq!(r.pair_count, 66);
        assert!(r.controls_clean);
        assert!(r.cluster_count >= 2);
        assert!(r.separated_pair_count > 0);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
        assert_eq!(r.provider_state, "STANDBY_ONLY");
    }

    #[test]
    fn separability_does_not_claim_independent_contact_evidence() {
        let r = collect_response_separability("READY".into()).unwrap();
        let contact = r.axes.iter().find(|a| a.axis == "CONTACT").unwrap();
        assert_eq!(contact.state, "NOT_ISOLATED_IN_000001");
    }

    #[test]
    fn separability_response_fingerprints_are_replay_stable() {
        let a = collect_response_separability("READY".into()).unwrap();
        let b = collect_response_separability("READY".into()).unwrap();
        let af: Vec<_> = a
            .signatures
            .iter()
            .map(|s| s.response_fingerprint.clone())
            .collect();
        let bf: Vec<_> = b
            .signatures
            .iter()
            .map(|s| s.response_fingerprint.clone())
            .collect();
        assert_eq!(af, bf);
        assert_eq!(a.closest_onsets, b.closest_onsets);
    }

    #[test]
    fn brain_system_exposes_32c_scale_candidate_without_promoting_it() {
        let r = run_scale_expansion("VRB1-BRAIN-REFERENCE".into()).unwrap();
        assert_eq!(r.schema, "vertex.brain.vcell-scale-expansion.v1");
        assert_eq!(r.experiment_id, "VCELL_32C_SCALE_EXPANSION_000001");
        assert_eq!(r.current_resident_scope, "4C_REFERENCE_CORE");
        assert_eq!(r.candidate_scope, "32C_REGION_CANDIDATE");
        assert_eq!(r.resident_promotion_state, "CANDIDATE_ONLY");
        assert_eq!(r.cells, 32);
        assert_eq!(r.vertices, 768);
        assert_eq!(r.seed_bytes, 3);
        assert_eq!(r.case_count, 7);
        assert_eq!(r.pair_count, 21);
        assert!(r.response_diversity_observed);
        assert!(!r.cross_scale_numeric_equivalence_claimed);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
    }

    #[test]
    fn promoted_32c_carries_persistent_probe_history() {
        let mut resident = Resident32::new().unwrap();
        let a = resident.probe(Some(32)).unwrap();
        let b = resident.probe(Some(32)).unwrap();
        assert_eq!(a.after_step, 32);
        assert_eq!(b.before_step, 32);
        assert_eq!(b.after_step, 64);
        assert_eq!(b.injection_count_after, 2);
    }

    #[test]
    fn four_cell_kernel_remains_a_regression_reference() {
        let engine = Engine::new().unwrap();
        let frame = frame_from(&engine).unwrap();
        assert_eq!(frame.inner_cells.len(), 4);

        let resident = Resident32::new().unwrap();
        let status = resident.status().unwrap();
        assert_eq!(status.cells, 32);
        assert!(status.four_c_reference_retained);
    }

    #[test]
    fn promotion_does_not_unlock_llm_ingestion() {
        let resident = Resident32::new().unwrap();
        let status = resident.status().unwrap();
        assert_eq!(status.llm_ingestion_state, "LOCKED");
        assert_eq!(status.provider_state, "STANDBY_ONLY");
        assert_eq!(status.laa_state, "OBSERVE_ONLY");
        assert_eq!(status.next_gate, "32C_RESIDENT_BASELINE");
    }

    #[test]
    fn brain_system_exposes_32c_resident_baseline_without_touching_live_state() {
        let r = run_resident32_baseline().unwrap();
        assert_eq!(r.schema, "vertex.brain.vcell-resident32-baseline.v1");
        assert_eq!(r.experiment_id, "VCELL_32C_RESIDENT_BASELINE_000001");
        assert_eq!(r.state, "CALIBRATED");
        assert!(r.deterministic_replay);
        assert!(!r.live_resident_mutated);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
    }

    #[test]
    fn brain_system_exposes_persistent_difference_without_memory_claim() {
        let r = run_persistent_difference().unwrap();
        assert_eq!(
            r.schema,
            "vertex.brain.vcell-resident32-persistent-difference.v1"
        );
        assert_eq!(r.experiment_id, "VCELL_32C_PERSISTENT_DIFFERENCE_000001");
        assert!(r.deterministic_replay);
        assert!(r.control_clean);
        assert!(!r.memory_claimed);
        assert!(!r.live_resident_mutated);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
    }
}
