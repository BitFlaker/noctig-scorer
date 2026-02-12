const CORNER_RADIUS_PX: f32 = 6.0;

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> x_data: array<f32>;
@group(0) @binding(2) var<storage, read> y_data: array<f32>;
@group(0) @binding(3) var<storage, read> val_data: array<f32>;
@group(0) @binding(4) var<storage, read> colors: array<vec3<f32>>;

struct Uniforms {
    resolution: vec2f,
    min_value: f32,
    max_value: f32,
    x_count: u32,
    y_count: u32,
    color_map_size: u32,
};

struct VertexIn {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var output: VertexOut;

    let x = f32(in.vertex_index & 1u);
    let y = f32((in.vertex_index >> 1u) & 1u);

    output.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    output.uv = vec2<f32>(x, y);

    return output;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4f {
    // Convert UV coordinates to grid indices
    let x_index = u32(input.uv.x * f32(uniforms.x_count));
    let y_index = u32(input.uv.y * f32(uniforms.y_count));

    // Clamp indices to valid range
    let x_idx = clamp(x_index, 0u, uniforms.x_count - 1u);
    let y_idx = clamp(y_index, 0u, uniforms.y_count - 1u);

    // Access 2D array as flattened 1D array (row-major order)
    let grid_index = y_idx * uniforms.x_count + x_idx;
    let cell_value = val_data[grid_index];

    // Normalize cell value to [0, 1] based on min/max
    let normalized_value = (cell_value - uniforms.min_value) / (uniforms.max_value - uniforms.min_value);

    // Get color from colormap
    let color = color_map(normalized_value);

    // Get corner radius UV
    let corner_radius_uv = vec2<f32>(
        CORNER_RADIUS_PX / uniforms.resolution.x,
        CORNER_RADIUS_PX / uniforms.resolution.y
    );

    // Get alpha for rounded corners
    var alpha = 1.0;
    let pixel_coord = input.uv * uniforms.resolution;
    let edge_dist = min(pixel_coord, uniforms.resolution - pixel_coord);
    if (edge_dist.x < CORNER_RADIUS_PX && edge_dist.y < CORNER_RADIUS_PX) {
        let corner_pos = vec2<f32>(CORNER_RADIUS_PX) - edge_dist;
        let dist = length(corner_pos);
        if (dist > CORNER_RADIUS_PX) {
            discard;
        }
        alpha = smoothstep(CORNER_RADIUS_PX, CORNER_RADIUS_PX - 1.0, dist);
    }

    return vec4<f32>(color, alpha);
}

fn color_map(value: f32) -> vec3<f32> {
    let max_len = uniforms.color_map_size - 1;

    // Map the value to the colormap closest color map indices
    let t = clamp(value, 0.0, 1.0);
    let scaled = t * f32(max_len);
    let idx = u32(scaled);
    let frac = scaled - f32(idx);

    if idx >= max_len {
        return colors[max_len];
    }

    // Linear interpolation between both color stops
    let color1 = colors[u32(idx)];
    let color2 = colors[u32(idx) + 1];

    let r = color1.r + (color2.r - color1.r) * frac;
    let g = color1.g + (color2.g - color1.g) * frac;
    let b = color1.b + (color2.b - color1.b) * frac;

    return vec3(r, g, b);
}
