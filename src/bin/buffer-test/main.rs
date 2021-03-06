use rendertoy::*;

fn main() {
    let rtoy = Rendertoy::new();

    let tex_key = TextureKey::new(256, 256, Format::R16G16B16A16_SFLOAT);

    let gradients_tex = compute_tex(
        tex_key,
        load_cs(asset!("shaders/buffer_test.glsl")),
        shader_uniforms!(
            buf0: upload_array_buffer(Box::new(vec![0.04f32, 0.3f32, 0.8f32, 0.0f32])),
            buf1: upload_array_buffer(Box::new(vec![0.5f32, 0.5f32])),
        ),
    );

    rtoy.draw_forever(|_| gradients_tex.clone());
}
