#version 450
// The video's luma plane: the frame's colour as Y' of BT.601 in studio range (16..235), one
// byte a pixel, the same numbers a software conversion of the frame makes.
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D frame;
void main() {
    vec3 rgb = texelFetch(frame, ivec2(gl_FragCoord.xy), 0).rgb;
    float y = (16.0 + 65.481 * rgb.r + 128.553 * rgb.g + 24.966 * rgb.b) / 255.0;
    result = vec4(y, 0.0, 0.0, 1.0);
}
