# Vertex Brain System 0.9.1

## Resident Response Baseline 000001

vCELL is already mounted as the Brain System resident cognitive substrate.
0.4.0 does **not** begin LLM ingestion.

This release calibrates the existing 4C resident core before any model-transfer work.

Baseline contract:

`vertex.brain.vcell-resident-baseline.v1`

Experiment:

`VCELL_RESIDENT_RESPONSE_BASELINE_000001`

The standardized baseline records:

- exact same-input replay control
- Geometry response equivalence and delta egress
- self-reconfiguration / recovery observations
- first temporal delta and per-cell arrival order
- temporal locality and peak influence
- time-gated response span
- dominant-destination variation
- deterministic timing controls
- a stable deterministic baseline fingerprint

The fingerprint intentionally excludes benchmark timing/performance values so replay
does not become host-speed dependent.

## LLM / LAA gate

For this release:

- `LAA = OBSERVE_ONLY`
- `LLM_INGESTION = LOCKED_UNTIL_POST_BASELINE_GATE`
- `PROVIDER = STANDBY_ONLY`
- `TRANSFER = CANDIDATE_GATED`

LM Studio or another provider may be running outside Brain System, but 0.4.0 does
not connect to it and does not consume model responses.

Next gate:

`RESIDENT_SCALE_AND_RESPONSE_SEPARABILITY`

The purpose is to know the resident substrate before exposing it to a teacher model.

## Interpretation

Resident Response Baseline 000001 is a calibration signature.

It is **not**:

- an intelligence score
- an LLM transfer map
- proof of memory capacity
- permission to ingest a model

It is the pre-transfer reference against which later vCELL growth and transfer
experiments can be compared.


## Global Font Delta

Brain System 0.4.1 adds a persistent whole-UI font adjustment.

`Rendered font size = each component's original/default font size + FONT DELTA`

This is not browser zoom and does not scale panel geometry.

- default: `+2pt`
- range: `-2pt .. +6pt`
- step: `1pt`
- `−` / `+` changes the common delta
- center click restores `+2pt`
- center double-click restores original `0pt`
- local preference persists across restarts
- canvas CELL labels follow the same delta
- decorative glyph icons stay fixed to prevent geometry drift

Brain Membrane, vCELL Resident Core, Baseline 000001, LAA, and LLM ingestion
gates are unchanged.


## 4C Response Separability 000001

Experiment:

`VCELL_4C_RESPONSE_SEPARABILITY_000001`

Contract:

`vertex.brain.vcell-response-separability.v1`

This experiment asks whether the verified 4C resident core produces distinguishable
response identities under controlled temporal conditions while the same 3-byte VXN
seed and resident substrate are retained.

### Response signature

A response signature is built only from observed outputs:

- first trace step
- trace at return
- peak trace magnitude
- peak trace step
- final trace magnitude
- final max state delta
- final destination-share distance
- dominant destination

The stimulus onset is recorded as a case label but deliberately excluded from the
response fingerprint. This prevents trivial "different input label = different
response" classification.

### Pairwise separability

All timing-sweep responses are normalized into response space and compared
pairwise.

The default threshold is `0.08`.

That threshold is an **analysis threshold**, not a physical law and not an
intelligence score.

The report returns:

- response signatures / fingerprints
- complete pair matrix
- separated-pair ratio
- minimum / maximum / mean pair distance
- closest pair
- cluster count
- dominant-destination diversity

Classification is:

- `FULLY_SEPARATED`
- `PARTIALLY_SEPARATED`
- `COLLAPSED`

### Axis discipline

- TIME: measured with the full response matrix
- GEOMETRY: independent egress-response delta is reused from verified delta evidence
- HISTORY: reported only as `PATH_DEPENDENCE_OBSERVED_NOT_MEMORY`
- CONTACT: `NOT_ISOLATED_IN_000001`

Contact is intentionally not claimed because 000001 does not independently perturb
the contact axis.

### External intelligence remains locked

- LLM ingestion: `LOCKED`
- provider: `STANDBY_ONLY`
- LAA: `OBSERVE_ONLY`
- transfer: `CANDIDATE_GATED`

Next gate after separability evidence:

`32C_SCALE_EXPANSION`


## 32C Scale Expansion 000001

Brain System observes the existing `REGION_FLOW_000009` real 32C region as a scale
candidate while the verified 4C core remains resident.

- x8 CELL count: 4C -> 32C
- 768 real vertices
- real Face/Point inter-CELL contacts
- same 3-byte / 24-bit VXN
- seven Geometry gate conditions
- complete 21-pair response comparison

Response identity comes from normalized 32C CELL share, shell peaks, and arrival
profile. Gate tilt is deliberately excluded from the fingerprint.

Current 4C and REGION FLOW 32C kernels are not forced into direct numerical
equivalence because REGION FLOW 000009 still marks FIRST_WAVE carrier provisional.

`cross_scale_numeric_equivalence_claimed = false`

32C remains `CANDIDATE_ONLY`.
LLM ingestion remains `LOCKED`.
Next gate: `32C_RESIDENT_PROMOTION`.


## 32C Resident Promotion 000001

32C is now the persistent BrainSystem resident runtime.

- Resident ID: REGION 000032
- 32 CELL / 768 vertices
- same 3-byte VXN
- persistent RegionWave state across probe calls
- STATUS / PULSE / RESET controls
- 4C retained as 4C_REGRESSION_REFERENCE
- LLM ingestion remains LOCKED

Next gate: 32C_RESIDENT_BASELINE


## 32C Resident Baseline 000001
RESET -> WRITE(128) -> SETTLE(512) -> PASSIVE READ -> SETTLE(512) -> PASSIVE READ. Replayed twice on isolated same-topology calibration lanes. Live Resident is not mutated. Persistent post-settle difference is not yet called memory. Next gate: 32C_PERSISTENT_DIFFERENCE.


## 32C Persistent Difference 000001

Long-horizon observation:

`WRITE -> 256 -> 512 -> 1024 -> 2048 -> 4096`

A zero-write control lane runs the same settle horizons.

Possible classifications:
CONTROL_CONTAMINATED / RELAXED_TO_BASELINE / DECAYING_RESIDUAL /
PERSISTENT_PLATEAU_CANDIDATE / NON_MONOTONIC_PERSISTENT_RESPONSE.

Memory is not claimed.


## 32C Persistent Difference Result Egress 000001

Verification now runs one additional narrow `cargo test ... -- --nocapture`
command that prints the actual long-horizon experiment result into VERTEX WORKS
Evidence.

Printed result includes:

- state / classification
- H256 / H512 / H1024 / H2048 / H4096 energy, amplitude, active cells and state fingerprint
- final energy and amplitude ratios
- tail relative deltas
- zero-write control result
- deterministic replay result
- persistent-difference flag
- plateau candidate flag
- non-monotonic flag
- memory claim
- LLM ingestion state
- next gate
- experiment fingerprint

This patch does not change the experiment algorithm or classification rules.
