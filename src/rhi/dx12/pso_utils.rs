//! A handful of functions to make filling out PSO create info structures less painful

#![allow(unsafe_code)]

use crate::rhi::dx12::util::enum_conversions::{to_dx12_blend, to_dx12_compare_func, to_dx12_stencil_op};
use crate::shaderpack;
use std::mem;
use winapi::shared::dxgiformat::*;
use winapi::um::d3d12::*;

pub fn make_render_target_blend_desc(data: &shaderpack::PipelineCreationInfo) -> D3D12_RENDER_TARGET_BLEND_DESC {
    D3D12_RENDER_TARGET_BLEND_DESC {
        BlendEnable: data.states.contains(&shaderpack::RasterizerState::Blending) as i32,
        LogicOpEnable: 0,
        SrcBlend: to_dx12_blend(&data.src_blend_factor),
        DestBlend: to_dx12_blend(&data.dst_blend_factor),
        BlendOp: D3D12_BLEND_OP_ADD,
        SrcBlendAlpha: to_dx12_blend(&data.src_blend_factor),
        DestBlendAlpha: to_dx12_blend(&data.dst_blend_factor),
        BlendOpAlpha: D3D12_BLEND_OP_ADD,
        LogicOp: 0,
        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL as u8,
    }
}

pub fn make_rasterizer_desc(data: &shaderpack::PipelineCreationInfo) -> D3D12_RASTERIZER_DESC {
    let get_cull_mode = |states: &Vec<shaderpack::RasterizerState>| {
        if states.contains(&shaderpack::RasterizerState::InvertCulling) {
            D3D12_CULL_MODE_FRONT
        } else if states.contains(&shaderpack::RasterizerState::DisableCulling) {
            D3D12_CULL_MODE_NONE
        } else {
            D3D12_CULL_MODE_FRONT
        }
    };

    D3D12_RASTERIZER_DESC {
        FillMode: D3D12_FILL_MODE_SOLID,
        CullMode: get_cull_mode(&data.states),
        FrontCounterClockwise: true as i32,
        DepthBias: data.depth_bias.round() as i32,
        DepthBiasClamp: 0.0,
        SlopeScaledDepthBias: data.slope_scaled_depth_bias,
        DepthClipEnable: true as i32,
        // TODO: Handle MSAA at all stages of Nova
        MultisampleEnable: (data.msaa_support != shaderpack::MSAASupport::None) as i32,
        AntialiasedLineEnable: 0,
        ForcedSampleCount: 0,
        ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_ON,
    }
}

pub fn make_depth_stencil_state(data: &shaderpack::PipelineCreationInfo) -> D3D12_DEPTH_STENCIL_DESC {
    let get_depth_write_mask = |states: &Vec<shaderpack::RasterizerState>| {
        if states.contains(&shaderpack::RasterizerState::DisableDepthWrite) {
            D3D12_DEPTH_WRITE_MASK_ZERO
        } else {
            D3D12_DEPTH_WRITE_MASK_ZERO
        }
    };

    D3D12_DEPTH_STENCIL_DESC {
        DepthEnable: data.states.contains(&shaderpack::RasterizerState::DisableDepthTest) as i32,
        DepthWriteMask: get_depth_write_mask(&data.states),
        DepthFunc: to_dx12_compare_func(&data.depth_func),
        StencilEnable: data.states.contains(&shaderpack::RasterizerState::EnableStencilTest) as i32,
        StencilReadMask: data.stencil_read_mask as u8,
        StencilWriteMask: data.stencil_write_mask as u8,
        FrontFace: match &data.front_face {
            Some(front_face) => D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: to_dx12_stencil_op(&front_face.fail_op),
                StencilDepthFailOp: to_dx12_stencil_op(&front_face.depth_fail_op),
                StencilPassOp: to_dx12_stencil_op(&front_face.pass_op),
                StencilFunc: to_dx12_compare_func(&front_face.compare_op),
            },
            None => D3D12_DEPTH_STENCILOP_DESC {
                ..unsafe { mem::zeroed() }
            },
        },
        BackFace: match &data.back_face {
            Some(back_face) => D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: to_dx12_stencil_op(&back_face.fail_op),
                StencilDepthFailOp: to_dx12_stencil_op(&back_face.depth_fail_op),
                StencilPassOp: to_dx12_stencil_op(&back_face.pass_op),
                StencilFunc: to_dx12_compare_func(&back_face.compare_op),
            },
            None => D3D12_DEPTH_STENCILOP_DESC {
                ..unsafe { mem::zeroed() }
            },
        },
    }
}

pub fn get_input_descriptions() -> Vec<D3D12_INPUT_ELEMENT_DESC> {
    vec![
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "POSITION\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "NORMAL\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 1,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "TANGENT\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 2,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "TEXCOORD\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 3,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "LMUV\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 4,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "VTEX_ID\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32_UINT,
            InputSlot: 5,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: "DATA\0".as_ptr() as *const _,
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
            InputSlot: 6,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
    ]
}
