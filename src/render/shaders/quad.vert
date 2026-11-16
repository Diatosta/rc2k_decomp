#version 450
// A rectangle of the 2D atlas at whole pixels of the screen: the corner's place on the screen in
// pixels and its texel in the atlas, and the first texel of its palette (x below 0: none).
layout(location = 0) in vec4 corner;
layout(location = 1) in vec2 palette;
layout(set = 1, binding = 0) uniform Screen { vec2 size; };
layout(location = 0) out vec2 out_texel;
layout(location = 1) flat out vec2 out_palette;
void main() {
    gl_Position = vec4(corner.x / size.x * 2.0 - 1.0, 1.0 - corner.y / size.y * 2.0, 0.0, 1.0);
    out_texel = corner.zw;
    out_palette = palette;
}
