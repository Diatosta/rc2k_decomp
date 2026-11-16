#version 450
// What `rasteriser::Triangle` does to a pixel, before the blending.
layout(location = 0) noperspective in vec4 colour;
layout(location = 1) noperspective in float fog;
layout(location = 2) in vec2 uv;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D picture;
layout(set = 3, binding = 0) uniform Pixel {
    vec4 fog_colour;
    float alpha_reference;
    int alpha_function; // Direct3D's comparison; 0: no alpha test
    int mixing;         // state 0x15
    int flags;          // 1 textured, 2 the texture has alpha, 4 fog; the diagnostic cuts from bit 8
};
// `gpu_cut`'s shader cuts, one flag bit each (all clear: the shader as it always was).
bool cut(int n) { return (flags & (0x100 << n)) != 0; }
bool passes(float value) {
    switch (alpha_function) {
    case 1: return false;
    case 2: return value < alpha_reference;
    case 3: return value == alpha_reference;
    case 4: return value <= alpha_reference;
    case 5: return value > alpha_reference;
    case 6: return value != alpha_reference;
    case 7: return value >= alpha_reference;
    default: return true;
    }
}
void main() {
    vec4 out_colour = colour;
    if (cut(5)) { // flat
        result = out_colour;
        return;
    }
    if ((flags & 1) != 0) {
        vec4 texel = cut(2) ? vec4(0.5) : texture(picture, cut(3) ? vec2(0.5) : uv); // tex, texhot
        if (mixing == 1 || mixing == 7) out_colour = texel;
        else if (mixing == 4) out_colour = texel * colour;
        else out_colour = vec4(texel.rgb * colour.rgb, (flags & 2) != 0 ? texel.a : colour.a);
    }
    if (alpha_function != 0 && !cut(1) && !passes(out_colour.a)) discard;
    if ((flags & 4) != 0 && !cut(0)) { // fog
        float kept = clamp(fog, 0.0, 1.0);
        out_colour.rgb = out_colour.rgb * kept + fog_colour.rgb * (1.0 - kept);
    }
    result = out_colour;
}
