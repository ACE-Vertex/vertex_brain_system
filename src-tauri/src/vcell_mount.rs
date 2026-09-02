use crate::resident32::Resident32Status;
use serde::Serialize;

pub const VCELL_MOUNT_CONTRACT: &str = "vertex.brain.vcell-mount.v2";
pub const VCELL_MOUNT_VERSION: &str = "0.2.0";

#[derive(Debug, Clone, Serialize)]
pub struct VcellMountCapability {
    pub name: &'static str,
    pub direction: &'static str,
    pub authority: &'static str,
    pub state: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcellMountView {
    pub contract: &'static str,
    pub version: &'static str,
    pub state: &'static str,
    pub residency: &'static str,
    pub role: &'static str,
    pub resident_id: &'static str,
    pub lineage_origin: &'static str,
    pub reference_scope: &'static str,
    pub promotion_evidence: &'static str,
    pub architecture: &'static str,
    pub seed_transport: &'static str,
    pub seed_bytes: usize,
    pub current_seed_hex: String,
    pub resident_cells: usize,
    pub vertices: usize,
    pub links: usize,
    pub contact_points: usize,
    pub phase: String,
    pub persistent_state: bool,
    pub geometry_mutable: bool,
    pub contact_observable: bool,
    pub time_sensitive: bool,
    pub history_sensitive: bool,
    pub laa_transfer_ingress: &'static str,
    pub automatic_transplant: bool,
    pub direct_world_mutation: bool,
    pub capabilities: Vec<VcellMountCapability>,
}

pub fn vcell_mount_view_32c(s: &Resident32Status) -> VcellMountView {
    VcellMountView {
        contract: VCELL_MOUNT_CONTRACT,
        version: VCELL_MOUNT_VERSION,
        state: "ENTHRONED",
        residency: "PERSISTENT_RESIDENT_CORE",
        role: "NATIVE_COGNITIVE_SUBSTRATE",
        resident_id: "REGION 000032",
        lineage_origin: "GENESIS 000001",
        reference_scope: "4C_REGRESSION_REFERENCE",
        promotion_evidence: "VCELL_32C_SCALE_EXPANSION_000001",
        architecture: "vCELL 32C Region / Nested Adaptive Configuration",
        seed_transport: "VXN_MICRO_SEED_24BIT",
        seed_bytes: s.seed_bytes,
        current_seed_hex: s.seed_hex.clone(),
        resident_cells: s.cells,
        vertices: s.vertices,
        links: s.links,
        contact_points: s.contact_points,
        phase: s.phase.clone(),
        persistent_state: s.persistent_dynamic_state,
        geometry_mutable: true,
        contact_observable: true,
        time_sensitive: true,
        history_sensitive: true,
        laa_transfer_ingress: "CANDIDATE_GATED",
        automatic_transplant: false,
        direct_world_mutation: false,
        capabilities: vec![
            VcellMountCapability {
                name: "hold_32c_persistent_state",
                direction: "vCELL <-> Brain",
                authority: "RESIDENT",
                state: "ACTIVE",
            },
            VcellMountCapability {
                name: "observe_32c_contact_region",
                direction: "vCELL -> Brain",
                authority: "READ",
                state: "ACTIVE",
            },
            VcellMountCapability {
                name: "pulse_resident_state",
                direction: "Brain -> vCELL",
                authority: "EXPERIMENT",
                state: "ACTIVE",
            },
            VcellMountCapability {
                name: "retain_4c_regression_reference",
                direction: "Reference -> Brain",
                authority: "READ",
                state: "ACTIVE",
            },
            VcellMountCapability {
                name: "receive_laa_transfer_candidate",
                direction: "LAA -> Brain -> vCELL",
                authority: "CANDIDATE_ONLY",
                state: "GATED",
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resident32::Resident32;

    #[test]
    fn mount_v2_is_32c() {
        let r = Resident32::new().unwrap();
        let s = r.status().unwrap();
        let m = vcell_mount_view_32c(&s);
        assert_eq!(m.contract, "vertex.brain.vcell-mount.v2");
        assert_eq!(m.resident_id, "REGION 000032");
        assert_eq!(m.resident_cells, 32);
        assert_eq!(m.vertices, 768);
        assert_eq!(m.reference_scope, "4C_REGRESSION_REFERENCE");
    }

    #[test]
    fn transfer_remains_gated() {
        let r = Resident32::new().unwrap();
        let s = r.status().unwrap();
        let m = vcell_mount_view_32c(&s);
        assert_eq!(m.laa_transfer_ingress, "CANDIDATE_GATED");
        assert!(!m.automatic_transplant);
        assert!(!m.direct_world_mutation);
    }
}
