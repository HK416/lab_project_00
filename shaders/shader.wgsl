const EPSILON: f32 = 1.192092896e-07f;

struct VertexOutput {
    @builtin(position) clip_position: vec4f, 
    @location(0) color: vec4f, 
}

struct CameraUniformLayout {
    camera: mat4x4f, 
    projection: mat4x4f, 
}

struct ObjectUniformLayout {
    world: mat4x4f, 
    color: vec4f, 
}

struct TransparentPassOutput {
    @location(0) accum: vec4f, 
    @location(1) reveal: f32, 
}

@group(0) @binding(0)
var<uniform> camera_data: CameraUniformLayout;
@group(1) @binding(0)
var<uniform> object_data: ObjectUniformLayout;
@group(0) @binding(0)
var accum: texture_2d<f32>;
@group(0) @binding(1)
var reveal: texture_2d<f32>;



@vertex
fn vs_main(@location(0) pos: vec3f) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera_data.projection * camera_data.camera * object_data.world * vec4f(pos, 1.0);
    out.color = object_data.color;
    return out;
}

@fragment
fn fs_opaque_main(in: VertexOutput) -> @location(0) vec4f {
    return in.color;
}

@fragment
fn fs_transparent_pass(in: VertexOutput) -> TransparentPassOutput {
    let depth = in.clip_position.z;
    let color = in.color;

    let weight: f32 = clamp(pow(min(1.0, color.a * 10.0) + 0.01, 3.0) * 1e8f * pow(1.0 - depth * 0.9, 3.0), 1e-2f, 3e3f);

    var out: TransparentPassOutput;
    out.accum = vec4f(color.rgb * color.a, color.a) * weight;
    out.reveal = color.a;

    return out;
}

@vertex
fn vs_composite_pass(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
    var position: vec4f;
    switch (index) {
        case 0u: {
            position = vec4f(-1.0, -1.0, 0.0, 1.0);
            break;
        }
        case 1u: {
            position = vec4f(-1.0, 1.0, 0.0, 1.0);
            break;
        }
        case 2u: {
            position = vec4f(1.0, -1.0, 0.0, 1.0);
            break;
        }
        case 3u: {
            position = vec4f(1.0, 1.0, 0.0, 1.0);
            break;
        }
        default { }
    }
    return position;
}

@fragment
fn fs_composite_pass(@builtin(position) clip_position: vec4f) -> @location(0) vec4f {
    let coords: vec2i = vec2i(clip_position.xy);
    
    let revealage: f32 = textureLoad(reveal, coords, 0).r;
    if (is_approximately_equal(revealage, 1.0)) {
        discard;
    }

    var accumulation: vec4f = textureLoad(accum, coords, 0);

    if (is_infinite(max(max(abs(accumulation.x), abs(accumulation.y)), abs(accumulation.z)))) {
        accumulation = vec4f(accumulation.a, accumulation.a, accumulation.a, accumulation.a);
    }

    let average_color = accumulation.rgb / max(accumulation.a, EPSILON);

    return vec4f(average_color, 1.0 - revealage);
}

fn is_infinite(v: f32) -> bool {
    return v != 0.0 && v * 2.0 == v;
}

fn is_approximately_equal(a: f32, b: f32) -> bool {
    return abs(a - b) <= min(abs(a), abs(b)) * EPSILON;
}
