use rendertoy::*;

fn main() {
    let mut rtoy = Rendertoy::new();

    let time = init_dynamic!(const_f32(0f32));
    let mouse_x = init_dynamic!(const_f32(0f32));
    let frame_index = init_dynamic!(const_u32(0));

    let tex_key_f16 = TextureKey {
        width: rtoy.width(),
        height: rtoy.height(),
        format: gl::RGBA16F,
    };

    let tex_key_f32 = TextureKey {
        width: tex_key_f16.width,
        height: tex_key_f16.height,
        format: gl::RGBA32F,
    };

    let primary_vis_tex = compute_tex(
        tex_key_f32,
        load_cs(asset!("shaders/area-lighting/primary_vis.glsl")),
        shader_uniforms!(
            g_mouseX: mouse_x.clone(),
            g_roughnessMultTex: load_tex_with_params(
                asset!("rendertoy::images/shadertoyWood.jpg"),
                TexParams { gamma: TexGamma::Linear }
            )
        ),
    );

    let light_samples_tex = compute_tex(
        tex_key_f16,
        load_cs(asset!("shaders/area-lighting/sample_lights.glsl")),
        shader_uniforms!(
            g_frameIndex: frame_index.clone(),
            g_mouseX: mouse_x.clone(),
            g_primaryVisTex: primary_vis_tex.clone(),
            g_whiteNoise: load_tex_with_params(
                asset!("rendertoy::images/noise/white_uniform_r_256.png"),
                TexParams { gamma: TexGamma::Linear }
            ),
        ),
    );

    let surface_samples_tex = compute_tex(
        tex_key_f16,
        load_cs(asset!("shaders/area-lighting/sample_surfaces.glsl")),
        shader_uniforms!(
            g_frameIndex: frame_index.clone(),
            g_mouseX: mouse_x.clone(),
            g_primaryVisTex: primary_vis_tex.clone(),
        ),
    );

    let filtered_light_tex = compute_tex(
        tex_key_f16,
        load_cs(asset!("shaders/area-lighting/filter_light.glsl")),
        shader_uniforms!(
            g_mouseX: mouse_x.clone(),
            g_primaryVisTex: primary_vis_tex.clone(),
            g_lightSamplesTex: light_samples_tex.clone(),
        ),
    );

    let filtered_surface_tex = compute_tex(
        tex_key_f16,
        load_cs(asset!("shaders/area-lighting/filter_surface.glsl")),
        shader_uniforms!(
            g_mouseX: mouse_x.clone(),
            g_primaryVisTex: primary_vis_tex.clone(),
            g_surfaceSamplesTex: surface_samples_tex.clone(),
        ),
    );

    let fused_lighting_tex = compute_tex(
        tex_key_f16,
        load_cs(asset!("shaders/area-lighting/fuse_lighting.glsl")),
        shader_uniforms!(
            g_filteredLightTex: filtered_light_tex.clone(),
            g_filteredSurfaceTex: filtered_surface_tex.clone(),
        ),
    );

    let accum_lighting_tex = init_dynamic!(load_tex(asset!("rendertoy::images/black.png")));

    redef_dynamic!(
        accum_lighting_tex,
        compute_tex(
            tex_key_f32,
            load_cs(asset!("shaders/area-lighting/temporal_accum.glsl")),
            shader_uniforms!(
                g_filteredLightingTex: fused_lighting_tex.clone(),
                g_prevOutputTex: accum_lighting_tex.clone(),
            )
        )
    );

    let final_tex = compute_tex(
        tex_key_f16,
        load_cs(asset!("shaders/area-lighting/normalize.glsl")),
        shader_uniforms!(g_inputTex: accum_lighting_tex,),
    );

    let mut t = 0.0f32;
    let mut fidx = 0u32;

    rtoy.draw_forever(|frame_state| {
        t += 0.01;
        fidx += 1;
        redef_dynamic!(time, const_f32(t));
        redef_dynamic!(mouse_x, const_f32(frame_state.mouse.pos.x));
        redef_dynamic!(frame_index, const_u32(fidx));

        final_tex.clone()
    });
}
