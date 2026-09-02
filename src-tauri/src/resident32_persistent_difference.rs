use crate::resident32::{Resident32, Resident32PassiveRead};
use serde::Serialize;

pub const PERSISTENT_DIFF_CONTRACT: &str = "vertex.brain.vcell-resident32-persistent-difference.v1";
pub const PERSISTENT_DIFF_EXPERIMENT: &str = "VCELL_32C_PERSISTENT_DIFFERENCE_000001";

pub const WRITE_STEPS: usize = 128;
pub const HORIZONS: [usize; 5] = [256, 512, 1024, 2048, 4096];
pub const ENERGY_EPSILON: f64 = 1.0e-12;
pub const PLATEAU_RELATIVE_DELTA_MAX: f64 = 0.05;

#[derive(Debug, Clone, Serialize)]
pub struct HorizonSample {
    pub cumulative_settle_steps: usize,
    pub state: Resident32PassiveRead,
    pub energy_ratio_vs_first: f64,
    pub amplitude_ratio_vs_first: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DifferenceRun {
    pub run_id: &'static str,
    pub reset: Resident32PassiveRead,
    pub write_fingerprint: String,
    pub after_write_step: u64,
    pub samples: Vec<HorizonSample>,
    pub final_state_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlRun {
    pub reset: Resident32PassiveRead,
    pub samples: Vec<HorizonSample>,
    pub remained_zero: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistentDifferenceReport {
    pub schema: &'static str,
    pub experiment_id: &'static str,
    pub state: &'static str,
    pub classification: &'static str,
    pub resident_scope: &'static str,
    pub calibration_lane: &'static str,
    pub live_resident_mutated: bool,
    pub cells: usize,
    pub vertices: usize,
    pub seed_bytes: usize,
    pub write_steps: usize,
    pub horizons: Vec<usize>,
    pub run_a: DifferenceRun,
    pub run_b: DifferenceRun,
    pub zero_write_control: ControlRun,
    pub deterministic_replay: bool,
    pub control_clean: bool,
    pub monotonic_energy_decay: bool,
    pub monotonic_amplitude_decay: bool,
    pub first_energy: f64,
    pub final_energy: f64,
    pub final_energy_ratio: f64,
    pub final_amplitude_ratio: f64,
    pub tail_relative_energy_delta: f64,
    pub tail_relative_amplitude_delta: f64,
    pub persistent_difference_at_final_horizon: bool,
    pub plateau_candidate: bool,
    pub non_monotonic_behavior: bool,
    pub experiment_fingerprint: String,
    pub memory_claimed: bool,
    pub llm_ingestion_state: &'static str,
    pub provider_state: &'static str,
    pub laa_state: &'static str,
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

fn ratio(value: f64, base: f64) -> f64 {
    if base.abs() <= ENERGY_EPSILON {
        if value.abs() <= ENERGY_EPSILON {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        value / base
    }
}

fn rel_delta(a: f64, b: f64) -> f64 {
    let denom = a.abs().max(b.abs()).max(ENERGY_EPSILON);
    (a - b).abs() / denom
}

fn run_written(run_id: &'static str) -> Result<DifferenceRun, String> {
    let mut resident = Resident32::new()?;
    let reset = resident.passive_read();
    let write = resident.probe(Some(WRITE_STEPS))?;

    let mut samples = Vec::with_capacity(HORIZONS.len());
    let mut cumulative = 0usize;
    let mut first_energy = None;
    let mut first_amp = None;

    for horizon in HORIZONS {
        let delta = horizon - cumulative;
        let state = resident.settle(delta)?;
        cumulative = horizon;

        let e0 = *first_energy.get_or_insert(state.total_signal_energy);
        let a0 = *first_amp.get_or_insert(state.peak_amplitude);

        samples.push(HorizonSample {
            cumulative_settle_steps: cumulative,
            energy_ratio_vs_first: ratio(state.total_signal_energy, e0),
            amplitude_ratio_vs_first: ratio(state.peak_amplitude, a0),
            state,
        });
    }

    let final_state_fingerprint = samples
        .last()
        .map(|s| s.state.state_fingerprint.clone())
        .unwrap_or_default();

    Ok(DifferenceRun {
        run_id,
        reset,
        write_fingerprint: write.response_fingerprint,
        after_write_step: write.after_step,
        samples,
        final_state_fingerprint,
    })
}

fn run_control() -> Result<ControlRun, String> {
    let mut resident = Resident32::new()?;
    let reset = resident.passive_read();

    let mut samples = Vec::with_capacity(HORIZONS.len());
    let mut cumulative = 0usize;

    for horizon in HORIZONS {
        let delta = horizon - cumulative;
        let state = resident.settle(delta)?;
        cumulative = horizon;

        samples.push(HorizonSample {
            cumulative_settle_steps: cumulative,
            energy_ratio_vs_first: 0.0,
            amplitude_ratio_vs_first: 0.0,
            state,
        });
    }

    let remained_zero = samples.iter().all(|s| {
        s.state.total_signal_energy.abs() <= ENERGY_EPSILON
            && s.state.peak_amplitude.abs() <= ENERGY_EPSILON
            && s.state.injection_count == 0
    });

    Ok(ControlRun {
        reset,
        samples,
        remained_zero,
    })
}

fn monotonic_nonincreasing(values: &[f64]) -> bool {
    values.windows(2).all(|w| w[1] <= w[0] + ENERGY_EPSILON)
}

fn fingerprint(
    a: &DifferenceRun,
    b: &DifferenceRun,
    control: &ControlRun,
    classification: &str,
) -> String {
    let mut material = format!(
        "class={classification}|writeA={}|writeB={}|control={}|",
        a.write_fingerprint, b.write_fingerprint, control.remained_zero
    );
    for sample in &a.samples {
        material.push_str(&format!(
            "{}:{:.12}:{:.12}:{}|",
            sample.cumulative_settle_steps,
            sample.state.total_signal_energy,
            sample.state.peak_amplitude,
            sample.state.state_fingerprint,
        ));
    }
    format!("VR32PD1-{:016X}", fnv1a64(material.as_bytes()))
}

pub fn run_persistent_difference() -> Result<PersistentDifferenceReport, String> {
    let run_a = run_written("A")?;
    let run_b = run_written("B")?;
    let zero_write_control = run_control()?;

    let deterministic_replay = run_a.write_fingerprint == run_b.write_fingerprint
        && run_a.samples.len() == run_b.samples.len()
        && run_a
            .samples
            .iter()
            .zip(run_b.samples.iter())
            .all(|(a, b)| {
                a.state.state_fingerprint == b.state.state_fingerprint
                    && a.state.total_signal_energy == b.state.total_signal_energy
                    && a.state.peak_amplitude == b.state.peak_amplitude
            });

    let energies: Vec<f64> = run_a
        .samples
        .iter()
        .map(|s| s.state.total_signal_energy)
        .collect();
    let amplitudes: Vec<f64> = run_a
        .samples
        .iter()
        .map(|s| s.state.peak_amplitude)
        .collect();

    let monotonic_energy_decay = monotonic_nonincreasing(&energies);
    let monotonic_amplitude_decay = monotonic_nonincreasing(&amplitudes);
    let non_monotonic_behavior = !(monotonic_energy_decay && monotonic_amplitude_decay);

    let first_energy = *energies.first().unwrap_or(&0.0);
    let final_energy = *energies.last().unwrap_or(&0.0);
    let first_amp = *amplitudes.first().unwrap_or(&0.0);
    let final_amp = *amplitudes.last().unwrap_or(&0.0);

    let final_energy_ratio = ratio(final_energy, first_energy);
    let final_amplitude_ratio = ratio(final_amp, first_amp);

    let tail_relative_energy_delta = if energies.len() >= 2 {
        rel_delta(energies[energies.len() - 2], energies[energies.len() - 1])
    } else {
        0.0
    };

    let tail_relative_amplitude_delta = if amplitudes.len() >= 2 {
        rel_delta(
            amplitudes[amplitudes.len() - 2],
            amplitudes[amplitudes.len() - 1],
        )
    } else {
        0.0
    };

    let control_clean = zero_write_control.remained_zero;
    let persistent_difference_at_final_horizon =
        final_energy.abs() > ENERGY_EPSILON || final_amp.abs() > ENERGY_EPSILON;

    let plateau_candidate = persistent_difference_at_final_horizon
        && tail_relative_energy_delta <= PLATEAU_RELATIVE_DELTA_MAX
        && tail_relative_amplitude_delta <= PLATEAU_RELATIVE_DELTA_MAX
        && !non_monotonic_behavior;

    let classification = if !control_clean {
        "CONTROL_CONTAMINATED"
    } else if !persistent_difference_at_final_horizon {
        "RELAXED_TO_BASELINE"
    } else if non_monotonic_behavior {
        "NON_MONOTONIC_PERSISTENT_RESPONSE"
    } else if plateau_candidate {
        "PERSISTENT_PLATEAU_CANDIDATE"
    } else {
        "DECAYING_RESIDUAL"
    };

    let state = if deterministic_replay && control_clean {
        "OBSERVED"
    } else {
        "OBSERVE"
    };
    let experiment_fingerprint = fingerprint(&run_a, &run_b, &zero_write_control, classification);

    Ok(PersistentDifferenceReport {
        schema: PERSISTENT_DIFF_CONTRACT,
        experiment_id: PERSISTENT_DIFF_EXPERIMENT,
        state,
        classification,
        resident_scope: "32C_PERSISTENT_RESIDENT_CORE",
        calibration_lane: "ISOLATED_SAME_TOPOLOGY",
        live_resident_mutated: false,
        cells: 32,
        vertices: 768,
        seed_bytes: 3,
        write_steps: WRITE_STEPS,
        horizons: HORIZONS.to_vec(),
        run_a,
        run_b,
        zero_write_control,
        deterministic_replay,
        control_clean,
        monotonic_energy_decay,
        monotonic_amplitude_decay,
        first_energy,
        final_energy,
        final_energy_ratio,
        final_amplitude_ratio,
        tail_relative_energy_delta,
        tail_relative_amplitude_delta,
        persistent_difference_at_final_horizon,
        plateau_candidate,
        non_monotonic_behavior,
        experiment_fingerprint,
        memory_claimed: false,
        llm_ingestion_state: "LOCKED",
        provider_state: "STANDBY_ONLY",
        laa_state: "OBSERVE_ONLY",
        next_gate: "32C_RECALL_CANDIDATE",
        interpretation: "Written response is tracked across long passive settling horizons against a zero-write control. Persistent plateau is only a candidate dynamic phenomenon. Memory is explicitly not claimed.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_difference_replays_exactly() {
        let r = run_persistent_difference().unwrap();
        assert_eq!(r.state, "OBSERVED");
        assert!(r.deterministic_replay);
        assert!(r.control_clean);
    }

    #[test]
    fn zero_write_control_remains_clean() {
        let r = run_persistent_difference().unwrap();
        assert!(r.zero_write_control.remained_zero);
        assert!(r.zero_write_control.samples.iter().all(|s| {
            s.state.injection_count == 0 && s.state.total_signal_energy.abs() <= ENERGY_EPSILON
        }));
    }

    #[test]
    fn experiment_never_claims_memory() {
        let r = run_persistent_difference().unwrap();
        assert!(!r.memory_claimed);
        assert_eq!(r.llm_ingestion_state, "LOCKED");
    }

    #[test]
    fn horizon_schedule_is_strictly_increasing() {
        assert_eq!(HORIZONS, [256, 512, 1024, 2048, 4096]);
        assert!(HORIZONS.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn final_classification_is_bounded() {
        let r = run_persistent_difference().unwrap();
        assert!(matches!(
            r.classification,
            "CONTROL_CONTAMINATED"
                | "RELAXED_TO_BASELINE"
                | "NON_MONOTONIC_PERSISTENT_RESPONSE"
                | "PERSISTENT_PLATEAU_CANDIDATE"
                | "DECAYING_RESIDUAL"
        ));
    }

    #[test]
    fn evidence_prints_persistent_difference_result() {
        let r = run_persistent_difference().unwrap();

        println!("=== 32C PERSISTENT DIFFERENCE RESULT ===");
        println!("EXPERIMENT {}", r.experiment_id);
        println!("STATE {}", r.state);
        println!("CLASSIFICATION {}", r.classification);
        println!("CELLS {}", r.cells);
        println!("VERTICES {}", r.vertices);
        println!("VXN_BYTES {}", r.seed_bytes);
        println!("WRITE_STEPS {}", r.write_steps);

        for sample in &r.run_a.samples {
            println!(
                "H{} ENERGY {:.15e} AMPLITUDE {:.15e} ACTIVE_CELLS {} STATE_FP {}",
                sample.cumulative_settle_steps,
                sample.state.total_signal_energy,
                sample.state.peak_amplitude,
                sample.state.active_cells,
                sample.state.state_fingerprint
            );
        }

        println!("FIRST_ENERGY {:.15e}", r.first_energy);
        println!("FINAL_ENERGY {:.15e}", r.final_energy);
        println!("FINAL_ENERGY_RATIO {:.15e}", r.final_energy_ratio);
        println!("FINAL_AMPLITUDE_RATIO {:.15e}", r.final_amplitude_ratio);
        println!(
            "TAIL_RELATIVE_ENERGY_DELTA {:.15e}",
            r.tail_relative_energy_delta
        );
        println!(
            "TAIL_RELATIVE_AMPLITUDE_DELTA {:.15e}",
            r.tail_relative_amplitude_delta
        );
        println!("ZERO_WRITE_CONTROL_CLEAN {}", r.control_clean);
        println!("REPLAY_EXACT {}", r.deterministic_replay);
        println!(
            "PERSISTENT_DIFFERENCE_AT_FINAL_HORIZON {}",
            r.persistent_difference_at_final_horizon
        );
        println!("PLATEAU_CANDIDATE {}", r.plateau_candidate);
        println!("NON_MONOTONIC {}", r.non_monotonic_behavior);
        println!("MEMORY_CLAIM {}", r.memory_claimed);
        println!("LLM_INGESTION {}", r.llm_ingestion_state);
        println!("NEXT_GATE {}", r.next_gate);
        println!("FINGERPRINT {}", r.experiment_fingerprint);
        println!("=== END 32C PERSISTENT DIFFERENCE RESULT ===");

        assert!(r.deterministic_replay);
        assert!(r.control_clean);
        assert!(!r.memory_claimed);
    }
}
