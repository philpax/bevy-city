#define_import_path gta::common

// adaptation of https://stackoverflow.com/a/24319877
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let hsv = vec3<f32>((100.0 + hsv.x) % 1.0, hsv.yz);
    let hue_slice: f32 = 6.0 * hsv.x;
    let hue_slice_integer: f32 = floor(hue_slice);
    let hue_slice_interpolant: f32 = hue_slice - hue_slice_integer;
    let temp_rgb = vec3<f32>(
        hsv.z * (1.0 - hsv.y),
        hsv.z * (1.0 - hsv.y * hue_slice_interpolant),
        hsv.z * (1.0 - hsv.y * (1.0 - hue_slice_interpolant))
    );
    let is_odd_slice: f32 = hue_slice_integer % 2.0;
    let three_slice_selector: f32 = 0.5 * (hue_slice_integer - is_odd_slice);
    let scrolling_rgb_for_even_slices = vec3<f32>(hsv.z, temp_rgb.zx);
    let scrolling_rgb_for_odd_slices = vec3<f32>(temp_rgb.y, hsv.z, temp_rgb.x);
    let scrolling_rgb = mix(scrolling_rgb_for_even_slices, scrolling_rgb_for_odd_slices, is_odd_slice);
    let is_not_first_slice: f32 = clamp(three_slice_selector, 0.0, 1.0);
    let is_not_second_slice: f32 = clamp(three_slice_selector - 1.0, 0.0, 1.0);
    return mix(
        scrolling_rgb.xyz,
        mix(scrolling_rgb.zxy, scrolling_rgb.yzx, is_not_second_slice),
        is_not_first_slice
    );
}