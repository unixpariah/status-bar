struct ProjectionUniform {
    projection: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> projection: ProjectionUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) rect_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) rect_color: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_size: vec4<f32>,
    @location(6) border_color: vec4<f32>,
};

struct InstanceInput {
    @location(1) rect_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) rect_color: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_size: vec4<f32>,
    @location(6) border_color: vec4<f32>,
}

// MIT License. © 2023 Inigo Quilez, Munrocket
// https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4
// https://compute.toys/view/398
fn sdf_rounded_rect(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var x = r.x;
    var y = r.y;
    x = select(r.z, r.x, p.x > 0.);
    y = select(r.w, r.y, p.x > 0.);
    x = select(y, x, p.y > 0.);
    let q = abs(p) - b + x;
    return min(max(q.x, q.y), 0.) + length(max(q, vec2<f32>(0.))) - x;
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let border_width = instance.border_size.x + instance.border_size.z;
    let border_height = instance.border_size.y + instance.border_size.w;

    let adjusted_size = instance.rect_size + vec2<f32>(border_width, border_height);
    let adjusted_pos = instance.rect_pos - vec2<f32>(instance.border_size.x, instance.border_size.y);
    let position = model.position * adjusted_size + adjusted_pos;

    out.clip_position = projection.projection * vec4<f32>(position, 0.0, 1.0);

    out.uv = position;
    out.rect_color = instance.rect_color;
    out.rect_pos = instance.rect_pos;
    out.rect_size = instance.rect_size;
    out.border_radius = instance.border_radius;
    out.border_size = instance.border_size;
    out.border_color = instance.border_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = sdf_rounded_rect(in.uv - in.rect_pos - (in.rect_size / 2.0), in.rect_size / 2.0, in.border_radius);
    let smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, dist);
    let color = vec4<f32>(in.rect_color.rgb, in.rect_color.a * smoothed_alpha);

    let total_size = in.rect_size + vec2<f32>(in.border_size.x + in.border_size.z, in.border_size.y + in.border_size.w);
    let total_pos = in.rect_pos - vec2<f32>(in.border_size.x, in.border_size.y);

    let border_dist = sdf_rounded_rect(in.uv - total_pos - (total_size / 2.0), total_size / 2.0, in.border_radius);
    let border_smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, border_dist);
    let border_color = vec4<f32>(in.border_color.rgb, in.border_color.a * border_smoothed_alpha);

    return mix(color, border_color, smoothstep(0.0, 1.0, dist));
    //let shadow_softness = 30.0;
    //let shadow_offset = vec2<f32>(0.0, 10.0);
    //let shadow_distance = sdf_rounded_rect(in.uv - in.rect_pos + shadow_offset - (in.rect_size / 2.0), in.rect_size / 2.0, in.border_radius);
    //let shadow_alpha = 1.0 - smoothstep(-shadow_softness, shadow_softness, shadow_distance);
    //let shadow_color = vec4<f32>(0.4, 0.4, 0.4, 1.0);

    //return mix(color, shadow_color, shadow_alpha - smoothed_alpha);
}
