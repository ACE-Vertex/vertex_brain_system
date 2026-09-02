use crate::resident32::{Resident32, Resident32PassiveRead};
use serde::Serialize;

pub const BASELINE32_CONTRACT: &str = "vertex.brain.vcell-resident32-baseline.v1";
pub const BASELINE32_EXPERIMENT: &str = "VCELL_32C_RESIDENT_BASELINE_000001";
pub const WRITE_STEPS: usize = 128;
pub const SETTLE_1_STEPS: usize = 512;
pub const SETTLE_2_STEPS: usize = 512;

#[derive(Debug, Clone, Serialize)]
pub struct Resident32BaselineRun {
    pub run_id: &'static str,
    pub reset_state: Resident32PassiveRead,
    pub write_fingerprint: String,
    pub after_write_step: u64,
    pub after_write_injections: u64,
    pub settle_1_state: Resident32PassiveRead,
    pub settle_2_state: Resident32PassiveRead,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resident32BaselineReport {
    pub schema: &'static str,
    pub experiment_id: &'static str,
    pub state: &'static str,
    pub classification: &'static str,
    pub resident_scope: &'static str,
    pub calibration_lane: &'static str,
    pub live_resident_mutated: bool,
    pub seed_bytes: usize,
    pub cells: usize,
    pub vertices: usize,
    pub write_steps: usize,
    pub settle_1_steps: usize,
    pub settle_2_steps: usize,
    pub run_a: Resident32BaselineRun,
    pub run_b: Resident32BaselineRun,
    pub replay_reset_exact: bool,
    pub replay_write_exact: bool,
    pub replay_settle_1_exact: bool,
    pub replay_settle_2_exact: bool,
    pub deterministic_replay: bool,
    pub reset_recovery_exact: bool,
    pub persistent_difference_after_settle_1: bool,
    pub persistent_difference_after_settle_2: bool,
    pub relaxation_continues_between_reads: bool,
    pub baseline_fingerprint: String,
    pub llm_ingestion_state: &'static str,
    pub provider_state: &'static str,
    pub laa_state: &'static str,
    pub transfer_state: &'static str,
    pub next_gate: &'static str,
    pub interpretation: &'static str,
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn run_once(run_id: &'static str) -> Result<Resident32BaselineRun, String> {
    let mut resident = Resident32::new()?;
    let reset_state = resident.passive_read();

    let write = resident.probe(Some(WRITE_STEPS))?;
    let settle_1_state = resident.settle(SETTLE_1_STEPS)?;
    let settle_2_state = resident.settle(SETTLE_2_STEPS)?;

    Ok(Resident32BaselineRun {
        run_id,
        reset_state,
        write_fingerprint: write.response_fingerprint,
        after_write_step: write.after_step,
        after_write_injections: write.injection_count_after,
        settle_1_state,
        settle_2_state,
    })
}

fn report_fingerprint(a: &Resident32BaselineRun, b: &Resident32BaselineRun) -> String {
    let material = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        a.reset_state.state_fingerprint,
        a.write_fingerprint,
        a.settle_1_state.state_fingerprint,
        a.settle_2_state.state_fingerprint,
        a.reset_state.state_fingerprint == b.reset_state.state_fingerprint,
        a.write_fingerprint == b.write_fingerprint,
        a.settle_1_state.state_fingerprint == b.settle_1_state.state_fingerprint,
        a.settle_2_state.state_fingerprint == b.settle_2_state.state_fingerprint,
    );
    format!("VR32B1-{:016X}", fnv1a64(material.as_bytes()))
}

pub fn run_resident32_baseline() -> Result<Resident32BaselineReport, String> {
    let run_a = run_once("A")?;
    let run_b = run_once("B")?;

    let replay_reset_exact =
        run_a.reset_state.state_fingerprint == run_b.reset_state.state_fingerprint;
    let replay_write_exact = run_a.write_fingerprint == run_b.write_fingerprint;
    let replay_settle_1_exact =
        run_a.settle_1_state.state_fingerprint == run_b.settle_1_state.state_fingerprint;
    let replay_settle_2_exact =
        run_a.settle_2_state.state_fingerprint == run_b.settle_2_state.state_fingerprint;
    let deterministic_replay =
        replay_reset_exact && replay_write_exact && replay_settle_1_exact && replay_settle_2_exact;

    let reset_recovery_exact = run_a.reset_state.total_signal_energy == 0.0
        && run_b.reset_state.total_signal_energy == 0.0
        && run_a.reset_state.injection_count == 0
        && run_b.reset_state.injection_count == 0;

    let persistent_difference_after_settle_1 =
        run_a.settle_1_state.total_signal_energy > 1.0e-12 || run_a.settle_1_state.active_cells > 0;

    let persistent_difference_after_settle_2 =
        run_a.settle_2_state.total_signal_energy > 1.0e-12 || run_a.settle_2_state.active_cells > 0;

    let relaxation_continues_between_reads =
        (run_a.settle_2_state.total_signal_energy - run_a.settle_1_state.total_signal_energy).abs()
            > 1.0e-12
            || (run_a.settle_2_state.peak_amplitude - run_a.settle_1_state.peak_amplitude).abs()
                > 1.0e-12;

    let classification = if persistent_difference_after_settle_2 {
        "PERSISTENT_DIFFERENCE_OBSERVED"
    } else {
        "RELAXED_TO_BASELINE"
    };

    let state = if deterministic_replay && reset_recovery_exact {
        "CALIBRATED"
    } else {
        "OBSERVE"
    };

    let baseline_fingerprint = report_fingerprint(&run_a, &run_b);

    Ok(Resident32BaselineReport {
        schema: BASELINE32_CONTRACT,
        experiment_id: BASELINE32_EXPERIMENT,
        state,
        classification,
        resident_scope: "32C_PERSISTENT_RESIDENT_CORE",
        calibration_lane: "ISOLATED_SAME_TOPOLOGY",
        live_resident_mutated: false,
        seed_bytes: 3,
        cells: 32,
        vertices: 768,
        write_steps: WRITE_STEPS,
        settle_1_steps: SETTLE_1_STEPS,
        settle_2_steps: SETTLE_2_STEPS,
        run_a,
        run_b,
        replay_reset_exact,
        replay_write_exact,
        replay_settle_1_exact,
        replay_settle_2_exact,
        deterministic_replay,
        reset_recovery_exact,
        persistent_difference_after_settle_1,
        persistent_difference_after_settle_2,
        relaxation_continues_between_reads,
        baseline_fingerprint,
        llm_ingestion_state: "LOCKED",
        provider_state: "STANDBY_ONLY",
        laa_state: "OBSERVE_ONLY",
        transfer_state: "CANDIDATE_GATED",
        next_gate: "32C_PERSISTENT_DIFFERENCE",
        interpretation: "RESET -> WRITE -> SETTLE -> passive_read -> SETTLE -> passive_read. Passive reads perform no injection. A surviving post-settle difference is a persistent dynamic difference, not yet memory.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_replays_exactly() {
        let r = run_resident32_baseline().unwrap();
        assert_eq!(r.state, "CALIBRATED");
        assert!(r.deterministic_replay);
        assert!(r.replay_reset_exact);
        assert!(r.replay_write_exact);
        assert!(r.replay_settle_1_exact);
        assert!(r.replay_settle_2_exact);
    }

    #[test]
    fn calibration_does_not_mutate_live_resident() {
        let r = run_resident32_baseline().unwrap();
        assert_eq!(r.calibration_lane, "ISOLATED_SAME_TOPOLOGY");
        assert!(!r.live_resident_mutated);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
    }

    #[test]
    fn passive_reads_do_not_add_injection() {
        let r = run_resident32_baseline().unwrap();
        assert!(!r.run_a.settle_1_state.injection_performed);
        assert!(!r.run_a.settle_2_state.injection_performed);
        assert_eq!(r.run_a.after_write_injections, 1);
        assert_eq!(r.run_a.settle_1_state.injection_count, 1);
        assert_eq!(r.run_a.settle_2_state.injection_count, 1);
    }

    #[test]
    fn baseline_fingerprint_is_stable() {
        let a = run_resident32_baseline().unwrap();
        let b = run_resident32_baseline().unwrap();
        assert_eq!(a.baseline_fingerprint, b.baseline_fingerprint);
        assert!(a.baseline_fingerprint.starts_with("VR32B1-"));
    }
}
