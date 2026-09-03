# VERTEX BRAIN SYSTEM PUBLIC SITE PATH GUARD HOTFIX 000003

## Failure observed in 000001

The Hub root correctly resolved to:

`G:\Vertex_Project\Development\vertex_studio_ai\VertexHub\site`

but the Brain System builder rejected that path as being outside `vertex_studio_ai`.

The cause was a separator bug in the Brain System copy of `Test-PathUnder`: it appended a double backslash sequence in the PowerShell string comparison, so the valid child path did not match the parent prefix.

## Fix

000003 replaces the Brain System builder's path containment routine with the exact already-proven implementation used by Vertex Works 000041.

Vertex Works 000041 successfully deployed through the same canonical fallback path:

`G:\Vertex_Project\Development\vertex_studio_ai\VertexHub\site`

## Site content

000003 is self-contained and also carries the bilingual academic site source:

- Japanese default
- English switching
- readable typography
- formal observability language
- state vector `x_t ∈ Ω_R`
- boundary operator `∂M`
- observation map `O_v : Ω_R → E`
- working hypotheses H1-H4
- visitor counter

## Closure

VRA apply
→ Python runner
→ PowerShell builder
→ existing Vertex Hub public root
→ `/vertex-brain-system`
→ backup
→ preserve counter data
→ verify source and deployment
→ HTTP check
→ commit/push `vertex_brain_system`
→ commit/push Hub route in `vertex_studio_ai`
