#version 450
// A menu's frame with its 3D drawn at a larger size than the frame (`gpu::MenuHigh`): where the
// frame's picture is still what it was when the 3D was drawn (`kept`), the larger 3D picture is
// taken; where the game drew over it since (the text and the buttons), the frame's own picture,
// enlarged with the linear filter.
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D low;
layout(set = 2, binding = 1) uniform sampler2D kept;
layout(set = 2, binding = 2) uniform sampler2D high;
void main() {
    vec2 at = gl_FragCoord.xy / vec2(textureSize(high, 0));
    ivec2 source = min(ivec2(at * vec2(textureSize(low, 0))), textureSize(low, 0) - 1);
    if (texelFetch(low, source, 0) == texelFetch(kept, source, 0)) {
        result = vec4(texelFetch(high, ivec2(gl_FragCoord.xy), 0).rgb, 1.0);
    } else {
        result = vec4(texture(low, at).rgb, 1.0);
    }
}
