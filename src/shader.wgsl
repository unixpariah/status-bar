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
    @location(10) boxshadow_offset: vec2<f32>,
    @location(11) boxshadow_softness: f32,
    @location(12) boxshadow_color: vec4<f32>,
    @location(13) brightness: f32,
    @location(14) saturate: f32,
    @location(15) contrast: f32,
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
    @location(10) boxshadow_offset: vec2<f32>,
    @location(11) boxshadow_softness: f32,
    @location(12) boxshadow_color: vec4<f32>,
    @location(13) brightness: f32,
    @location(14) saturate: f32,
    @location(15) contrast: f32,
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
    out.boxshadow_offset = instance.boxshadow_offset;
    out.boxshadow_softness = instance.boxshadow_softness;
    out.boxshadow_color = instance.boxshadow_color;
    out.brightness = instance.brightness;
    out.saturate = instance.saturate;
    out.contrast = instance.contrast;

    return out;
}

// MIT License. © 2023 Inigo Quilez, Munrocket
// https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4
// https://compute.toys/view/398
fn sdf_rounded_rect(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var x = r.x;
    var y = r.y;
    x = select(r.z, r.x, p.x > 0.0);
    y = select(r.w, r.y, p.x > 0.0);
    x = select(y, x, p.y > 0.0);
    let q = abs(p) - b + x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - x;
}

fn brightness_matrix(brightness: f32) -> mat4x4<f32> {
    return mat4x4<f32>( 1, 0, 0, 0,
			0, 1, 0, 0,
			0, 0, 1, 0,
			brightness, brightness, brightness, 1 );
}

fn contrast_matrix(contrast: f32) -> mat4x4<f32> {
    let t = (1.0 - contrast) / 2.0;

    return mat4x4<f32>( contrast, 0, 0, 0,
			0, contrast, 0, 0,
			0, 0, contrast, 0,
			t, t, t, 1);
}

fn saturation_matrix(saturation: f32) -> mat4x4<f32> {
    let luminance = vec3<f32>(0.3086, 0.6094, 0.0820);
    let one_minus_sat = 1.0 - saturation;

    var red: vec3<f32> = vec3<f32>(luminance.x * one_minus_sat);
    red += vec3<f32>(saturation, 0, 0);

    var green: vec3<f32> = vec3<f32>(luminance.y * one_minus_sat);
    green += vec3<f32>(0, saturation, 0);

    var blue: vec3<f32> = vec3<f32>(luminance.z * one_minus_sat);
    blue += vec3<f32>(0, 0, saturation);

    return mat4x4<f32>(
        vec4<f32>(red, 0.0),
        vec4<f32>(green, 0.0),
        vec4<f32>(blue, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

//mat4 saturationMatrix( float saturation )
//{
//    vec3 luminance = vec3( 0.3086, 0.6094, 0.0820 );
//    
//    float oneMinusSat = 1.0 - saturation;
//    
//    vec3 red = vec3( luminance.x * oneMinusSat );
//    red+= vec3( saturation, 0, 0 );
//    
//    vec3 green = vec3( luminance.y * oneMinusSat );
//    green += vec3( 0, saturation, 0 );
//    
//    vec3 blue = vec3( luminance.z * oneMinusSat );
//    blue += vec3( 0, 0, saturation );
//    
//    return mat4( red,     0,
//                 green,   0,
//                 blue,    0,
//                 0, 0, 0, 1 );
//}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos: vec2<f32> = in.rect_pos;
    var size: vec2<f32> = in.rect_size;
    var dist: f32 = sdf_rounded_rect(
        in.uv - pos - (size / 2.0),
        size / 2.0,
        in.border_radius
    );
    let rect_alpha = 1.0 - smoothstep(0.0, 2.0, dist);
    var color: vec4<f32> = vec4<f32>(in.rect_color.rgb, in.rect_color.a * rect_alpha);

    if (in.border_size.x > 0.0 || in.border_size.y > 0.0 || in.border_size.w > 0.0 || in.border_size.z > 0.0) {
	size += vec2<f32>((in.border_size.x + in.border_size.z), (in.border_size.y + in.border_size.w)) / 2.0;
    	pos -= vec2<f32>(in.border_size.x, in.border_size.y) / 2.0;
    	let border_dist = sdf_rounded_rect(
    	    in.uv - pos - (size / 2.0),
    	    size / 2.0,
    	    in.border_radius
    	);
    	let border_alpha = 1.0 - smoothstep(0.0, 2.0, border_dist);
    	let border_color = vec4<f32>(in.border_color.rgb, in.border_color.a * border_alpha);

	color = mix(color, border_color, smoothstep(0.0, 1.0, dist));
	dist = border_dist;
    }

   // if (in.boxshadow_offset.x != 0 || in.boxshadow_offset.y != 0) {
   //     let shadow_distance = sdf_rounded_rect(in.uv - pos + in.boxshadow_offset - (size / 2.0), size / 2.0, in.border_radius);
   // 	let shadow_alpha = 1.0 - smoothstep(-in.boxshadow_softness, in.boxshadow_softness, shadow_distance);
   // 	let border_alpha = 1.0 - smoothstep(0.0, 2.0, dist);
   // 	let quadColor = mix(vec4<f32>(0.0, 0.0, 0.0, 0.0), color, border_alpha);
   // 	color = mix(quadColor, in.boxshadow_color, shadow_alpha - border_alpha);
   //     dist = shadow_distance;
   // }

    if (in.outline_offset > 0.0) {
        size += vec2<f32>(in.outline_offset) * 2.0;
    	pos -= vec2<f32>(in.outline_offset);
        color = mix(color, vec4<f32>(0.0), smoothstep(0.0, 1.0, dist));
    	dist = sdf_rounded_rect(
    	    in.uv - pos - (size / 2.0),
    	    size / 2.0,
    	    in.border_radius
    	);
    }

    if (in.outline_width > 0.0) {
        size += vec2<f32>(in.outline_width) * 2.0;
    	pos -= vec2<f32>(in.outline_width);
    	let outline_dist = sdf_rounded_rect(
    	    in.uv - pos - (size / 2.0),
    	    size / 2.0,
    	    in.border_radius
    	);
    	let outline_alpha = 1.0 - smoothstep(0.0, 2.0, outline_dist);
    	let outline_color = vec4<f32>(in.outline_color.rgb, in.outline_color.a * outline_alpha);

        color = mix(color, outline_color, smoothstep(0.0, 1.0, dist));
        dist = outline_dist;
    }

    return brightness_matrix(in.brightness) * contrast_matrix(in.contrast) * saturation_matrix(in.saturate) * color;
}
