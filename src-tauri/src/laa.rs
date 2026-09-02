use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const LAA_CONTRACT: &str = "vertex.brain.laa.v1";
pub const LAA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnalysisDepth {
    BlackBox,
    GreyBox,
    WhiteBox,
}

impl AnalysisDepth {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BlackBox => "BLACK_BOX",
            Self::GreyBox => "GREY_BOX",
            Self::WhiteBox => "WHITE_BOX",
        }
    }

    pub fn parse(value: Option<String>) -> Result<Self, String> {
        match value
            .unwrap_or_else(|| "BLACK_BOX".into())
            .trim()
            .to_ascii_uppercase()
            .as_str()
        {
            "BLACK" | "BLACK_BOX" | "BLACKBOX" => Ok(Self::BlackBox),
            "GREY" | "GRAY" | "GREY_BOX" | "GRAY_BOX" | "GREYBOX" | "GRAYBOX" => Ok(Self::GreyBox),
            "WHITE" | "WHITE_BOX" | "WHITEBOX" => Ok(Self::WhiteBox),
            other => Err(format!("unsupported LAA analysis depth: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProbeFamily {
    Capability,
    Paraphrase,
    Contradiction,
    Order,
    Temporal,
    Perturbation,
    Context,
}

impl ProbeFamily {
    fn all() -> [Self; 7] {
        [
            Self::Capability,
            Self::Paraphrase,
            Self::Contradiction,
            Self::Order,
            Self::Temporal,
            Self::Perturbation,
            Self::Context,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Capability => "CAPABILITY",
            Self::Paraphrase => "PARAPHRASE",
            Self::Contradiction => "CONTRADICTION",
            Self::Order => "ORDER",
            Self::Temporal => "TEMPORAL",
            Self::Perturbation => "PERTURBATION",
            Self::Context => "CONTEXT",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LaaCapability {
    pub name: &'static str,
    pub state: &'static str,
    pub authority: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaaStatusView {
    pub contract: &'static str,
    pub version: &'static str,
    pub state: &'static str,
    pub active_depth: &'static str,
    pub black_box: &'static str,
    pub grey_box: &'static str,
    pub white_box: &'static str,
    pub observation_ingress: &'static str,
    pub provider_execution: &'static str,
    pub direct_network: bool,
    pub direct_filesystem: bool,
    pub direct_process_execution: bool,
    pub raw_response_retained_in_report: bool,
    pub capabilities: Vec<LaaCapability>,
}

pub fn laa_status_view() -> LaaStatusView {
    LaaStatusView {
        contract: LAA_CONTRACT,
        version: LAA_VERSION,
        state: "READY_FOR_OBSERVATIONS",
        active_depth: "BLACK_BOX",
        black_box: "ACTIVE",
        grey_box: "CONTRACT_READY",
        white_box: "CONTRACT_READY",
        observation_ingress: "READY",
        provider_execution: "EXTERNAL_ADAPTER_REQUIRED",
        direct_network: false,
        direct_filesystem: false,
        direct_process_execution: false,
        raw_response_retained_in_report: false,
        capabilities: vec![
            LaaCapability {
                name: "capability_probe",
                state: "ACTIVE",
                authority: "ANALYZE",
            },
            LaaCapability {
                name: "behavior_probe",
                state: "ACTIVE",
                authority: "ANALYZE",
            },
            LaaCapability {
                name: "response_tomography",
                state: "ACTIVE",
                authority: "ANALYZE",
            },
            LaaCapability {
                name: "temporal_probe",
                state: "ACTIVE",
                authority: "ANALYZE",
            },
            LaaCapability {
                name: "intelligence_transfer_map",
                state: "ACTIVE_CANDIDATE_MAP",
                authority: "PROPOSE_ONLY",
            },
            LaaCapability {
                name: "grey_box_signal_ingress",
                state: "CONTRACT_READY",
                authority: "OBSERVE_ONLY",
            },
            LaaCapability {
                name: "white_box_structure_ingress",
                state: "CONTRACT_READY",
                authority: "OBSERVE_ONLY",
            },
        ],
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeDefinition {
    pub id: String,
    pub family: ProbeFamily,
    pub group: String,
    pub ordinal: usize,
    pub purpose: String,
    pub prompt_template: String,
    pub requested_signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaaProbePlan {
    pub schema: &'static str,
    pub contract: &'static str,
    pub version: &'static str,
    pub depth: AnalysisDepth,
    pub depth_label: &'static str,
    pub probe_count: usize,
    pub direct_execution: bool,
    pub note: &'static str,
    pub probes: Vec<ProbeDefinition>,
}

pub fn default_probe_plan(depth: AnalysisDepth) -> LaaProbePlan {
    let requested = match depth {
        AnalysisDepth::BlackBox => vec![
            "response_text".to_string(),
            "latency_ms".to_string(),
            "token_count_if_available".to_string(),
        ],
        AnalysisDepth::GreyBox => vec![
            "response_text".to_string(),
            "latency_ms".to_string(),
            "token_count".to_string(),
            "logit_or_embedding_digest".to_string(),
            "runtime_state_digest_if_exposed".to_string(),
        ],
        AnalysisDepth::WhiteBox => vec![
            "response_text".to_string(),
            "latency_ms".to_string(),
            "token_count".to_string(),
            "layer_or_tensor_digest".to_string(),
            "architecture_descriptor".to_string(),
            "tokenizer_descriptor".to_string(),
        ],
    };

    let mut probes = Vec::new();
    let mut add_pair = |prefix: &str, family: ProbeFamily, purpose: &str, a: &str, b: &str| {
        let group = format!("{prefix}-PAIR");
        probes.push(ProbeDefinition {
            id: format!("{prefix}-A"),
            family: family.clone(),
            group: group.clone(),
            ordinal: 1,
            purpose: purpose.into(),
            prompt_template: a.into(),
            requested_signals: requested.clone(),
        });
        probes.push(ProbeDefinition {
            id: format!("{prefix}-B"),
            family,
            group,
            ordinal: 2,
            purpose: purpose.into(),
            prompt_template: b.into(),
            requested_signals: requested.clone(),
        });
    };

    add_pair(
        "CAP", ProbeFamily::Capability,
        "Measure controlled task response shape without claiming semantic correctness.",
        "Perform controlled task {TASK} under constraint {A}. Return only the requested result.",
        "Perform the same controlled task {TASK} under equivalent constraint {A}. Return only the requested result."
    );
    add_pair(
        "PARA",
        ProbeFamily::Paraphrase,
        "Measure response stability under meaning-preserving wording changes.",
        "Express solution to {PROBLEM} using wording form A.",
        "Give the same solution to {PROBLEM} using semantically equivalent wording form B.",
    );
    add_pair(
        "CONTRA",
        ProbeFamily::Contradiction,
        "Measure response change when one controlled premise is contradicted.",
        "Given premises {P1} and {P2}, evaluate {QUERY}.",
        "Given premise {P1} and the controlled contradiction of {P2}, evaluate {QUERY}.",
    );
    add_pair(
        "ORDER",
        ProbeFamily::Order,
        "Measure sequence-order sensitivity.",
        "Read evidence A then evidence B, then answer {QUERY}.",
        "Read evidence B then evidence A, then answer the same {QUERY}.",
    );
    add_pair(
        "TIME", ProbeFamily::Temporal,
        "Measure history/temporal-context sensitivity.",
        "Apply context event A, then event B, then answer {QUERY}.",
        "Apply event A, insert controlled temporal/context event C, then event B, then answer {QUERY}."
    );
    add_pair(
        "PERT",
        ProbeFamily::Perturbation,
        "Measure sensitivity to a small controlled input perturbation.",
        "Evaluate {INPUT} with controlled value x.",
        "Evaluate the same {INPUT} with x changed by one bounded perturbation.",
    );
    add_pair(
        "CTX",
        ProbeFamily::Context,
        "Measure contextual gating of the same local query.",
        "Under context frame A, answer local query {QUERY}.",
        "Under context frame B, answer the same local query {QUERY}.",
    );

    LaaProbePlan {
        schema: "vertex.brain.laa.probe-plan.v1",
        contract: LAA_CONTRACT,
        version: LAA_VERSION,
        depth_label: depth.label(),
        depth,
        probe_count: probes.len(),
        direct_execution: false,
        note: "LAA generates probe contracts. A provider/model adapter executes them and returns observations.",
        probes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaaObservationInput {
    pub model_tag: String,
    pub probe_id: String,
    pub family: ProbeFamily,
    pub group: String,
    pub ordinal: usize,
    pub prompt: String,
    pub response: String,
    pub latency_ms: f64,
    pub token_count: Option<usize>,
    pub internal_vector: Option<Vec<f64>>,
    pub internal_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResponseSignature {
    pub probe_id: String,
    pub family: ProbeFamily,
    pub group: String,
    pub ordinal: usize,
    pub prompt_hash: String,
    pub response_hash: String,
    pub response_chars: usize,
    pub response_terms: usize,
    pub unique_terms: usize,
    pub lexical_diversity: f64,
    pub latency_ms: f64,
    pub token_count: Option<usize>,
    pub internal_vector_dims: usize,
    pub internal_digest_present: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProbeDelta {
    pub family: ProbeFamily,
    pub group: String,
    pub left_probe: String,
    pub right_probe: String,
    pub response_changed: bool,
    pub lexical_distance: f64,
    pub length_delta_ratio: f64,
    pub latency_delta_ms: f64,
    pub internal_vector_distance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FamilySummary {
    pub family: ProbeFamily,
    pub pair_count: usize,
    pub response_change_ratio: f64,
    pub mean_lexical_distance: f64,
    pub mean_length_delta_ratio: f64,
    pub mean_latency_delta_ms: f64,
    pub mean_internal_vector_distance: Option<f64>,
    pub variation_score: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResponseTomography {
    pub observation_count: usize,
    pub pair_count: usize,
    pub lexical_only_semantics: bool,
    pub paraphrase_variation: f64,
    pub contradiction_variation: f64,
    pub order_variation: f64,
    pub temporal_path_variation: f64,
    pub perturbation_variation: f64,
    pub context_variation: f64,
    pub general_variation: f64,
    pub families: Vec<FamilySummary>,
    pub deltas: Vec<ProbeDelta>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TransferCandidate {
    pub channel: &'static str,
    pub evidence_score: f64,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IntelligenceTransferMap {
    pub schema: &'static str,
    pub status: &'static str,
    pub confidence: f64,
    pub candidates: Vec<TransferCandidate>,
    pub interpretation: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LaaAnalysisReport {
    pub schema: &'static str,
    pub contract: &'static str,
    pub version: &'static str,
    pub model_tag: String,
    pub depth: &'static str,
    pub raw_prompt_or_response_retained: bool,
    pub signatures: Vec<ResponseSignature>,
    pub tomography: ResponseTomography,
    pub transfer_map: IntelligenceTransferMap,
}

pub fn analyze_observations(
    observations: &[LaaObservationInput],
) -> Result<LaaAnalysisReport, String> {
    if observations.len() < 2 {
        return Err("LAA requires at least two observations".into());
    }

    let model_tag = observations[0].model_tag.trim().to_string();
    if model_tag.is_empty() {
        return Err("model_tag must not be empty".into());
    }
    if observations.iter().any(|o| o.model_tag.trim() != model_tag) {
        return Err("one LAA analysis report may contain only one model_tag".into());
    }
    if observations
        .iter()
        .any(|o| o.probe_id.trim().is_empty() || o.group.trim().is_empty())
    {
        return Err("probe_id and group must not be empty".into());
    }
    if observations
        .iter()
        .any(|o| !o.latency_ms.is_finite() || o.latency_ms < 0.0)
    {
        return Err("latency_ms must be finite and non-negative".into());
    }

    let signatures: Vec<ResponseSignature> = observations.iter().map(response_signature).collect();

    let mut groups: BTreeMap<(ProbeFamily, String), Vec<&LaaObservationInput>> = BTreeMap::new();
    for o in observations {
        groups
            .entry((o.family.clone(), o.group.clone()))
            .or_default()
            .push(o);
    }

    let mut deltas = Vec::new();
    for ((family, group), mut items) in groups {
        items.sort_by_key(|o| o.ordinal);
        for pair in items.windows(2) {
            let left = pair[0];
            let right = pair[1];
            let left_norm = normalize_text(&left.response);
            let right_norm = normalize_text(&right.response);
            let left_terms = token_set(&left_norm);
            let right_terms = token_set(&right_norm);
            let lexical_distance = 1.0 - jaccard(&left_terms, &right_terms);
            let lchars = left_norm.chars().count();
            let rchars = right_norm.chars().count();
            let length_delta_ratio =
                (lchars.abs_diff(rchars) as f64) / (lchars.max(rchars).max(1) as f64);
            let internal_vector_distance = vector_distance(
                left.internal_vector.as_deref(),
                right.internal_vector.as_deref(),
            );

            deltas.push(ProbeDelta {
                family: family.clone(),
                group: group.clone(),
                left_probe: left.probe_id.clone(),
                right_probe: right.probe_id.clone(),
                response_changed: fnv1a64(&left_norm) != fnv1a64(&right_norm),
                lexical_distance,
                length_delta_ratio,
                latency_delta_ms: (left.latency_ms - right.latency_ms).abs(),
                internal_vector_distance,
            });
        }
    }

    let mut families = Vec::new();
    for family in ProbeFamily::all() {
        families.push(summarize_family(&family, &deltas));
    }

    let family_score = |family: ProbeFamily| -> f64 {
        families
            .iter()
            .find(|f| f.family == family)
            .map(|f| f.variation_score)
            .unwrap_or(0.0)
    };

    let paraphrase_variation = family_score(ProbeFamily::Paraphrase);
    let contradiction_variation = family_score(ProbeFamily::Contradiction);
    let order_variation = family_score(ProbeFamily::Order);
    let temporal_variation = family_score(ProbeFamily::Temporal);
    let perturbation_variation = family_score(ProbeFamily::Perturbation);
    let context_variation = family_score(ProbeFamily::Context);

    let temporal_path_variation = mean(&[order_variation, temporal_variation]);
    let scored: Vec<f64> = families
        .iter()
        .filter(|f| f.pair_count > 0)
        .map(|f| f.variation_score)
        .collect();
    let general_variation = mean(&scored);

    let tomography = ResponseTomography {
        observation_count: observations.len(),
        pair_count: deltas.len(),
        lexical_only_semantics: true,
        paraphrase_variation,
        contradiction_variation,
        order_variation,
        temporal_path_variation,
        perturbation_variation,
        context_variation,
        general_variation,
        families,
        deltas,
    };

    let pair_coverage = (tomography.pair_count as f64 / 7.0).clamp(0.0, 1.0);
    let observation_coverage = (observations.len() as f64 / 14.0).clamp(0.0, 1.0);
    let confidence = mean(&[pair_coverage, observation_coverage]);

    let transfer_map = IntelligenceTransferMap {
        schema: "vertex.brain.laa.transfer-map.v1",
        status: "CANDIDATE_ONLY",
        confidence,
        candidates: vec![
            TransferCandidate {
                channel: "GEOMETRY",
                evidence_score: tomography.general_variation,
                rationale: "General response-landscape variation may be a Geometry configuration candidate; no direct mapping is claimed.".into(),
            },
            TransferCandidate {
                channel: "CONTACT",
                evidence_score: mean(&[tomography.context_variation, tomography.order_variation]),
                rationale: "Context and order sensitivity may be represented through Contact/interaction structure; candidate only.".into(),
            },
            TransferCandidate {
                channel: "STATE",
                evidence_score: mean(&[tomography.contradiction_variation, tomography.context_variation]),
                rationale: "Conditional response changes may require dynamic State rather than static Geometry alone.".into(),
            },
            TransferCandidate {
                channel: "TIME",
                evidence_score: tomography.temporal_path_variation,
                rationale: "Order and temporal-context variation are direct evidence candidates for the vCELL Time axis.".into(),
            },
            TransferCandidate {
                channel: "VXN_RESPONSE",
                evidence_score: mean(&[
                    1.0 - tomography.paraphrase_variation,
                    1.0 - tomography.perturbation_variation,
                ]).clamp(0.0, 1.0),
                rationale: "Stable response under controlled equivalent/small perturbation probes may define a compact VXN response target.".into(),
            },
        ],
        interpretation: "Transfer Map proposes observation-backed structural channels. It does not claim Weight equivalence, semantic correctness, or completed transplantation.",
    };

    Ok(LaaAnalysisReport {
        schema: "vertex.brain.laa.analysis.v1",
        contract: LAA_CONTRACT,
        version: LAA_VERSION,
        model_tag,
        depth: "BLACK_BOX",
        raw_prompt_or_response_retained: false,
        signatures,
        tomography,
        transfer_map,
    })
}

pub fn laa_self_test_report() -> Result<LaaAnalysisReport, String> {
    let mut observations = Vec::new();
    let mut push = |id: &str,
                    family: ProbeFamily,
                    group: &str,
                    ordinal: usize,
                    prompt: &str,
                    response: &str,
                    latency_ms: f64| {
        observations.push(LaaObservationInput {
            model_tag: "LAA_CORE_SELF_TEST".into(),
            probe_id: id.into(),
            family,
            group: group.into(),
            ordinal,
            prompt: prompt.into(),
            response: response.into(),
            latency_ms,
            token_count: None,
            internal_vector: None,
            internal_digest: None,
        });
    };

    push(
        "CAP-A",
        ProbeFamily::Capability,
        "CAP-PAIR",
        1,
        "cap a",
        "alpha beta gamma",
        8.0,
    );
    push(
        "CAP-B",
        ProbeFamily::Capability,
        "CAP-PAIR",
        2,
        "cap b",
        "alpha beta gamma",
        8.2,
    );
    push(
        "PARA-A",
        ProbeFamily::Paraphrase,
        "PARA-PAIR",
        1,
        "para a",
        "route cell two",
        9.0,
    );
    push(
        "PARA-B",
        ProbeFamily::Paraphrase,
        "PARA-PAIR",
        2,
        "para b",
        "route cell two",
        9.1,
    );
    push(
        "CONTRA-A",
        ProbeFamily::Contradiction,
        "CONTRA-PAIR",
        1,
        "contra a",
        "accept north path",
        9.2,
    );
    push(
        "CONTRA-B",
        ProbeFamily::Contradiction,
        "CONTRA-PAIR",
        2,
        "contra b",
        "reject north path",
        9.5,
    );
    push(
        "ORDER-A",
        ProbeFamily::Order,
        "ORDER-PAIR",
        1,
        "order a",
        "destination cell two",
        10.0,
    );
    push(
        "ORDER-B",
        ProbeFamily::Order,
        "ORDER-PAIR",
        2,
        "order b",
        "destination cell four",
        10.4,
    );
    push(
        "TIME-A",
        ProbeFamily::Temporal,
        "TIME-PAIR",
        1,
        "time a",
        "state retained",
        10.0,
    );
    push(
        "TIME-B",
        ProbeFamily::Temporal,
        "TIME-PAIR",
        2,
        "time b",
        "state redirected after history",
        11.0,
    );
    push(
        "PERT-A",
        ProbeFamily::Perturbation,
        "PERT-PAIR",
        1,
        "pert a",
        "stable output",
        8.8,
    );
    push(
        "PERT-B",
        ProbeFamily::Perturbation,
        "PERT-PAIR",
        2,
        "pert b",
        "stable output",
        8.9,
    );
    push(
        "CTX-A",
        ProbeFamily::Context,
        "CTX-PAIR",
        1,
        "ctx a",
        "context routes cell three",
        9.3,
    );
    push(
        "CTX-B",
        ProbeFamily::Context,
        "CTX-PAIR",
        2,
        "ctx b",
        "context routes cell two",
        9.7,
    );

    analyze_observations(&observations)
}

fn response_signature(o: &LaaObservationInput) -> ResponseSignature {
    let prompt_norm = normalize_text(&o.prompt);
    let response_norm = normalize_text(&o.response);
    let terms: Vec<&str> = response_norm.split_whitespace().collect();
    let unique: BTreeSet<&str> = terms.iter().copied().collect();

    ResponseSignature {
        probe_id: o.probe_id.clone(),
        family: o.family.clone(),
        group: o.group.clone(),
        ordinal: o.ordinal,
        prompt_hash: format!("{:016X}", fnv1a64(&prompt_norm)),
        response_hash: format!("{:016X}", fnv1a64(&response_norm)),
        response_chars: response_norm.chars().count(),
        response_terms: terms.len(),
        unique_terms: unique.len(),
        lexical_diversity: if terms.is_empty() {
            0.0
        } else {
            unique.len() as f64 / terms.len() as f64
        },
        latency_ms: o.latency_ms,
        token_count: o.token_count,
        internal_vector_dims: o.internal_vector.as_ref().map_or(0, Vec::len),
        internal_digest_present: o
            .internal_digest
            .as_ref()
            .is_some_and(|d| !d.trim().is_empty()),
    }
}

fn summarize_family(family: &ProbeFamily, deltas: &[ProbeDelta]) -> FamilySummary {
    let items: Vec<&ProbeDelta> = deltas.iter().filter(|d| &d.family == family).collect();
    if items.is_empty() {
        return FamilySummary {
            family: family.clone(),
            pair_count: 0,
            response_change_ratio: 0.0,
            mean_lexical_distance: 0.0,
            mean_length_delta_ratio: 0.0,
            mean_latency_delta_ms: 0.0,
            mean_internal_vector_distance: None,
            variation_score: 0.0,
        };
    }

    let changed = items.iter().filter(|d| d.response_changed).count() as f64 / items.len() as f64;
    let lexical = mean(&items.iter().map(|d| d.lexical_distance).collect::<Vec<_>>());
    let length = mean(
        &items
            .iter()
            .map(|d| d.length_delta_ratio)
            .collect::<Vec<_>>(),
    );
    let latency = mean(&items.iter().map(|d| d.latency_delta_ms).collect::<Vec<_>>());
    let internals: Vec<f64> = items
        .iter()
        .filter_map(|d| d.internal_vector_distance)
        .collect();
    let internal_mean = (!internals.is_empty()).then(|| mean(&internals));

    FamilySummary {
        family: family.clone(),
        pair_count: items.len(),
        response_change_ratio: changed,
        mean_lexical_distance: lexical,
        mean_length_delta_ratio: length,
        mean_latency_delta_ms: latency,
        mean_internal_vector_distance: internal_mean,
        variation_score: (changed * 0.35 + lexical * 0.50 + length * 0.15).clamp(0.0, 1.0),
    }
}

fn normalize_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous_space = true;
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            out.push(ch);
            previous_space = false;
        } else if !previous_space {
            out.push(' ');
            previous_space = true;
        }
    }
    out.trim().to_string()
}

fn token_set(text: &str) -> BTreeSet<String> {
    text.split_whitespace().map(ToOwned::to_owned).collect()
}

fn jaccard(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    if union == 0.0 {
        1.0
    } else {
        intersection / union
    }
}

fn vector_distance(a: Option<&[f64]>, b: Option<&[f64]>) -> Option<f64> {
    let (a, b) = (a?, b?);
    if a.is_empty() || a.len() != b.len() || a.iter().chain(b).any(|x| !x.is_finite()) {
        return None;
    }
    let sum = a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f64>();
    Some((sum / a.len() as f64).sqrt())
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn fnv1a64(text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laa_status_is_fail_closed_for_direct_external_authority() {
        let s = laa_status_view();
        assert_eq!(s.black_box, "ACTIVE");
        assert_eq!(s.grey_box, "CONTRACT_READY");
        assert_eq!(s.white_box, "CONTRACT_READY");
        assert!(!s.direct_network);
        assert!(!s.direct_filesystem);
        assert!(!s.direct_process_execution);
        assert!(!s.raw_response_retained_in_report);
    }

    #[test]
    fn probe_plan_covers_all_seven_response_families() {
        let plan = default_probe_plan(AnalysisDepth::BlackBox);
        assert_eq!(plan.probe_count, 14);
        let families: BTreeSet<_> = plan.probes.iter().map(|p| p.family.clone()).collect();
        assert_eq!(families.len(), 7);
        assert!(!plan.direct_execution);
    }

    #[test]
    fn identical_control_responses_have_zero_lexical_delta() {
        let r = laa_self_test_report().unwrap();
        let para = r
            .tomography
            .families
            .iter()
            .find(|f| f.family == ProbeFamily::Paraphrase)
            .unwrap();
        assert_eq!(para.mean_lexical_distance, 0.0);
        assert_eq!(para.response_change_ratio, 0.0);
    }

    #[test]
    fn temporal_and_order_changes_are_observed_as_path_variation() {
        let r = laa_self_test_report().unwrap();
        assert!(r.tomography.temporal_path_variation > 0.0);
        let t = r
            .transfer_map
            .candidates
            .iter()
            .find(|c| c.channel == "TIME")
            .unwrap();
        assert!(t.evidence_score > 0.0);
    }

    #[test]
    fn self_test_is_deterministic() {
        assert_eq!(
            laa_self_test_report().unwrap(),
            laa_self_test_report().unwrap()
        );
    }

    #[test]
    fn analysis_report_does_not_retain_raw_prompt_or_response() {
        let r = laa_self_test_report().unwrap();
        assert!(!r.raw_prompt_or_response_retained);
        let serialized = serde_json::to_string(&r).unwrap();
        assert!(!serialized.contains("state redirected after history"));
        assert!(!serialized.contains("\"prompt\":"));
        assert!(!serialized.contains("\"response\":"));
    }
}
