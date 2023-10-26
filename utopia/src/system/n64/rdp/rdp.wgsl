struct Scissor {
    scale: vec2<f32>
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> scissor: Scissor;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let x = (model.position[0] * 2.0 / scissor.scale[0]) - 1.0;
    let y = 1.0 - (model.position[1] * 2.0 / scissor.scale[1]);
    let z = model.position[2] / 65536.0;
    out.clip_position = vec4<f32>(x, y, z, 1.0);
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    return out;
}

struct Combiner {
    rgb0: vec4<i32>,
    alpha0: vec4<i32>,
    rgb1: vec4<i32>,
    alpha1: vec4<i32>,
    blend0: vec4<i32>,
    blend1: vec4<i32>,
    prim_color: vec4<f32>,
    env_color: vec4<f32>,
    blend_color: vec4<f32>,
    fog_color: vec4<f32>,
    cycle_type: i32,
}

struct Inputs {
    tex0: vec4<f32>,
    shade: vec4<f32>,
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> combiner: Combiner;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if combiner.cycle_type == 3 {
        return in.color;
    }

    let tex0 = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // TODO: 2-cycle mode
    let inputs = Inputs(tex0, in.color);
    let combined = vec4(rgb(inputs, 0), rgb(inputs, 1), rgb(inputs, 2), alpha(inputs));
    let blended = blend(in.color[3], combined);
    return blended;
}

fn rgb(inputs: Inputs, element: i32) -> f32 {
    let sub_a = rgb_input(inputs, element, combiner.rgb1[0]);
    let sub_b = rgb_input(inputs, element, combiner.rgb1[1]);
    let mul = rgb_input(inputs, element, combiner.rgb1[2]);
    let add = rgb_input(inputs, element, combiner.rgb1[3]);
    return (sub_a - sub_b) * mul + add;
}

fn alpha(inputs: Inputs) -> f32 {
    let sub_a = alpha_input(inputs, combiner.alpha1[0]);
    let sub_b = alpha_input(inputs, combiner.alpha1[1]);
    let mul = alpha_input(inputs, combiner.alpha1[2]);
    let add = alpha_input(inputs, combiner.alpha1[3]);
    return (sub_a - sub_b) * mul + add;
}

fn rgb_input(inputs: Inputs, element: i32, source: i32) -> f32 {
    switch source {
        case 0: { return 0.0; } // Previous colour
        case 1, 2: { return inputs.tex0[element]; }
        case 3: { return combiner.prim_color[element]; }
        case 4: { return inputs.shade[element]; }
        case 5: { return combiner.env_color[element]; }
        case 6, 7: { return 1.0; } // 'Key' constants
        case 8: { return 0.0; } // Previous alpha
        case 9, 10: { return inputs.tex0[3]; }
        case 11: { return combiner.prim_color[3]; }
        case 12: { return inputs.shade[3]; }
        case 13: { return combiner.env_color[3]; }
        case 14, 15: { return 1.0; } // LOD Fraction
        case 16 { return 1.0; } // Noise
        case 17, 18: { return 1.0; } // Convert K constants
        case 19: { return 1.0; } // Constant value 1.0
        default: { return 0.0; }
    }
}

fn alpha_input(inputs: Inputs, source: i32) -> f32 {
    switch source {
        case 0: { return 0.0; } // Previous alpha
        case 1, 2: { return inputs.tex0[3]; }
        case 3: { return combiner.prim_color[3]; }
        case 4: { return inputs.shade[3]; }
        case 5: { return combiner.env_color[3]; }
        case 6, 7: { return 1.0; } // LOD Fraction
        case 8: { return 1.0; } // Constant value 1.0
        default: { return 0.0; }
    }
}

fn blend(shade: f32, input: vec4<f32>) -> vec4<f32> {
    let p = combiner.blend1[0];
    let m = combiner.blend1[1];
    let a = combiner.blend1[2];
    let b = combiner.blend1[3];

    switch b {
        // TODO: 'B' modes other than '0'
        default: {
            if p != 1 && m != 1 {
                return vec4(pm_select(p, input) + pm_select(m, input), 1.0);
            } else if p != 1 {
                return vec4(pm_select(p, input), alpha_select(a, shade, input[3]));
            } else if m != 1 {
                return vec4(pm_select(m, input), 1.0 - alpha_select(a, shade, input[3]));
            } else {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
}

fn pm_select(source: i32, color: vec4<f32>) -> vec3<f32> {
    switch source {
        case 0: { return color.xyz; }
        case 1: { return vec3(0.0, 0.0, 0.0); } // Should be unreachable
        case 2: { return combiner.blend_color.xyz; }
        default: { return combiner.fog_color.xyz; }
    }
}

fn alpha_select(source: i32, shade: f32, alpha: f32) -> f32 {
    switch source {
        case 0: { return alpha; }
        case 1: { return combiner.fog_color[3]; }
        case 2: { return shade; }
        default: { return 0.0; }
    }
}