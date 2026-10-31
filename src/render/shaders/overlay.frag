#version 450
// The game's 2D layer over the 3D: every pixel the game has not written still holds the mark
// (`gpu::UNWRITTEN`), and shows what is under it.
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D canvas;
void main() {
    vec4 pixel = texelFetch(canvas, ivec2(gl_FragCoord.xy), 0);
    uvec4 bytes = uvec4(pixel * 255.0 + 0.5);
    if (bytes == uvec4(0xFF, 0x00, 0xFF, 0xA5)) discard;
    result = vec4(pixel.rgb, 1.0);
}
