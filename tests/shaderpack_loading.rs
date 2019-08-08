use futures::executor::{block_on, ThreadPoolBuilder};
use nova_rs::shaderpack::*;
use std::path::PathBuf;

#[test]
fn default_nova_shaderpack() -> Result<(), ShaderpackLoadingFailure> {
    let mut threadpool = ThreadPoolBuilder::new()
        .name_prefix("default_nova_shaderpack")
        .create()
        .unwrap();
    let threadpool2 = threadpool.clone();

    let mut parsed: ShaderpackData = threadpool.run(load_nova_shaderpack(
        threadpool2,
        PathBuf::from("tests/data/shaderpacks/nova/DefaultShaderpack"),
    ))?;

    // Renderpass checking
    {
        let passes = &parsed.passes;
        assert_eq!(passes.len(), 2);

        ///// ////// /////
        ///// Pass 1 /////
        ///// ////// /////
        let pass = &passes[0];
        assert_eq!(pass.name, "Forward");

        // Texture outputs
        assert_eq!(pass.texture_outputs.len(), 1);
        let texture_output = &pass.texture_outputs[0];
        assert_eq!(texture_output.name, "LitWorld");
        assert_eq!(texture_output.clear, true);

        // Depth Texture
        assert_eq!(pass.depth_texture.is_some(), true);
        let depth_texture = pass.depth_texture.as_ref().unwrap();
        assert_eq!(depth_texture.name, "DepthBuffer");
        assert_eq!(depth_texture.clear, true);
        assert_eq!(depth_texture.pixel_format, PixelFormat::RGBA8);

        // Buffer Inputs
        assert_eq!(pass.input_buffers.len(), 2);
        let buffer_input = &pass.input_buffers[0];
        assert_eq!(buffer_input, "NovaMegaMesh_Vertices");
        let buffer_input = &pass.input_buffers[1];
        assert_eq!(buffer_input, "NovaMegaMesh_Indices");

        ///// ////// /////
        ///// Pass 2 /////
        ///// ////// /////
        let pass = &passes[1];
        assert_eq!(pass.name, "Final");

        // Texture inputs
        assert_eq!(pass.texture_inputs.len(), 1);
        let texture_input = &pass.texture_inputs[0];
        assert_eq!(texture_input, "LitWorld");

        // Texture outputs
        assert_eq!(pass.texture_outputs.len(), 1);
        let texture_output = &pass.texture_outputs[0];
        assert_eq!(texture_output.name, "Backbuffer");
        assert_eq!(texture_output.clear, false);
    }

    // Resources
    {
        let resources = &parsed.resources;

        assert_eq!(resources.textures.len(), 2);

        ///// ///////// /////
        ///// Texture 1 /////
        ///// ///////// /////
        let texture = &resources.textures[0];
        assert_eq!(texture.name, "LitWorld");
        let format = &texture.format;
        assert_eq!(format.pixel_format, PixelFormat::RGBA8);
        assert_eq!(format.dimension_type, TextureDimensionType::ScreenRelative);
        assert_eq!(format.width, 1.0_f32);
        assert_eq!(format.height, 1.0_f32);

        ///// ///////// /////
        ///// Texture 2 /////
        ///// ///////// /////
        let texture = &resources.textures[1];
        assert_eq!(texture.name, "DepthBuffer");
        let format = &texture.format;
        assert_eq!(format.pixel_format, PixelFormat::Depth);
        assert_eq!(format.dimension_type, TextureDimensionType::ScreenRelative);
        assert_eq!(format.width, 1.0_f32);
        assert_eq!(format.height, 1.0_f32);

        assert_eq!(resources.samplers.len(), 1);

        ///// ///////// /////
        ///// Sampler 1 /////
        ///// ///////// /////
        let sampler = &resources.samplers[0];
        assert_eq!(sampler.name, "Point");
        assert_eq!(sampler.filter, TextureFilter::Point);
        assert_eq!(sampler.wrap_mode, WrapMode::Clamp);
    }

    // Materials
    {
        assert_eq!(parsed.materials.len(), 5);

        parsed.materials.sort_by_cached_key(|m| m.name.clone());

        ///// ///////// /////
        ///// Final.mat /////
        ///// ///////// /////
        let material = &parsed.materials[0];
        assert_eq!(material.name, "final");
        assert_eq!(material.geometry_filter, "geometry_type::fullscreen_quad");

        assert_eq!(material.passes.len(), 1);
        let pass = &material.passes[0];
        assert_eq!(pass.name, "main");
        assert_eq!(pass.pipeline, "Final");
        assert_eq!(pass.material_name, material.name);

        assert_eq!(pass.bindings.contains_key("per_model_uniforms"), true);
        let binding = &pass.bindings["per_model_uniforms"];
        assert_eq!(binding, "NovaModelMatrixBuffer");

        ///// //////////////////// /////
        ///// gbuffers_terrain.mat /////
        ///// //////////////////// /////
        let material = &parsed.materials[1];
        assert_eq!(material.name, "gbuffers_terrain");
        assert_eq!(material.geometry_filter, "geometry_type::block AND not_transparent");

        assert_eq!(material.passes.len(), 1);
        let pass = &material.passes[0];
        assert_eq!(pass.name, "main");
        assert_eq!(pass.pipeline, "gbuffers_terrain");
        assert_eq!(pass.material_name, material.name);

        assert_eq!(pass.bindings.contains_key("per_model_uniforms"), true);
        let binding = &pass.bindings["per_model_uniforms"];
        assert_eq!(binding, "NovaModelMatrixBuffer");

        ///// /////// /////
        ///// gui.mat /////
        ///// /////// /////
        let material = &parsed.materials[2];
        assert_eq!(material.name, "gui");
        assert_eq!(material.geometry_filter, "geometry_type::gui");

        assert_eq!(material.passes.len(), 1);
        let pass = &material.passes[0];
        assert_eq!(pass.name, "main");
        assert_eq!(pass.pipeline, "gui");
        assert_eq!(pass.material_name, material.name);

        assert_eq!(pass.bindings.contains_key("per_model_uniforms"), true);
        let binding = &pass.bindings["per_model_uniforms"];
        assert_eq!(binding, "NovaModelMatrixBuffer");

        ///// ////////////////// /////
        ///// gui_background.mat /////
        ///// ////////////////// /////
        let material = &parsed.materials[3];
        assert_eq!(material.name, "gui_background");
        assert_eq!(material.geometry_filter, "geometry_type::gui_background");

        assert_eq!(material.passes.len(), 1);
        let pass = &material.passes[0];
        assert_eq!(pass.name, "main");
        assert_eq!(pass.pipeline, "gui");
        assert_eq!(pass.material_name, material.name);

        assert_eq!(pass.bindings.contains_key("per_model_uniforms"), true);
        let binding = &pass.bindings["per_model_uniforms"];
        assert_eq!(binding, "NovaModelMatrixBuffer");

        ///// //////////// /////
        ///// gui_text.mat /////
        ///// //////////// /////
        let material = &parsed.materials[4];
        assert_eq!(material.name, "gui_text");
        assert_eq!(material.geometry_filter, "geometry_type::text");

        assert_eq!(material.passes.len(), 1);
        let pass = &material.passes[0];
        assert_eq!(pass.name, "main");
        assert_eq!(pass.pipeline, "gui");
        assert_eq!(pass.material_name, material.name);

        assert_eq!(pass.bindings.contains_key("per_model_uniforms"), true);
        let binding = &pass.bindings["per_model_uniforms"];
        assert_eq!(binding, "NovaModelMatrixBuffer");
    }

    // Pipelines
    {
        assert_eq!(parsed.pipelines.len(), 3);

        parsed.pipelines.sort_by_cached_key(|m| m.name.clone());

        ///// ////////////// /////
        ///// final.pipeline /////
        ///// ////////////// /////
        let pipeline = &parsed.pipelines[0];
        assert_eq!(pipeline.name, "Final");
        assert_eq!(pipeline.pass, "Final");

        assert_eq!(pipeline.states.len(), 3);
        assert_eq!(pipeline.states.contains(&RasterizerState::DisableAlphaWrite), true);
        assert_eq!(pipeline.states.contains(&RasterizerState::DisableDepthWrite), true);
        assert_eq!(pipeline.states.contains(&RasterizerState::DisableDepthTest), true);

        assert_eq!(
            pipeline.vertex_shader,
            ShaderSource::Path(String::from("shaders/textured_unlit"))
        );
        assert_eq!(
            pipeline.fragment_shader,
            Some(ShaderSource::Path(String::from("shaders/image_passthrough")))
        );

        assert_eq!(pipeline.vertex_fields.len(), 2);
        let vertex_field = &pipeline.vertex_fields[0];
        assert_eq!(vertex_field.semantic_name, "position_in");
        assert_eq!(vertex_field.field, VertexField::Position);

        let vertex_field = &pipeline.vertex_fields[1];
        assert_eq!(vertex_field.semantic_name, "uv_in");
        assert_eq!(vertex_field.field, VertexField::UV0);

        ///// ///////////////////////// /////
        ///// gbuffers_terrain.pipeline /////
        ///// ///////////////////////// /////
        let pipeline = &parsed.pipelines[1];
        assert_eq!(pipeline.name, "gbuffers_terrain");
        assert_eq!(pipeline.pass, "Forward");

        assert_eq!(pipeline.states.len(), 1);
        assert_eq!(pipeline.states.contains(&RasterizerState::DisableAlphaWrite), true);

        assert_eq!(
            pipeline.vertex_shader,
            ShaderSource::Path(String::from("shaders/gbuffers_terrain"))
        );
        assert_eq!(
            pipeline.fragment_shader,
            Some(ShaderSource::Path(String::from("shaders/gbuffers_terrain")))
        );

        assert_eq!(pipeline.vertex_fields.len(), 9);
        let vertex_field = &pipeline.vertex_fields[0];
        assert_eq!(vertex_field.semantic_name, "position_in");
        assert_eq!(vertex_field.field, VertexField::Position);

        let vertex_field = &pipeline.vertex_fields[1];
        assert_eq!(vertex_field.semantic_name, "color_in");
        assert_eq!(vertex_field.field, VertexField::Color);

        let vertex_field = &pipeline.vertex_fields[2];
        assert_eq!(vertex_field.semantic_name, "uv_in");
        assert_eq!(vertex_field.field, VertexField::UV0);

        let vertex_field = &pipeline.vertex_fields[3];
        assert_eq!(vertex_field.semantic_name, "lightmap_uv_in");
        assert_eq!(vertex_field.field, VertexField::UV1);

        let vertex_field = &pipeline.vertex_fields[4];
        assert_eq!(vertex_field.semantic_name, "normal_in");
        assert_eq!(vertex_field.field, VertexField::Normal);

        let vertex_field = &pipeline.vertex_fields[5];
        assert_eq!(vertex_field.semantic_name, "tangent_in");
        assert_eq!(vertex_field.field, VertexField::Tangent);

        let vertex_field = &pipeline.vertex_fields[6];
        assert_eq!(vertex_field.semantic_name, "mix_tex_coord_in");
        assert_eq!(vertex_field.field, VertexField::MidTexCoord);

        let vertex_field = &pipeline.vertex_fields[7];
        assert_eq!(vertex_field.semantic_name, "virtual_texture_id_in");
        assert_eq!(vertex_field.field, VertexField::VirtualTextureId);

        let vertex_field = &pipeline.vertex_fields[8];
        assert_eq!(vertex_field.semantic_name, "mc_entity_id_in");
        assert_eq!(vertex_field.field, VertexField::McEntityId);

        ///// ////////////////// /////
        ///// gui.pipeline /////
        ///// ////////////////// /////
        let pipeline = &parsed.pipelines[2];
        assert_eq!(pipeline.name, "gui");
        assert_eq!(pipeline.pass, "Forward");
        assert_eq!(pipeline.depth_func, CompareOp::LessEqual);

        assert_eq!(pipeline.states.len(), 1);
        assert_eq!(pipeline.states.contains(&RasterizerState::DisableAlphaWrite), true);

        assert_eq!(pipeline.vertex_shader, ShaderSource::Path(String::from("shaders/gui")));
        assert_eq!(
            pipeline.fragment_shader,
            Some(ShaderSource::Path(String::from("shaders/gui")))
        );

        assert_eq!(pipeline.vertex_fields.len(), 4);
        let vertex_field = &pipeline.vertex_fields[0];
        assert_eq!(vertex_field.semantic_name, "position_in");
        assert_eq!(vertex_field.field, VertexField::Position);

        let vertex_field = &pipeline.vertex_fields[1];
        assert_eq!(vertex_field.semantic_name, "uv_in");
        assert_eq!(vertex_field.field, VertexField::UV0);

        let vertex_field = &pipeline.vertex_fields[2];
        assert_eq!(vertex_field.semantic_name, "color_in");
        assert_eq!(vertex_field.field, VertexField::Color);

        let vertex_field = &pipeline.vertex_fields[3];
        assert_eq!(vertex_field.semantic_name, "virtual_texture_id_in");
        assert_eq!(vertex_field.field, VertexField::VirtualTextureId);
    }

    Ok(())
}
