# Vertex Brain System 0.4 — Pre-Transfer Calibration

```text
             LM Studio / Existing LLM
                    |
                 STANDBY
                    X
               NO INGESTION
                    X
                    |
                  LAA
             OBSERVE_ONLY
                    |
             CANDIDATE_GATED
                    |
        +-----------+-----------+
        |     BRAIN SYSTEM      |
        |                       |
        | Resident Baseline     |
        | 000001 / 4C           |
        |         |             |
        |         v             |
        |   +=============+     |
        |   ||   vCELL   ||     |
        |   || ENTHRONED ||     |
        |   ||   4C      ||     |
        |   +=============+     |
        |         |             |
        |      vSCOPE           |
        |      Evidence         |
        +-----------------------+
```

## Why baseline comes first

A transfer experiment has no meaning without a known receiver state.

Before observing or ingesting a teacher model, Brain System must know whether the
resident substrate itself is reproducible and how it reacts to controlled changes.

Resident Response Baseline 000001 therefore captures five response axes:

1. Replay / deterministic control
2. Geometry response
3. Propagation response
4. Temporal response
5. Persistence / recovery response

## Fingerprint

`VRB1-*` is a deterministic hash over stable structural/dynamic observations.

It deliberately does not include wall-clock benchmark values.

The fingerprint can later answer:

- Did the substrate change?
- Did scaling preserve or alter its response regime?
- Did a transfer experiment leave a persistent response change?
- Did a cue return the system toward an earlier response signature?

Those later questions are not answered by 000001 itself.

## Current gate

`LLM_INGESTION = LOCKED_UNTIL_POST_BASELINE_GATE`

Next:

`RESIDENT_SCALE_AND_RESPONSE_SEPARABILITY`


## Additive Typography Contract

```text
component default font size + FONT DELTA = rendered semantic text size
```

The preference layer changes typography only; it does not apply browser zoom and
does not scale panel, grid, icon, vCELL, or observation geometry.

Default is `+2pt`. Relative typography hierarchy remains defined by the original
component defaults.


## Response Separability Layer

```text
4C Resident Baseline
        |
        v
standardized temporal sweep
        |
        v
observed response trajectories
        |
        v
Response Signature VRS1
        |
        v
complete pair-distance matrix
        |
        +--> separated pairs
        +--> closest pair
        +--> response clusters
        +--> destination diversity
        |
        v
4C Response Landscape Evidence
```

The analysis does not insert the stimulus onset into the response fingerprint.

This is a deliberate observability constraint: response identity must come from
what the resident substrate does, not from the label attached to the stimulus.

Contact remains outside the independent perturbation set for 000001.


## 32C Scale Candidate

```text
4C VERIFIED RESIDENT
        |
   BrainSystem
        |
        +---- observe ----> 32C REGION CANDIDATE
                             768 vertices
                             real contacts
                             same 3-byte VXN
                                   |
                           7 geometry cases
                                   |
                           21 response pairs
                                   |
                           SCALE EVIDENCE
                                   |
                                   X
                           no auto promotion
```

No false numeric equivalence is asserted across current carrier generations.


## 32C Resident Promotion

32C is the persistent resident substrate. 4C is retained as a regression/history
reference. Promotion does not delete prior Evidence and does not unlock LLM ingestion.


## Persistent Difference Observatory

Written A/B lanes and a zero-write control are observed at cumulative passive
settle horizons 256, 512, 1024, 2048 and 4096. Plateau is a candidate only.
