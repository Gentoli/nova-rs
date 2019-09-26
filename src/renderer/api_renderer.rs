use crate::mesh::MeshData;
use crate::renderer::rendergraph::{
    FullMaterialPassName, MaterialPassKey, MaterialPassMetadata, Pipeline, PipelineMetadata, Renderpass,
    RenderpassMetadata,
};
use crate::renderer::Renderer;
use crate::rhi;
use crate::rhi::{Device, ResourceBindingDescription, Swapchain};
use crate::settings::Settings;
use crate::shaderpack::{
    MaterialData, PipelineCreationInfo, RenderPassCreationInfo, ShaderpackData, ShaderpackResourceData,
    TextureCreateInfo,
};
use cgmath::Vector2;
use spirv_cross::spirv::Resource;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

pub fn new_dx12_renderer(settings: Settings) -> Box<dyn Renderer> {
    unimplemented!();
}

pub fn new_vulkan_renderer(settings: Settings) -> Box<dyn Renderer> {
    unimplemented!();
}

enum PlatformRendererCreationError {
    ApiNotSupported,
}

/// Actual renderer boi
///
/// A Renderer which is specialized for a graphics API
pub struct ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    device: GraphicsApi::Device,

    /// Flag for if we can render frames. If this is false then no frames get rendered, aka execute frame is a no-op
    can_render: bool,

    // Render graph data
    has_rendergraph: bool,

    /// All the current shaderpack's renderpasses, in submission order
    renderpasses: Vec<Renderpass<GraphicsApi>>,

    /// All the textures that the current render graph needs
    renderpass_textures: HashMap<String, GraphicsApi::Image>,
    renderpass_texture_infos: HashMap<String, TextureCreateInfo>,

    swapchain: Arc<GraphicsApi::Swapchain>,
}

impl<GraphicsApi> ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// Creates a new renderer
    pub fn new(settings: Settings) -> Result<Self, PlatformRendererCreationError> {
        let graphics_api = GraphicsApi::new(settings);

        let mut adapters = graphics_api.get_adapters();

        match adapters.len() {
            0 => Err(PlatformRendererCreationError::ApiNotSupported),
            _ => Ok(ApiRenderer {
                device: adapters.remove(0),
                can_render: true,
                has_rendergraph: false,
                renderpasses: Default::default(),
                renderpass_textures: Default::default(),
                renderpass_texture_infos: Default::default(),
                swapchain: graphics_api.get_swapchain(),
            }),
        }
    }

    fn destroy_render_passes(&mut self) {
        for renderpass in self.renderpasses {
            self.device.destroy_renderpass(renderpass.renderpass);
            self.device.destroy_framebuffer(renderpass.framebuffer);

            for pipeline in renderpass.pipelines {
                self.device.destroy_pipeline(pipeline.pipeline);

                for material_pass in pipeline.passes {
                    // TODO: Either find some way to store mesh data externally, or tell everyone that they have to
                    // reload meshes when the user changes shaderpack
                }
            }
        }

        self.renderpasses.clear();
    }

    fn destroy_rendergraph_resources(&mut self) {
        for (name, image) in self.renderpass_textures {
            self.device.destroy_image(image);
        }

        self.renderpass_textures.clear();
        self.renderpass_texture_infos.clear();

        // TODO: Destroy dynamic buffers when Nova supports them
    }

    fn create_rendergraph_resources(&mut self, resource_info: ShaderpackResourceData) {
        let swapchain: &GraphicsApi::Swapchain = self.swapchain.borrow();
        let swapchain_size = swapchain.get_size();

        // This is kinda garbage but I don't know a better way
        self.renderpass_textures = HashMap::with_capacity(resource_info.textures.len());
        resource_info.textures.iter().for_each(|info| {
            match self.device.create_image(info, &swapchain_size) {
                Ok(image) => {
                    self.renderpass_textures.insert(info.name, image);
                }
                Err(e) => error!("{}", e),
            };
        });

        self.renderpass_texture_infos = HashMap::with_capacity(resource_info.textures.len());
        resource_info.textures.iter().for_each(|info| {
            self.renderpass_texture_infos.insert(info.name, info.clone());
        });

        // TODO: Create rendergraph buffers, once the rendergraph loader can handle them
    }

    fn create_rendergraph_textures(&mut self, &texture_infos: &Vec<TextureCreateInfo>) {}

    fn create_render_passes(
        &mut self,
        passes: Vec<RenderPassCreationInfo>,
        pipelines: Vec<PipelineCreationInfo>,
        materials: Vec<MaterialData>,
    ) -> bool {
        let mut success = true;

        let mut total_num_descriptors = 0;
        for material_data in materials {
            for material_pass in material_data.passes {
                total_num_descriptors += material_pass.bindings.len();
            }
        }

        let descriptor_pool = self
            .device
            .create_descriptor_pool(total_num_descriptors as u32, 0, total_num_descriptors as u32)
            .unwrap();

        let swapchain: &GraphicsApi::Swapchain = self.swapchain.borrow();
        let swapchain_size = swapchain.get_size();

        for pass_info in passes {
            let mut renderpass: Renderpass<GraphicsApi> = Default::default();

            let mut renderpass_metadata: RenderpassMetadata = Default::default();
            renderpass_metadata.data = pass_info.clone();

            let mut output_images = Vec::<GraphicsApi::Image>::with_capacity(pass_info.texture_outputs.len());
            let mut attachment_errors = Vec::<String>::with_capacity(pass_info.texture_outputs.len());
            let mut framebuffer_size = Vector2::<f32>::new(0.0, 0.0);

            for attachment_info in pass_info.texture_outputs {
                if attachment_info.name == "Backbuffer" {
                    // Nova itself handles the backbuffer, but it needs renderpasses to be able to use it, so it
                    // needs some special handling
                    if pass_info.texture_outputs.len() == 0 {
                        renderpass.writes_to_backbuffer = true;
                    } else {
                        attachment_errors
                                .push(format!("Pass {} writes to the backbuffer and {} other textures, but that's not allowed. If a pass writes to the backbuffer, it can't write to any other textures", pass_info.name, pass_info.texture_outputs.len()))
                    }
                } else {
                    let image = self.renderpass_textures.get(&attachment_info.name).unwrap();
                    output_images.push(*image);

                    let image_info = self.renderpass_texture_infos.get(&attachment_info.name).unwrap();
                    let attachment_size = image_info.format.get_size_in_pixels(swapchain_size);

                    if framebuffer_size.x > 0.0 {
                        if attachment_size != framebuffer_size {
                            attachment_errors.push(format!("Attachment {} has a size of {:?}, but the framebuffer for pass {} has a size of {:?} - these must match! All attachments of a single renderpass must have the same size", attachment_info.name, attachment_size, pass_info.name, framebuffer_size));
                        }
                    } else {
                        framebuffer_size = attachment_size;
                    }
                }
            }

            if !attachment_errors.is_empty() {
                for error in attachment_errors {
                    error!("{}", error);
                }

                error!(
                    "Could not create renderpass {} because of errors in its attachment specifications",
                    pass_info.name
                );

                success = false;
                continue;
            }

            match self.device.create_renderpass(pass_info.clone()) {
                Ok(rhi_renderpass) => renderpass.renderpass = rhi_renderpass,
                Err(err) => {
                    error!("Could not create RHI object for renderpass {}: {}", pass_info.name, err);

                    success = false;
                    continue;
                }
            }

            match self
                .device
                .create_framebuffer(&renderpass.renderpass, &output_images, framebuffer_size.clone())
            {
                Ok(rhi_framebuffer) => renderpass.framebuffer = rhi_framebuffer,
                Err(err) => {
                    error!(
                        "Could not create framebuffer for renderpass {}: {}",
                        pass_info.name, err
                    );

                    success = false;
                    continue;
                }
            }

            renderpass.pipelines.reserve(pipelines.len());
            for pipeline_info in pipelines {
                if pipeline_info.pass == pipeline_info.name {
                    let mut bindings = HashMap::<String, ResourceBindingDescription>::new();

                    // TODO: Get bindings BEFORE MERGING

                    let pipeline_interface = self
                        .device
                        .create_pipeline_interface(&bindings, &pass_info.texture_outputs, &pass_info.depth_texture)
                        .unwrap();

                    match self.create_graphics_pipeline(pipeline_interface, &pipeline_info) {
                        Ok((pipeline, pipeline_metadata)) => {
                            let template_key = MaterialPassKey {
                                renderpass_index: self.renderpasses.len() as u32,
                                pipeline_index: renderpass.pipelines.len() as u32,
                                material_pass_key: 0,
                            };

                            self.create_materials_for_pipeline(
                                &pipeline,
                                &pipeline_metadata.material_metadatas,
                                &materials,
                                &pipeline_info.name,
                                &pipeline_interface,
                                &descriptor_pool,
                                &template_key,
                            );

                            renderpass.pipelines.push(pipeline);

                            renderpass_metadata
                                .pipeline_metadata
                                .insert(pipeline_info.name, pipeline_metadata);
                        }
                        Err(err) => {
                            error!(
                                "Could not create pipeline {} for pass {}: {}",
                                pipeline_info.name, pass_info.name, err
                            );

                            success = false;
                            continue;
                        }
                    }
                }
            }
        }

        success
    }

    fn create_graphics_pipeline(
        &self,
        interface: GraphicsApi::PipelineInterface,
        create_info: &PipelineCreationInfo,
    ) -> Result<(Pipeline<GraphicsApi>, PipelineMetadata), String> {
        let mut metadata = PipelineMetadata {
            data: create_info.clone(),
            material_metadatas: vec![],
        };

        // TODO: Create the material metadatas BEFORE MERGING

        self.device
            .create_graphics_pipeline(interface, create_info)
            .map_error(|e| format!("Could not create pipeline {}: {}", create_info.name, e))
            .map(|pipeline| (pipeline, metadata))
    }

    fn create_materials_for_pipeline(
        &self,
        pipeline: &Pipeline<GraphicsApi>,
        material_metadata: &HashMap<FullMaterialPassName, MaterialPassMetadata>,
        materials: &Vec<MaterialData>,
        pipeline_name: &str,
        pipeline_interface: &GraphicsApi::PipelineInterface,
        descriptor_pool: &GraphicsApi::DescriptorPool,
        template_key: &MaterialPassKey,
    ) -> bool {
        unimplemented!();
    }
}

impl<GraphicsApi> Renderer for ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    fn set_render_graph(&mut self, graph: ShaderpackData) {
        if !self.renderpasses.is_empty() {
            self.destroy_render_passes();
            self.destroy_rendergraph_resources();

            info!("Destroyed old render graph's resources");
        }

        self.create_rendergraph_resources(graph.resources);
        info!("Created render graph's textures");

        self.create_render_passes(graph.passes, graph.pipelines, graph.materials);
        info!("Loaded render graph");
    }

    fn add_mesh(&mut self, mesh_data: MeshData) -> u32 {
        unimplemented!()
    }

    fn tick(&self, delta_time: f32) {
        unimplemented!()
    }
}
