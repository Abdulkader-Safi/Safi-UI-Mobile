// text.glsl — font-atlas glyph sampling (PRD §8.2).
//
// PLACEHOLDER until todo 16 lands the fontdue atlas + per-glyph UVs.
// For now this samples a uniform atlas texture with the supplied UVs and
// modulates by colour. Real glyph-rect packing and per-instance UV emission
// arrive with the atlas in todo 16.

#version 450

#ifdef VERTEX
layout(location = 0) in vec2 a_corner;        // unit-quad corner
layout(location = 1) in vec4 a_rect;          // x, y, width, height (px)
layout(location = 2) in vec4 a_uv;            // u0, v0, u1, v1
layout(location = 3) in vec4 a_color;         // RGBA, 0..1

layout(set = 1, binding = 0) uniform Uniforms {
    vec2 viewport_size;
} u;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv;

void main() {
    vec2 px = a_rect.xy + a_corner * a_rect.zw;
    vec2 clip = (px / u.viewport_size) * 2.0 - 1.0;
    clip.y = -clip.y;
    gl_Position = vec4(clip, 0.0, 1.0);

    v_color = a_color;
    v_uv = mix(a_uv.xy, a_uv.zw, a_corner);
}
#endif

#ifdef FRAGMENT
layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_uv;

layout(set = 2, binding = 0) uniform sampler2D u_atlas;

layout(location = 0) out vec4 frag;

void main() {
    float alpha = texture(u_atlas, v_uv).r;
    frag = vec4(v_color.rgb, v_color.a * alpha);
}
#endif
