# VERTEX BRAIN SYSTEM PUBLIC SITE 000001

## Purpose

Deploy a motivational public page for Vertex Brain System to the existing Vertex Hub public root using the same pattern established for Vertex Works.

## Public route

- Host: `vertex.a-portal.net`
- Route: `/vertex-brain-system/`
- Physical deployment folder: `VertexHub/site/vertex-brain-system`

## Source folder

Canonical site source is created under:

`G:\Vertex_Project\Development\vertex_brain_system\site\vertex-brain-system-public`

## Visual direction

- Dark cyber blue theme
- Central glowing neural core
- Resident / topology / observation sections
- Read-only observatory tone based on Brain System UI direction
- Access counter included

## Deployment strategy

This builder does **not** rediscover IIS.
It reuses the existing Vertex Hub public root from:

1. `VertexHub/deployment.json` → `physicalPath`
2. fallback: `VertexHub/site`

Then deploys `vertex-brain-system` beneath that root.

## Closure

- backup existing route
- preserve counter runtime data
- deploy source
- verify source and deployed copy
- best-effort ACL for `IIS_IUSRS`
- public HTTP check
- commit/push Brain System repo paths
- commit/push `vertex_studio_ai` Hub route
