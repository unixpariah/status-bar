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
    @location(7) outline_width: f32,
    @location(8) outline_offset: f32,
    @location(9) outline_color: vec4<f32>,
};

struct InstanceInput {
    @location(1) rect_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) rect_color: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_size: vec4<f32>,
    @location(6) border_color: vec4<f32>,
    @location(7) outline_width: f32,
    @location(8) outline_offset: f32,
    @location(9) outline_color: vec4<f32>,
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

    // This thing has to be properly done
    let border_width = instance.border_size.x + instance.border_size.z + (instance.outline_width + instance.outline_offset) * 2;
    let border_height = instance.border_size.y + instance.border_size.w + (instance.outline_width + instance.outline_offset) * 2;

    let adjusted_size = instance.rect_size + vec2<f32>(border_width, border_height);
    let adjusted_pos = instance.rect_pos - vec2<f32>(instance.border_size.x, instance.border_size.y) - vec2<f32>(instance.outline_width);
    let position = model.position * adjusted_size + adjusted_pos;

    out.clip_position = projection.projection * vec4<f32>(position, 0.0, 1.0);

    out.uv = position;
    out.rect_color = instance.rect_color;
    out.rect_pos = instance.rect_pos;
    out.rect_size = instance.rect_size;
    out.border_radius = instance.border_radius;
    out.border_size = instance.border_size;
    out.border_color = instance.border_color;
    out.outline_width = instance.outline_width;
    out.outline_offset = instance.outline_offset;
    out.outline_color = instance.outline_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = sdf_rounded_rect(in.uv - in.rect_pos - (in.rect_size / 2.0), in.rect_size / 2.0, in.border_radius);
    let smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, dist);
    let color = vec4<f32>(in.rect_color.rgb, in.rect_color.a * smoothed_alpha);

    let total_size = in.rect_size + vec2<f32>((in.border_size.x + in.border_size.z), (in.border_size.y + in.border_size.w)) / 2.0;
    let total_pos = in.rect_pos - vec2<f32>(in.border_size.x, in.border_size.y) / 2.0;
    let border_dist = sdf_rounded_rect(in.uv - total_pos - (total_size / 2.0), total_size / 2.0, in.border_radius);
    let border_smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, border_dist);
    let border_color = vec4<f32>(in.border_color.rgb, in.border_color.a * border_smoothed_alpha);

    let tot_size = total_size + vec2<f32>(in.outline_width) * 2.0;
    let tot_pos = total_pos - vec2<f32>(in.outline_width);
    let outline_dist = sdf_rounded_rect(in.uv - tot_pos - (tot_size / 2.0), tot_size / 2.0, in.border_radius);
    let outline_smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, outline_dist);
    let outline_color = vec4<f32>(in.outline_color.rgb, in.outline_color.a * outline_smoothed_alpha);

    return mix(mix(color, border_color, smoothstep(0.0, 1.0, dist)), outline_color, smoothstep(0.0, 1.0, border_dist));
}
