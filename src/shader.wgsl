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
    @location(0) rect_color: vec4<f32>,
    @location(1) rect_size: vec2<f32>,
    @location(2) rect_pos: vec2<f32>,
    @location(3) border_radius: vec4<f32>,
};

struct InstanceInput {
    @location(2) rect_dim: vec4<f32>,
    @location(3) rect_color: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
}

fn sdRoundedBox(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
  var x = r.x;
  var y = r.y;
  x = select(r.z, r.x, p.x > 0.);
  y = select(r.w, r.y, p.x > 0.);
  x  = select(y, x, p.y > 0.);
  let q = abs(p) - b + x;
  return min(max(q.x, q.y), 0.) + length(max(q, vec2f(0.))) - x;
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let position = model.position * instance.rect_dim.zw + instance.rect_dim.xy;

    out.rect_color = instance.rect_color;
    out.clip_position = projection.projection * vec4<f32>(position, 0.0, 1.0);
    out.rect_pos = (projection.projection * vec4<f32>(instance.rect_dim.xy, 0.0, 0.0)).xy;
    out.rect_size = (projection.projection * vec4<f32>(instance.rect_dim.zw, 0.0, 0.0)).xy;
    out.border_radius = instance.border_radius;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = sdRoundedBox(in.clip_position.xy - in.rect_pos, in.rect_size, in.border_radius);

    return in.rect_color;
    //return vec4<f32>(dist);
}
