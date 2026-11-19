#version 450
// The frame stretched to a smaller picture, each pixel the frame's pixel at its place scaled
// down in whole numbers (`x * frame width / picture width`: what the CPU's copy of the screen
// into a texture takes), so that the two pictures are the same pixels.
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 result;
layout(set = 2, binding = 0) uniform sampler2D frame;
layout(set = 3, binding = 0) uniform Picture {
    ivec4 size; // the picture's width and height
};
void main() {
    ivec2 at = ivec2(gl_FragCoord.xy);
    ivec2 from = textureSize(frame, 0);
    result = texelFetch(frame, ivec2(at.x * from.x / size.x, at.y * from.y / size.y), 0);
}
