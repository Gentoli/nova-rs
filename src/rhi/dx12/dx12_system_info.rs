pub enum Release {
    One,   // SDK version 10.00.10240.16384
    Two,   // SDK version 10.00.15063.0000, added depth bounds testing
    Three, // SDK version 10.00.17763.0001, added DXR and renderpasses
    Four,  // SDK version 10.00.18362.0116, added variable rate shading
}

pub struct Dx12SystemInfo {
    /// The release of DX12 that the system we're running on supports
    supported_version: Release,
}
