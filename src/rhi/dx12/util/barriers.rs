use crate::rhi::dx12::util::enum_conversions::to_dx12_state;
use crate::rhi::ResourceBarrier;
use std::mem;
use winapi::um::d3d12::*;

pub fn to_dx12_barriers(barrier_data: &ResourceBarrier, resource: *mut ID3D12Resource) -> Vec<D3D12_RESOURCE_BARRIER> {
    vec![
        to_memory_barrier(resource),
        to_transition_barrier(barrier_data, resource),
        // TODO: Handle cross-queue sharing
    ]
}

fn to_memory_barrier(resource: *mut ID3D12Resource) -> D3D12_RESOURCE_BARRIER {
    let mut memory_barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        ..unsafe { mem::zeroed() }
    };

    *unsafe { memory_barrier.u.UAV_mut() } = D3D12_RESOURCE_UAV_BARRIER { pResource: resource };

    memory_barrier
}

fn to_transition_barrier(barrier_data: &ResourceBarrier, resource: *mut ID3D12Resource) -> D3D12_RESOURCE_BARRIER {
    let mut transition_barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        ..unsafe { mem::zeroed() }
    };

    *unsafe { transition_barrier.u.Transition_mut() } = D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: resource,
        Subresource: 0,
        StateBefore: to_dx12_state(&barrier_data.initial_state),
        StateAfter: to_dx12_state(&barrier_data.final_state),
    };

    transition_barrier
}
