use tauri::{Manager, RunEvent, WebviewWindow};
use wgpu::CurrentSurfaceTexture;

struct WgpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Greet {
    name: String,
    x: i32,
    y: i32,
}

#[tauri::command]
fn greet(input: Greet) -> String {
    format!("Hello, ! You've been greeted from Rust!")
}

/// Creates a CAMetalLayer on top of the webview's content view and returns a raw pointer to it.
/// The layer is retained by its parent NSView (ObjC ARC) for the lifetime of the window.
#[cfg(target_os = "macos")]
unsafe fn add_metal_overlay(window: &WebviewWindow) -> *mut std::ffi::c_void {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject, Bool};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    use wgpu::rwh::{HasWindowHandle, RawWindowHandle};

    // Tauri's WebviewWindow implements HasWindowHandle (rwh 0.6).
    // AppKitWindowHandle::ns_view is the WKWebView (or its container NSView).
    let handle = window.window_handle().unwrap();
    let ns_view: *mut AnyObject = match handle.as_raw() {
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as *mut AnyObject,
        _ => panic!("expected AppKit window handle on macOS"),
    };

    // Walk up to the NSWindow, then grab its contentView as the insertion point.
    let ns_window: *mut AnyObject = msg_send![ns_view, window];
    let content_view: *mut AnyObject = msg_send![ns_window, contentView];

    // Small fixed rect in the top-left corner — just enough to verify compositing.
    let overlay_rect = NSRect::new(NSPoint::new(20.0, 20.0), NSSize::new(200.0, 200.0));

    // Plain NSView that will host the Metal layer.
    let ns_view_class = AnyClass::get(c"NSView").unwrap();
    let overlay: *mut AnyObject = msg_send![ns_view_class, alloc];
    let overlay: *mut AnyObject = msg_send![overlay, initWithFrame: overlay_rect];

    // Enable layer-backing so setLayer: below takes effect.
    let _: () = msg_send![overlay, setWantsLayer: Bool::YES];

    // CAMetalLayer is the surface wgpu will render into.
    let layer_class = AnyClass::get(c"CAMetalLayer").unwrap();
    let layer: *mut AnyObject = msg_send![layer_class, new];
    let _: () = msg_send![overlay, setLayer: layer];

    // addSubview: appends to the end of the subview list → drawn on top.
    let _: () = msg_send![content_view, addSubview: overlay];

    layer as *mut std::ffi::c_void
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let instance = wgpu::Instance::default();

            // On macOS: render into a CAMetalLayer added on top of the webview.
            // On other platforms: render into the window surface directly.
            let surface: wgpu::Surface<'static> = unsafe {
                #[cfg(target_os = "macos")]
                let target = {
                    let layer_ptr = add_metal_overlay(&window);
                    wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr)
                };

                #[cfg(not(target_os = "macos"))]
                let target = wgpu::SurfaceTargetUnsafe::from_display_and_window(&window, &window)
                    .unwrap();

                instance.create_surface_unsafe(target).unwrap()
            };

            let adapter = pollster::block_on(instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                },
            ))
            .unwrap();

            println!("adapter: {}", adapter.get_info().name);

            let (device, queue) =
                pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                    .unwrap();

            let caps = surface.get_capabilities(&adapter);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: caps.formats[0],
                width: 200,
                height: 200,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 1,
            };

            surface.configure(&device, &config);
            app.manage(WgpuState {
                surface,
                device,
                queue,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            RunEvent::MainEventsCleared => {
                let state = app_handle.state::<WgpuState>();
                let frame = state.surface.get_current_texture();
                if let CurrentSurfaceTexture::Success(texture) = frame {
                    let view = texture.texture.create_view(&Default::default());
                    let mut encoder = state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                    {
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                    }
                    state.queue.submit(Some(encoder.finish()));
                    texture.present();
                }
            }
            _ => {}
        });
}
