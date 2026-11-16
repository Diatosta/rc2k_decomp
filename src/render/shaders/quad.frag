#version 450
// The 2D atlas' texel of the rectangle's pixel: a gap (alpha 0) draws nothing; a palette glyph's
// texel holds an index in red, looked up in the palette's row of the same atlas; a direct
// colour is the texel; a solid rectangle has x of at most -2 in its palette, which holds its colour:
// minus (2 + red * 256 + green) in x and blue in y. Opaque, as the canvas' pixels are put over the 3D.
layout(location = 0) in vec2 texel;
layout(location = 1) flat in vec2 palette;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D atlas;
void main() {
    if (palette.x <= -2.0) {
        int packed = int(-palette.x) - 2;
        result = vec4(float(packed >> 8) / 255.0, float(packed & 255) / 255.0, palette.y / 255.0, 1.0);
        return;
    }
    vec4 value = texelFetch(atlas, ivec2(floor(texel)), 0);
    if (value.a < 0.5) discard;
    if (palette.x >= 0.0) {
        int index = int(value.r * 255.0 + 0.5);
        value = texelFetch(atlas, ivec2(int(palette.x) + index, int(palette.y)), 0);
    }
    result = vec4(value.rgb, 1.0);
}
