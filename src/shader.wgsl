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
    @location(1) pos: vec2<f32>,
    @location(2) size: vec2<f32>,
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

    let position = model.position * instance.size + instance.pos;
    out.clip_position = projection.projection * vec4<f32>(position, 0.0, 1.0);
    
    out.uv = position;
    out.rect_color = instance.rect_color;
    out.rect_pos = instance.pos 
		   + vec2<f32>(instance.border_size.x, instance.border_size.y) 
		   + vec2<f32>(instance.outline_width + instance.outline_offset);
    out.rect_size = instance.size 
		    - vec2<f32>(instance.border_size.x + instance.border_size.z, instance.border_size.y + instance.border_size.w)
		    - vec2<f32>(instance.outline_width * 2 + instance.outline_offset * 2);
    
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
    let rect_dist = sdf_rounded_rect(
        in.uv - in.rect_pos - (in.rect_size / 2.0),
        in.rect_size / 2.0,
        in.border_radius
    );
    let rect_alpha = 1.0 - smoothstep(0.0, 2.0, rect_dist);
    let rect_color = vec4<f32>(in.rect_color.rgb, in.rect_color.a * rect_alpha);

    let border_size = in.rect_size + vec2<f32>((in.border_size.x + in.border_size.z), (in.border_size.y + in.border_size.w)) / 2.0;
    let border_pos = in.rect_pos - vec2<f32>(in.border_size.x, in.border_size.y) / 2.0;
    let border_dist = sdf_rounded_rect(
        in.uv - border_pos - (border_size / 2.0),
        border_size / 2.0,
        in.border_radius
    );
    let border_alpha = 1.0 - smoothstep(0.0, 2.0, border_dist);
    let border_color = vec4<f32>(in.border_color.rgb, in.border_color.a * border_alpha);

    let outline_offset_size = border_size + vec2<f32>(in.outline_offset) * 2.0;
    let outline_offset_pos = border_pos - vec2<f32>(in.outline_offset);
    let outline_offset_dist = sdf_rounded_rect(
        in.uv - outline_offset_pos - (outline_offset_size / 2.0),
        outline_offset_size / 2.0,
        in.border_radius
    );

    let outline_size = outline_offset_size + vec2<f32>(in.outline_width) * 2.0;
    let outline_pos = outline_offset_pos - vec2<f32>(in.outline_width);
    let outline_dist = sdf_rounded_rect(
        in.uv - outline_pos - (outline_size / 2.0),
        outline_size / 2.0,
        in.border_radius
    );
    let outline_alpha = 1.0 - smoothstep(0.0, 2.0, outline_dist);
    let outline_color = vec4<f32>(in.outline_color.rgb, in.outline_color.a * outline_alpha);

    let blended_border = mix(rect_color, border_color, smoothstep(0.0, 1.0, rect_dist));
    let blended_offset_outline = mix(blended_border, vec4<f32>(0.0), smoothstep(0.0, 1.0, border_dist));
    let blended_outline = mix(blended_offset_outline, outline_color, smoothstep(0.0, 1.0, outline_offset_dist));
    return blended_outline;
}
