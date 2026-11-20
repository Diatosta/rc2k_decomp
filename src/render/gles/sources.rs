//! The GLSL ES sources of the OpenGL backend's programs.

use std::ffi::CStr;

// The game's screen in clip space: as `scene.vert`. There is no `noperspective` in ES 3.0: a
// value times W, interpolated with W and divided by it, comes out straight across the screen.
pub(super) const SCENE_VS: &CStr = c"#version 300 es
layout(location=0) in vec4 place; layout(location=1) in vec4 colour;
layout(location=2) in vec4 specular; layout(location=3) in vec2 uv;
layout(location=4) in vec4 colour2; layout(location=5) in vec4 specular2;
layout(location=6) in vec2 uv2; layout(location=7) in vec4 layers;
uniform vec2 size;
out highp vec4 v_colour; out highp float v_w;
#ifdef FIRST_TEXTURE
out highp vec2 v_uv; flat out lowp int v_unit; flat out highp float v_layer;
#endif
#ifdef FIRST_FOG
out highp float v_fog;
#endif
#ifdef SECOND
out highp vec4 v_colour2;
#ifdef SECOND_TEXTURE
out highp vec2 v_uv2; flat out lowp int v_unit2; flat out highp float v_layer2;
#endif
#ifdef SECOND_FOG
out highp float v_fog2;
#endif
#endif
void main() {
    float w = place.w > 0.0 ? 1.0 / place.w : 1.0;
    gl_Position = vec4(((place.x + 0.5) / size.x * 2.0 - 1.0) * w,
                       (1.0 - (place.y + 0.5) / size.y * 2.0) * w,
                       (clamp(place.z, 0.0, 1.0) * 2.0 - 1.0) * w, w);
    v_colour = colour.bgra * w; v_w = w;
#ifdef FIRST_TEXTURE
    v_uv = uv; v_unit = int(layers.x); v_layer = layers.y;
#endif
#ifdef FIRST_FOG
    v_fog = specular.a * w;
#endif
#ifdef SECOND
    v_colour2 = colour2.bgra * w;
#ifdef SECOND_TEXTURE
    v_uv2 = uv2; v_unit2 = int(layers.z); v_layer2 = layers.w;
#endif
#ifdef SECOND_FOG
    v_fog2 = specular2.a * w;
#endif
#endif
}
";

/// A batch's pass; with `SECOND` in `flags`, a second pass blended over it by its alpha as the
/// target's blend would (`batches::Second`), the first's colour first held in bytes as the
/// target holds it.
///
/// The `#define`s put before it choose what it is made for (`ShaderKind::defines`,
/// `ShaderKind::defines_for_any`): with `MADE_FOR_KIND` each pass's function, mixing and flags
/// are constants, so nothing of them is decided per pixel, and the passes and vertex values the
/// kind does not have (`FIRST_TEXTURE`, `FIRST_FOG`, `SECOND`, `SECOND_TEXTURE`, `SECOND_FOG`)
/// are left out of both shaders. The arithmetic of what is left is the same text either way.
pub(super) const SCENE_FS: &CStr = c"#version 300 es
precision mediump float;
in highp vec4 v_colour; in highp float v_w;
#ifdef FIRST_TEXTURE
in highp vec2 v_uv; flat in lowp int v_unit; flat in highp float v_layer;
#endif
#ifdef FIRST_FOG
in highp float v_fog;
#endif
#ifdef SECOND
in highp vec4 v_colour2;
#ifdef SECOND_TEXTURE
in highp vec2 v_uv2; flat in lowp int v_unit2; flat in highp float v_layer2;
#endif
#ifdef SECOND_FOG
in highp float v_fog2;
#endif
#endif
out vec4 result;
// The textures are layers of arrays (`TextureArray`), two bound for each pass: the vertex
// says which and which layer. A sampler without smaller levels reads the one level, so the
// level of detail `texture` works out changes nothing for it; the mipmapping sampler reads the
// smaller ones.
precision lowp sampler2DArray;
uniform sampler2DArray picture_a; uniform sampler2DArray picture_b;
uniform vec4 fog_colour; uniform float alpha_reference;
#ifdef MADE_FOR_KIND
const int alpha_function = ALPHA_FUNCTION; const int mixing = MIXING; const int flags = FLAGS;
#else
uniform int alpha_function; uniform int mixing; uniform int flags;
#endif
#ifdef SECOND
uniform sampler2DArray picture2_a; uniform sampler2DArray picture2_b;
uniform vec4 fog_colour2; uniform float alpha_reference2;
#ifdef MADE_FOR_KIND
const int alpha_function2 = ALPHA_FUNCTION2; const int mixing2 = MIXING2; const int flags2 = FLAGS2;
#else
uniform int alpha_function2; uniform int mixing2; uniform int flags2;
#endif
#endif
vec4 fetch(sampler2DArray a, sampler2DArray b, int unit, highp vec3 at) {
#ifdef CUT_TEX
    return vec4(0.5);
#endif
#ifdef CUT_TEXHOT
    at = vec3(0.5, 0.5, at.z);
#endif
    return unit == 0 ? texture(a, at) : texture(b, at);
}
bool passes(int function, float reference, float value) {
#ifdef CUT_ALPHA
    return true;
#endif
    if (function == 1) return false;
    if (function == 2) return value < reference;
    if (function == 3) return value == reference;
    if (function == 4) return value <= reference;
    if (function == 5) return value > reference;
    if (function == 6) return value != reference;
    if (function == 7) return value >= reference;
    return true;
}
vec4 mixed(vec4 colour, vec4 texel, int mixing, int flags) {
    if (mixing == 1 || mixing == 7) return texel;
    if (mixing == 4) return texel * colour;
    return vec4(texel.rgb * colour.rgb, (flags & 2) != 0 ? texel.a : colour.a);
}
vec4 fogged(vec4 c, float fog, vec4 fog_colour, int flags) {
#ifdef CUT_FOG
    return c;
#endif
    if ((flags & 4) == 0) return c;
    float kept = clamp(fog, 0.0, 1.0);
    return vec4(c.rgb * kept + fog_colour.rgb * (1.0 - kept), c.a);
}
#ifdef CUT_SECOND
#define SECOND_PASS false
#else
#define SECOND_PASS ((flags & 8) != 0)
#endif
void main() {
    vec4 colour = v_colour / v_w;
    vec4 c = colour;
#ifdef CUT_FLAT
    result = c;
    return;
#endif
#ifdef FIRST_TEXTURE
    // uploaded as they are: blue first
    if ((flags & 1) != 0) {
        vec4 texel = fetch(picture_a, picture_b, v_unit, vec3(v_uv, v_layer)).bgra;
        c = mixed(colour, texel, mixing, flags);
    }
#endif
#ifndef NO_DISCARD
    // Left out of the program for batches whose test cannot fail (`Batch::discard_free`): a
    // shader with a `discard` cannot have its hidden pixels rejected before it runs.
    if (alpha_function != 0 && !passes(alpha_function, alpha_reference, c.a)) discard;
#endif
#ifdef FIRST_FOG
    c = fogged(c, v_fog / v_w, fog_colour, flags);
#endif
#ifdef SECOND
    if (SECOND_PASS) {
        vec4 under = floor(c * 255.0 + 0.5) / 255.0;
        vec4 colour2 = v_colour2 / v_w;
        vec4 d = colour2;
#ifdef SECOND_TEXTURE
        if ((flags2 & 1) != 0) {
            vec4 texel = fetch(picture2_a, picture2_b, v_unit2, vec3(v_uv2, v_layer2)).bgra;
            d = mixed(colour2, texel, mixing2, flags2);
        }
#endif
        if (alpha_function2 == 0 || passes(alpha_function2, alpha_reference2, d.a)) {
#ifdef SECOND_FOG
            d = fogged(d, v_fog2 / v_w, fog_colour2, flags2);
#endif
            c = vec4(d.rgb * d.a + under.rgb * (1.0 - d.a), d.a * d.a + under.a * (1.0 - d.a));
        } else {
            c = under;
        }
    }
#endif
    result = c;
}
";

/// A picture over a rectangle of the target: `flip` for what the GL's own origin has upside
/// down (the frame buffer object's).
pub(super) const QUAD_VS: &CStr = c"#version 300 es
uniform float flip;
out vec2 v_uv;
void main() {
    vec2 corner = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    v_uv = vec2(corner.x, flip > 0.5 ? 1.0 - corner.y : corner.y);
    gl_Position = vec4(corner.x * 2.0 - 1.0, 1.0 - corner.y * 2.0, 0.0, 1.0);
}
";

/// A picture of the frame's shape in a window of another: `fit` is the picture's place as the
/// window's fractions (left, top, width, height).
pub(super) const FIT_VS: &CStr = c"#version 300 es
uniform float flip;
uniform vec4 fit;
out vec2 v_uv;
out vec2 v_win;
void main() {
    vec2 corner = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    v_win = corner;
    vec2 uv = (corner - fit.xy) / fit.zw;
    v_uv = vec2(uv.x, flip > 0.5 ? 1.0 - uv.y : uv.y);
    gl_Position = vec4(corner.x * 2.0 - 1.0, 1.0 - corner.y * 2.0, 0.0, 1.0);
}
";

/// The canvas's written pixels only.
pub(super) const OVERLAY_FS: &CStr = c"#version 300 es
precision mediump float;
in vec2 v_uv; out vec4 result; uniform sampler2D canvas;
void main() {
    vec4 pixel = texture(canvas, v_uv); // blue first
    uvec4 bytes = uvec4(pixel * 255.0 + 0.5);
    if (bytes == uvec4(0xFF, 0x00, 0xFF, 0xA5)) discard;
    // A top byte between 0 and 255 is a translucent white's alpha (a button picture's edge).
    float alpha = (bytes.a == 0u || bytes.a == 255u) ? 1.0 : pixel.a;
    result = vec4(pixel.bgr, alpha);
}
";

/// A whole picture of the game's bytes (blue first).
///
/// Beside the picture (`v_uv` outside the texture) the window shows black (`bars`, or no wash
/// yet) or the menu's wash: its background picture blurred and darkened, over the whole window
/// (`v_win`).
pub(super) const CANVAS_FS: &CStr = c"#version 300 es
precision mediump float;
in vec2 v_uv; in vec2 v_win; out vec4 result;
uniform sampler2D canvas; uniform sampler2D wash; uniform float bars; uniform float has_wash;
void main() {
    if (v_uv != clamp(v_uv, 0.0, 1.0)) {
        result = (bars > 0.5 || has_wash < 0.5)
            ? vec4(0.0, 0.0, 0.0, 1.0) : vec4(texture(wash, v_win).bgr, 1.0);
        return;
    }
    vec2 half_texel = 0.5 / vec2(textureSize(canvas, 0));
    result = vec4(texture(canvas, clamp(v_uv, half_texel, 1.0 - half_texel)).bgr, 1.0);
}
";

/// The frame buffer object's picture of a menu whose 3D was drawn larger (`HighTarget`): where it
/// is still what it was when the 3D was drawn (`kept`) the larger 3D picture is taken, where the
/// game drew over it since (the text and the buttons) the frame's own picture, beside it as
/// `CANVAS_FS`.
pub(super) const MERGE_FS: &CStr = c"#version 300 es
precision highp float;
in vec2 v_uv; in vec2 v_win; out vec4 result;
uniform sampler2D canvas; uniform sampler2D kept; uniform sampler2D high;
uniform sampler2D wash; uniform float bars; uniform float has_wash;
void main() {
    if (v_uv != clamp(v_uv, 0.0, 1.0)) {
        result = (bars > 0.5 || has_wash < 0.5)
            ? vec4(0.0, 0.0, 0.0, 1.0) : vec4(texture(wash, v_win).bgr, 1.0);
        return;
    }
    ivec2 size = textureSize(canvas, 0);
    ivec2 source = min(ivec2(v_uv * vec2(size)), size - 1);
    vec2 half_texel = 0.5 / vec2(size);
    vec2 at = clamp(v_uv, half_texel, 1.0 - half_texel);
    if (texelFetch(canvas, source, 0) == texelFetch(kept, source, 0)) {
        result = vec4(texture(high, at).rgb, 1.0);
    } else {
        result = vec4(texture(canvas, at).rgb, 1.0);
    }
}
";

/// The frame buffer object's picture (red first), beside it as `CANVAS_FS`.
pub(super) const TARGET_FS: &CStr = c"#version 300 es
precision mediump float;
in vec2 v_uv; in vec2 v_win; out vec4 result;
uniform sampler2D canvas; uniform sampler2D wash; uniform float bars; uniform float has_wash;
void main() {
    if (v_uv != clamp(v_uv, 0.0, 1.0)) {
        result = (bars > 0.5 || has_wash < 0.5)
            ? vec4(0.0, 0.0, 0.0, 1.0) : vec4(texture(wash, v_win).bgr, 1.0);
        return;
    }
    vec2 half_texel = 0.5 / vec2(textureSize(canvas, 0));
    result = vec4(texture(canvas, clamp(v_uv, half_texel, 1.0 - half_texel)).rgb, 1.0);
}
";
