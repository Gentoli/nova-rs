use crate::mesh::MeshData;
use crate::renderer::rendergraph::Renderpass;
use crate::renderer::Renderer;
use crate::rhi;
use crate::settings::Settings;
use crate::shaderpack::{
    MaterialData, PipelineCreationInfo, RenderPassCreationInfo, ShaderpackData, ShaderpackResourceData,
    TextureCreateInfo,
};
use cgmath::Vector2;
use std::collections::HashMap;

#[macro_use]
extern crate log;

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

    swapchain: GraphicsApi::Swapchain,
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

    fn create_rendergraph_resources(&mut self, resource_info: &ShaderpackResourceData) {
        self.renderpass_textures = resource_info
            .textures
            .iter()
            .map(|info| (info.name, self.device.create_image(info)))
            .collect();

        self.renderpass_texture_infos = resource_info.textures.iter().map(|info| (info.name, info)).collect();

        // TODO: Create rendergraph buffers
    }

    fn create_rendergraph_textures(&mut self, &texture_infos: &Vec<TextureCreateInfo>) {}

    fn create_render_passes(
        &mut self,
        passes: &Vec<RenderPassCreationInfo>,
        pipelines: &Vec<PipelineCreationInfo>,
        materials: &Vec<MaterialData>,
    ) {
        let mut total_num_descriptors = 0;
        for material_data in materials {
            for material_pass in material_data.passes {
                total_num_descriptors += material_pass.bindings.len();
            }
        }

        let descriptor_pool = self
            .device
            .create_descriptor_pool(total_num_descriptors, 0, total_num_descriptors);

        for pass_info in passes {
            let rhi_renderpass_result = self.device.create_renderpass(pass_info);
            if let Ok(rhi_renderpass) = rhi_renderpass_result {
                let mut renderpass: Renderpass<GraphicsApi> = Default::default();
                renderpass.renderpass = rhi_renderpass;

                let mut output_images = Vec::<GraphicsApi::Image>::with_capacity(pass_info.texture_outputs.len());
                let mut attachment_errors = Vec::<String>::with_capacity(pass_info.texture_outputs.len());
                let mut framebuffer_size = Vector2::<f32>::new(0, 0);

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
                        output_images.push(image);

                        let image_info = self.renderpass_texture_infos.get(&attachment_info.name).unwrap();
                        let attachment_size = image_info.format.get_size_in_pixels(self.swapchain.get_size());
                    }
                }
            } else if let Err(err) = rhi_renderpass_result {
                error!("Could not create renderpass {}: {}", pass_info.name, err);
            }
        }
    }
}

impl<GraphicsApi> Renderer for ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    fn set_render_graph(&mut self, graph: &ShaderpackData) {
        if !self.renderpasses.is_empty() {
            self.destroy_render_passes();
            self.destroy_rendergraph_resources();

            info!("Destroyed old render graph's resources");
        }

        self.create_rendergraph_resources(&graph.resources);
        info!("Created render graph's textures");

        self.create_render_passes(&graph.passes, &graph.pipelines, &graph.materials);
        info!("Loaded render graph");
    }

    fn add_mesh(&mut self, mesh_data: &MeshData) -> u32 {
        unimplemented!()
    }

    fn tick(&self, delta_time: f32) {
        unimplemented!()
    }
}
