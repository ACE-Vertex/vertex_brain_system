use serde::Serialize;
use vcell_region_flow_000009::{
    RegionWave, VxnMicroSeed, ARRIVAL_THRESHOLD, CELL_COUNT, INTRA_CELL_BONDS, NODE_COUNT,
};

pub const RESIDENT32_CONTRACT: &str = "vertex.brain.vcell-resident32.v1";
pub const RESIDENT32_PROMOTION: &str = "VCELL_32C_RESIDENT_PROMOTION_000001";
pub const DEFAULT_RESIDENT_PROBE_STEPS: usize = 256;
pub const MAX_RESIDENT_PROBE_STEPS: usize = 4096;

#[derive(Debug)]
pub struct Resident32 {
    wave: RegionWave,
    phase: String,
    probe_epoch: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resident32Status {
    pub schema: &'static str,
    pub promotion_id: &'static str,
    pub state: &'static str,
    pub residency: &'static str,
    pub role: &'static str,
    pub resident_id: &'static str,
    pub lineage_origin: &'static str,
    pub reference_scope: &'static str,
    pub promotion_evidence: &'static str,
    pub cells: usize,
    pub vertices: usize,
    pub intra_cell_bonds: usize,
    pub links: usize,
    pub contact_points: usize,
    pub root_degree: usize,
    pub shell_counts: [usize; 3],
    pub seed_hex: String,
    pub seed_bytes: usize,
    pub gate_tilt_deg: f64,
    pub phase: String,
    pub step_index: u64,
    pub time: f64,
    pub injection_count: u64,
    pub probe_epoch: u64,
    pub active_cells: usize,
    pub total_signal_energy: f64,
    pub peak_amplitude: f64,
    pub persistent_dynamic_state: bool,
    pub history_carried_between_probes: bool,
    pub four_c_reference_retained: bool,
    pub llm_ingestion_state: &'static str,
    pub provider_state: &'static str,
    pub laa_state: &'static str,
    pub next_gate: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resident32ProbeReport {
    pub schema: &'static str,
    pub resident_id: &'static str,
    pub phase: &'static str,
    pub probe_epoch: u64,
    pub steps: usize,
    pub before_step: u64,
    pub after_step: u64,
    pub injection_count_before: u64,
    pub injection_count_after: u64,
    pub response_fingerprint: String,
    pub reached_cells: usize,
    pub active_cells_end: usize,
    pub total_signal_energy_end: f64,
    pub peak_amplitude_end: f64,
    pub dominant_cell: u32,
    pub dominant_share: f64,
    pub shell_peak: [f64; 3],
    pub cell_peak_share: Vec<f64>,
    pub arrival_step_by_cell: Vec<Option<u64>>,
    pub persistent_history: bool,
    pub llm_ingestion_state: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Resident32PassiveRead {
    pub schema: &'static str,
    pub resident_id: &'static str,
    pub phase: String,
    pub step_index: u64,
    pub time: f64,
    pub injection_count: u64,
    pub probe_epoch: u64,
    pub active_cells: usize,
    pub total_signal_energy: f64,
    pub peak_amplitude: f64,
    pub cell_energy: Vec<f64>,
    pub shell_energy: [f64; 3],
    pub state_fingerprint: String,
    pub passive_read: bool,
    pub injection_performed: bool,
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn response_fingerprint(
    shares: &[f64],
    shell_peak: [f64; 3],
    arrivals: &[Option<u64>],
    energy: f64,
    amplitude: f64,
) -> String {
    let mut material = String::new();
    for value in shares {
        material.push_str(&format!("{value:.12};"));
    }
    material.push('|');
    for value in shell_peak {
        material.push_str(&format!("{value:.12};"));
    }
    material.push('|');
    for arrival in arrivals {
        match arrival {
            Some(step) => material.push_str(&format!("{step};")),
            None => material.push_str("N;"),
        }
    }
    material.push('|');
    material.push_str(&format!("energy={energy:.12};amp={amplitude:.12}"));
    format!("V32R1-{:016X}", fnv1a64(material.as_bytes()))
}

fn passive_state_fingerprint(
    step_index: u64,
    injection_count: u64,
    cell_energy: &[f64],
    shell_energy: [f64; 3],
    total_signal_energy: f64,
    peak_amplitude: f64,
) -> String {
    let mut material = format!(
        "step={step_index};inj={injection_count};energy={total_signal_energy:.12};amp={peak_amplitude:.12}|"
    );
    for value in cell_energy {
        material.push_str(&format!("{value:.12};"));
    }
    material.push('|');
    for value in shell_energy {
        material.push_str(&format!("{value:.12};"));
    }
    format!("V32S0-{:016X}", fnv1a64(material.as_bytes()))
}

impl Resident32 {
    pub fn new() -> Result<Self, String> {
        let seed = VxnMicroSeed::default();
        if seed.encode().len() != 3 {
            return Err("32C Resident requires 3-byte VXN".into());
        }
        let wave = RegionWave::new(0.0, seed)?;
        wave.medium.validate()?;
        wave.medium.region.validate()?;
        if wave.medium.cells.len() != CELL_COUNT || wave.medium.nodes.len() != NODE_COUNT {
            return Err("32C Resident topology mismatch".into());
        }
        Ok(Self {
            wave,
            phase: "ENTHRONED_READY".into(),
            probe_epoch: 0,
        })
    }

    pub fn status(&self) -> Result<Resident32Status, String> {
        let metrics = self.wave.medium.region.metrics()?;
        let snap = self.wave.snapshot();
        Ok(Resident32Status {
            schema: RESIDENT32_CONTRACT,
            promotion_id: RESIDENT32_PROMOTION,
            state: "ENTHRONED",
            residency: "PERSISTENT_RESIDENT_CORE",
            role: "NATIVE_COGNITIVE_SUBSTRATE",
            resident_id: "REGION 000032",
            lineage_origin: "GENESIS 000001",
            reference_scope: "4C_REGRESSION_REFERENCE",
            promotion_evidence: "VCELL_32C_SCALE_EXPANSION_000001",
            cells: metrics.cell_count,
            vertices: self.wave.medium.nodes.len(),
            intra_cell_bonds: INTRA_CELL_BONDS,
            links: metrics.link_count,
            contact_points: metrics.contact_points,
            root_degree: metrics.root_degree,
            shell_counts: metrics.shell_counts,
            seed_hex: self.wave.seed.hex(),
            seed_bytes: self.wave.seed.encode().len(),
            gate_tilt_deg: self.wave.medium.gate_tilt_deg,
            phase: self.phase.clone(),
            step_index: self.wave.step_index,
            time: self.wave.time,
            injection_count: self.wave.injection_count,
            probe_epoch: self.probe_epoch,
            active_cells: snap.active_cells,
            total_signal_energy: snap.total_signal_energy,
            peak_amplitude: snap.peak_amplitude,
            persistent_dynamic_state: true,
            history_carried_between_probes: true,
            four_c_reference_retained: true,
            llm_ingestion_state: "LOCKED",
            provider_state: "STANDBY_ONLY",
            laa_state: "OBSERVE_ONLY",
            next_gate: "32C_RESIDENT_BASELINE",
        })
    }

    pub fn probe(&mut self, steps: Option<usize>) -> Result<Resident32ProbeReport, String> {
        let steps = steps.unwrap_or(DEFAULT_RESIDENT_PROBE_STEPS);
        if steps == 0 || steps > MAX_RESIDENT_PROBE_STEPS {
            return Err(format!(
                "probe steps must be 1..={MAX_RESIDENT_PROBE_STEPS}"
            ));
        }

        let before_step = self.wave.step_index;
        let injection_count_before = self.wave.injection_count;
        self.wave.inject();

        let mut peaks = vec![0.0_f64; CELL_COUNT];
        let mut shell_peak = [0.0_f64; 3];
        let mut arrivals = vec![None; CELL_COUNT];
        let mut end = self.wave.snapshot();

        for local_step in 1..=steps {
            end = self.wave.step();
            for (slot, energy) in end.cell_energy.iter().copied().enumerate() {
                peaks[slot] = peaks[slot].max(energy);
                if energy > ARRIVAL_THRESHOLD && arrivals[slot].is_none() {
                    arrivals[slot] = Some(local_step as u64);
                }
            }
            for shell in 0..3 {
                shell_peak[shell] = shell_peak[shell].max(end.shell_energy[shell]);
            }
        }

        let total = peaks.iter().sum::<f64>();
        let shares: Vec<f64> = if total > 0.0 {
            peaks.iter().map(|value| value / total).collect()
        } else {
            vec![0.0; CELL_COUNT]
        };

        let mut ranked: Vec<(u32, f64)> = shares
            .iter()
            .copied()
            .enumerate()
            .map(|(slot, share)| ((slot + 1) as u32, share))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let (dominant_cell, dominant_share) = ranked.first().copied().unwrap_or((0, 0.0));

        let reached_cells = arrivals.iter().filter(|step| step.is_some()).count();
        let fingerprint = response_fingerprint(
            &shares,
            shell_peak,
            &arrivals,
            end.total_signal_energy,
            end.peak_amplitude,
        );

        self.probe_epoch += 1;
        self.phase = "ACTIVE_PERSISTENT".into();

        Ok(Resident32ProbeReport {
            schema: "vertex.brain.vcell-resident32-probe.v1",
            resident_id: "REGION 000032",
            phase: "ACTIVE_PERSISTENT",
            probe_epoch: self.probe_epoch,
            steps,
            before_step,
            after_step: self.wave.step_index,
            injection_count_before,
            injection_count_after: self.wave.injection_count,
            response_fingerprint: fingerprint,
            reached_cells,
            active_cells_end: end.active_cells,
            total_signal_energy_end: end.total_signal_energy,
            peak_amplitude_end: end.peak_amplitude,
            dominant_cell,
            dominant_share,
            shell_peak,
            cell_peak_share: shares,
            arrival_step_by_cell: arrivals,
            persistent_history: true,
            llm_ingestion_state: "LOCKED",
        })
    }

    pub fn passive_read(&self) -> Resident32PassiveRead {
        let snap = self.wave.snapshot();
        Resident32PassiveRead {
            schema: "vertex.brain.vcell-resident32-passive-read.v1",
            resident_id: "REGION 000032",
            phase: self.phase.clone(),
            step_index: self.wave.step_index,
            time: self.wave.time,
            injection_count: self.wave.injection_count,
            probe_epoch: self.probe_epoch,
            active_cells: snap.active_cells,
            total_signal_energy: snap.total_signal_energy,
            peak_amplitude: snap.peak_amplitude,
            cell_energy: snap.cell_energy.clone(),
            shell_energy: snap.shell_energy,
            state_fingerprint: passive_state_fingerprint(
                self.wave.step_index,
                self.wave.injection_count,
                &snap.cell_energy,
                snap.shell_energy,
                snap.total_signal_energy,
                snap.peak_amplitude,
            ),
            passive_read: true,
            injection_performed: false,
        }
    }

    pub fn settle(&mut self, steps: usize) -> Result<Resident32PassiveRead, String> {
        if steps == 0 || steps > MAX_RESIDENT_PROBE_STEPS {
            return Err(format!(
                "settle steps must be 1..={MAX_RESIDENT_PROBE_STEPS}"
            ));
        }
        for _ in 0..steps {
            self.wave.step();
        }
        self.phase = "SETTLED_OBSERVE".into();
        Ok(self.passive_read())
    }

    pub fn reset_dynamic_state(&mut self) -> Result<Resident32Status, String> {
        let seed = self.wave.seed;
        self.wave = RegionWave::new(0.0, seed)?;
        self.phase = "RESET_READY".into();
        self.probe_epoch = 0;
        self.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotion_creates_real_32c_resident() {
        let r = Resident32::new().unwrap();
        let s = r.status().unwrap();
        assert_eq!(s.resident_id, "REGION 000032");
        assert_eq!(s.cells, 32);
        assert_eq!(s.vertices, 768);
        assert_eq!(s.seed_bytes, 3);
        assert_eq!(s.residency, "PERSISTENT_RESIDENT_CORE");
        assert!(s.four_c_reference_retained);
        assert_eq!(s.llm_ingestion_state, "LOCKED");
    }

    #[test]
    fn probe_carries_state_between_calls() {
        let mut r = Resident32::new().unwrap();
        let a = r.probe(Some(32)).unwrap();
        let b = r.probe(Some(32)).unwrap();
        assert_eq!(a.before_step, 0);
        assert_eq!(a.after_step, 32);
        assert_eq!(b.before_step, 32);
        assert_eq!(b.after_step, 64);
        assert_eq!(b.injection_count_after, 2);
        assert!(b.persistent_history);
        assert_eq!(b.cell_peak_share.len(), CELL_COUNT);
        assert_eq!(b.arrival_step_by_cell.len(), CELL_COUNT);
    }

    #[test]
    fn reset_clears_dynamics_not_topology() {
        let mut r = Resident32::new().unwrap();
        r.probe(Some(32)).unwrap();
        let s = r.reset_dynamic_state().unwrap();
        assert_eq!(s.cells, 32);
        assert_eq!(s.vertices, 768);
        assert_eq!(s.step_index, 0);
        assert_eq!(s.injection_count, 0);
    }
    #[test]
    fn passive_read_does_not_inject_or_advance() {
        let r = Resident32::new().unwrap();
        let a = r.passive_read();
        let b = r.passive_read();
        assert_eq!(a.step_index, b.step_index);
        assert_eq!(a.injection_count, b.injection_count);
        assert_eq!(a.state_fingerprint, b.state_fingerprint);
        assert_eq!(a.cell_energy.len(), CELL_COUNT);
        assert_eq!(a.shell_energy.len(), 3);
        assert!(a.passive_read);
        assert!(!a.injection_performed);
    }

    #[test]
    fn settle_advances_without_new_injection() {
        let mut r = Resident32::new().unwrap();
        r.probe(Some(32)).unwrap();
        let before = r.passive_read();
        let after = r.settle(64).unwrap();
        assert_eq!(after.step_index, before.step_index + 64);
        assert_eq!(after.injection_count, before.injection_count);
        assert!(!after.injection_performed);
    }
}
