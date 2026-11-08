#!/usr/bin/env bash
# The renderer's shaders as SPIR-V (committed, so that a build needs no shader compiler).
set -euo pipefail
cd "$(dirname "$0")/../src/render/shaders"
for s in scene.vert scene.frag overlay.vert overlay.frag; do
    glslc -O --target-env=vulkan1.0 "$s" -o "$s.spv"
done
