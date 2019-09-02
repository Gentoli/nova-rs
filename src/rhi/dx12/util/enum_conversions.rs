use crate::rhi::{DescriptorType, QueueType, ResourceState};
use crate::shaderpack;
use winapi::um::d3d12::*;

pub fn to_dx12_range_type(descriptor_type: &DescriptorType) -> D3D12_DESCRIPTOR_RANGE_TYPE {
    match descriptor_type {
        DescriptorType::CombinedImageSampler => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        DescriptorType::UniformBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
        DescriptorType::StorageBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
    }
}

pub fn to_dx12_blend(blend_factor: &shaderpack::BlendFactor) -> D3D12_BLEND {
    match blend_factor {
        shaderpack::BlendFactor::One => D3D12_BLEND_ONE,
        shaderpack::BlendFactor::Zero => D3D12_BLEND_ZERO,
        shaderpack::BlendFactor::SrcColor => D3D12_BLEND_SRC_COLOR,
        shaderpack::BlendFactor::DstColor => D3D12_BLEND_DEST_COLOR,
        shaderpack::BlendFactor::OneMinusSrcColor => D3D12_BLEND_INV_SRC_COLOR,
        shaderpack::BlendFactor::OneMinusDstColor => D3D12_BLEND_INV_DEST_COLOR,
        shaderpack::BlendFactor::SrcAlpha => D3D12_BLEND_SRC_ALPHA,
        shaderpack::BlendFactor::DstAlpha => D3D12_BLEND_DEST_ALPHA,
        shaderpack::BlendFactor::OneMinusSrcAlpha => D3D12_BLEND_INV_SRC_ALPHA,
        shaderpack::BlendFactor::OneMinusDstAlpha => D3D12_BLEND_INV_DEST_ALPHA,
    }
}

pub fn to_dx12_compare_func(op: &shaderpack::CompareOp) -> D3D12_COMPARISON_FUNC {
    match op {
        shaderpack::CompareOp::Never => D3D12_COMPARISON_FUNC_NEVER,
        shaderpack::CompareOp::Less => D3D12_COMPARISON_FUNC_LESS,
        shaderpack::CompareOp::LessEqual => D3D12_COMPARISON_FUNC_LESS_EQUAL,
        shaderpack::CompareOp::Greater => D3D12_COMPARISON_FUNC_GREATER,
        shaderpack::CompareOp::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER_EQUAL,
        shaderpack::CompareOp::Equal => D3D12_COMPARISON_FUNC_EQUAL,
        shaderpack::CompareOp::NotEqual => D3D12_COMPARISON_FUNC_NOT_EQUAL,
        shaderpack::CompareOp::Always => D3D12_COMPARISON_FUNC_NEVER,
    }
}

pub fn to_dx12_stencil_op(op: &shaderpack::StencilOp) -> D3D12_STENCIL_OP {
    match op {
        shaderpack::StencilOp::Keep => D3D12_STENCIL_OP_KEEP,
        shaderpack::StencilOp::Zero => D3D12_STENCIL_OP_ZERO,
        shaderpack::StencilOp::Replace => D3D12_STENCIL_OP_REPLACE,
        shaderpack::StencilOp::Incr => D3D12_STENCIL_OP_INCR,
        shaderpack::StencilOp::IncrWrap => D3D12_STENCIL_OP_INCR_SAT,
        shaderpack::StencilOp::Decr => D3D12_STENCIL_OP_DECR,
        shaderpack::StencilOp::DecrWrap => D3D12_STENCIL_OP_DECR_SAT,
        shaderpack::StencilOp::Invert => D3D12_STENCIL_OP_INVERT,
    }
}

pub fn to_dx12_topology(topology: &shaderpack::PrimitiveTopology) -> D3D12_PRIMITIVE_TOPOLOGY_TYPE {
    match topology {
        shaderpack::PrimitiveTopology::Triangles => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        shaderpack::PrimitiveTopology::Lines => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
    }
}

pub fn to_dx12_state(state: &ResourceState) -> D3D12_RESOURCE_STATES {
    match state {
        ResourceState::Undefined => D3D12_RESOURCE_STATE_COMMON,
        ResourceState::General => D3D12_RESOURCE_STATE_COMMON,
        ResourceState::ColorAttachment => D3D12_RESOURCE_STATE_RENDER_TARGET,
        ResourceState::DepthStencilAttachment => D3D12_RESOURCE_STATE_DEPTH_WRITE,
        ResourceState::DepthReadOnlyStencilAttachment => D3D12_RESOURCE_STATE_DEPTH_WRITE,
        ResourceState::DepthAttachmentStencilReadOnly => D3D12_RESOURCE_STATE_DEPTH_WRITE,
        ResourceState::DepthStencilReadOnlyAttachment => D3D12_RESOURCE_STATE_DEPTH_READ,
        ResourceState::PresentSource => D3D12_RESOURCE_STATE_PRESENT,
        ResourceState::NonFragmentShaderReadOnly => D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
        ResourceState::FragmentShaderReadOnly => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
        ResourceState::TransferSource => D3D12_RESOURCE_STATE_COPY_SOURCE,
        ResourceState::TransferDestination => D3D12_RESOURCE_STATE_COPY_DEST,
    }
}

pub fn to_command_list_type(queue_type: &QueueType) -> D3D12_COMMAND_LIST_TYPE {
    match queue_type {
        QueueType::Graphics => D3D12_COMMAND_LIST_TYPE_DIRECT,
        QueueType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
        QueueType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
    }
}

pub fn to_dxgi_format(format: &shaderpack::PixelFormat) -> DXGI_FORMAT {
    match format {
        shaderpack::PixelFormat::RGBA8 => DXGI_FORMAT_R8G8B8A8_UNORM,
        shaderpack::PixelFormat::RGBA16F => DXGI_FORMAT_R16G16B16A16_FLOAT,
        shaderpack::PixelFormat::RGBA32F => DXGI_FORMAT_R32G32B32A32_FLOAT,
        shaderpack::PixelFormat::Depth => DXGI_FORMAT_D32_FLOAT,
        shaderpack::PixelFormat::DepthStencil => DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
    }
}
