// rect.glsl — solid + rounded-corner rectangle (PRD §8.2).
//
// Per-instance: rect (xywh), color (RGBA), radius (px).
// Two stages in one file: SHADER_STAGE chosen via the preprocessor.
// build.rs compiles this to both SPIR-V (Vulkan/Android) and MSL (Metal/iOS).

#version 450

#ifdef VERTEX
layout(location = 0) in vec2 a_corner;        // unit-quad corner (0,0)..(1,1)
layout(location = 1) in vec4 a_rect;          // x, y, width, height (px)
layout(location = 2) in vec4 a_color;         // RGBA, 0..1
layout(location = 3) in float a_radius;       // corner radius (px)

layout(set = 1, binding = 0) uniform Uniforms {
    vec2 viewport_size;
} u;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_local;        // px from top-left of this rect
layout(location = 2) out vec2 v_size;
layout(location = 3) out float v_radius;

void main() {
    vec2 px = a_rect.xy + a_corner * a_rect.zw;
    // map pixel-space (0..vp) -> clip-space (-1..1), Y down (Vulkan + Metal both flip in pipeline)
    vec2 clip = (px / u.viewport_size) * 2.0 - 1.0;
    clip.y = -clip.y;
    gl_Position = vec4(clip, 0.0, 1.0);

    v_color  = a_color;
    v_local  = a_corner * a_rect.zw;
    v_size   = a_rect.zw;
    v_radius = a_radius;
}
#endif

#ifdef FRAGMENT
layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_local;
layout(location = 2) in vec2 v_size;
layout(location = 3) in float v_radius;

layout(location = 0) out vec4 frag;

// Signed distance to a rounded box of half-size b with radius r, centered.
float sdf_rounded_box(vec2 p, vec2 b, float r) {
    vec2 d = abs(p) - b + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    vec2 half_size = v_size * 0.5;
    vec2 p = v_local - half_size;
    float d = sdf_rounded_box(p, half_size, v_radius);
    // 1px AA at the edge.
    float aa = 1.0 - smoothstep(-0.5, 0.5, d);
    frag = vec4(v_color.rgb, v_color.a * aa);
}
#endif
