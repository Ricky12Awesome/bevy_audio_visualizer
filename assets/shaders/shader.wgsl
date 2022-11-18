#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_view_types

@group(1) @binding(0)
var<uniform> arr: array<vec4<f32>, 4096>;

@group(1) @binding(1)
var<uniform> len: u32;

fn get_value(index: u32) -> f32 {
    let upper = u32(floor(f32(index) / 4.));
    let lower = index % u32(4);

    return arr[upper][lower];
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // for ome reason bevy removed view.width and view.height
    // you have to do it like this
    let width = f32(view.viewport[2]);
    let height = f32(view.viewport[3]);
    let len = f32(len);
    let size = width / len;
    let index = floor(position.x / size);

    let value = get_value(u32(index));
    let center = height / 2.;

    let top = center + value * center;
    let bottom = center - value * center;

    if position.y <= top + value && position.y >= bottom {
        return vec4<f32>(0.5, 0.01 + (value / 2.0), 0.5, 1.);
    }

    return vec4<f32>(0.05, 0.05, 0.05, 1.);
}