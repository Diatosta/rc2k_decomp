#!/usr/bin/env bash
# The renderer's shaders as SPIR-V, and from it as Metal's source for macOS (MSL, spirv-cross;
# its entry point is `main0`). Both committed, so that a build needs no shader compiler.
# MSL resources are numbered by their SPIR-V binding (`--msl-decoration-binding`), which is
# SDL's order on Metal: each stage's sampled textures and samplers, then uniform buffers, from
# [[texture(0)]], [[sampler(0)]], [[buffer(0)]]; the vertex buffer at [[buffer(14)]] (SDL's).
set -euo pipefail
cd "$(dirname "$0")/../src/render/shaders"
for s in scene.vert scene.frag overlay.vert overlay.frag quad.vert quad.frag; do
    glslc -O --target-env=vulkan1.0 "$s" -o "$s.spv"
    spirv-cross "$s.spv" --msl --msl-version 20100 --msl-decoration-binding --output "$s.msl"
done
