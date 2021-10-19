use pollster::block_on;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Limits, Queue, RequestAdapterOptions,
    Surface,
};

// TODO Result
pub fn get_adapter_surface(
    window: &impl raw_window_handle::HasRawWindowHandle,
) -> (Adapter, Surface) {
    let instance = wgpu::Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("Unable to find adapter");

    (adapter, surface)
}

// TODO Result
pub fn get_device_queue(adapter: &Adapter) -> (Device, Queue) {
    block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("Oblivion_Device"),
            features: Features::default(),
            limits: Limits::default(),
        },
        None,
    ))
    .expect("Unable to create device")
}
