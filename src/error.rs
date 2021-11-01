/// Result type with error type `OblivionError` that is commonly returned from functions.
pub type OblivionResult<T> = Result<T, OblivionError>;

/// Errors that can occur in Oblivion.
#[derive(thiserror::Error, Debug)]
pub enum OblivionError {
    #[error("Unable to find a valid adapter card.")]
    RequestAdapter,
    #[error("Unable to create logical graphics device.")]
    CreateDevice(#[from] wgpu::RequestDeviceError),
    #[error("Unable to map GPU buffer.")]
    MapBuffer(#[from] wgpu::BufferAsyncError),
    #[error("Unable to load font.")]
    LoadFont(#[from] glyph_brush::ab_glyph::InvalidFont),
    #[error("Invalid Surface.")]
    InvalidSurface,
    #[error("Error occured while retrieving render frame.")]
    RetrieveFrameError(#[from] wgpu::SurfaceError),
    //#[error("Failed to tesselate shape.")]
    //TessellationError(#[from] lyon::tessellation::TessellationError),
}
