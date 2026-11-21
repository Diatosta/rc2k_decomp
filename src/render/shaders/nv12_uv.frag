#version 450
// The video's chroma plane: the mean of each 2 x 2 block of the frame's colour as Cb and Cr of
// BT.601 in studio range (16..240), two bytes a pixel (blue difference first).
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D frame;
void main() {
    ivec2 corner = ivec2(gl_FragCoord.xy) * 2;
    vec3 rgb = texelFetch(frame, corner, 0).rgb + texelFetch(frame, corner + ivec2(1, 0), 0).rgb
        + texelFetch(frame, corner + ivec2(0, 1), 0).rgb
        + texelFetch(frame, corner + ivec2(1, 1), 0).rgb;
    rgb *= 0.25;
    float cb = (128.0 - 37.797 * rgb.r - 74.203 * rgb.g + 112.0 * rgb.b) / 255.0;
    float cr = (128.0 + 112.0 * rgb.r - 93.786 * rgb.g - 18.214 * rgb.b) / 255.0;
    result = vec4(cb, cr, 0.0, 1.0);
}
