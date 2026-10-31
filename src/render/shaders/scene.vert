#version 450
// A Direct3D 5 transformed vertex (x, y, z, 1/w on the screen) as the clip space of a GPU.
// Pixels are sampled at their integer places in Direct3D 5, at their middles here.
layout(location = 0) in vec4 place;
layout(location = 1) in vec4 colour;
layout(location = 2) in vec4 specular;
layout(location = 3) in vec2 uv;
layout(set = 1, binding = 0) uniform Screen { vec2 size; };
layout(location = 0) noperspective out vec4 out_colour;
layout(location = 1) noperspective out float out_fog;
layout(location = 2) out vec2 out_uv;
void main() {
    float w = place.w > 0.0 ? 1.0 / place.w : 1.0;
    gl_Position = vec4(((place.x + 0.5) / size.x * 2.0 - 1.0) * w,
                       (1.0 - (place.y + 0.5) / size.y * 2.0) * w, place.z * w, w);
    out_colour = colour.bgra; // the bytes are blue, green, red, alpha
    out_fog = specular.a;
    out_uv = uv;
}
