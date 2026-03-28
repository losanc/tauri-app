use wgpu::{Instance, Surface};

pub trait WgpuCompatibleSurface {
    fn create_surface(self, instance: &Instance) -> Surface<'static>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wgpu::Instance;
    use wgpu::Surface;
    use wgpu::SurfaceTarget;
    use wgpu::web_sys::HtmlCanvasElement;

    use crate::surface_helper::WgpuCompatibleSurface;
    impl WgpuCompatibleSurface for HtmlCanvasElement {
        fn create_surface(self, instance: &Instance) -> Surface<'static> {
            instance
                .create_surface(SurfaceTarget::Canvas(self))
                .expect("create surface failed")
        }
        fn width(&self) -> u32 {
            self.width()
        }
        fn height(&self) -> u32 {
            self.height()
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    use crate::surface_helper::WgpuCompatibleSurface;

    pub struct SurfaceHelper {
        surface: wgpu::SurfaceTargetUnsafe,
        width: u32,
        height: u32,
        position_x: u32,
        position_y: u32,
    }
    impl SurfaceHelper {
        pub fn new(
            window: &impl HasWindowHandle,
            width: u32,
            height: u32,
            position_x: u32,
            position_y: u32,
        ) -> Self {
            let handle = window.window_handle().unwrap();
            match handle.as_raw() {
                #[cfg(target_os = "macos")]
                RawWindowHandle::AppKit(app_kit_window_handle) => {
                    use objc2::runtime::AnyObject;
                    use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
                    use objc2_app_kit::NSView;
                    use objc2_foundation::{NSPoint, NSRect, NSSize};
                    use objc2_quartz_core::CAMetalLayer;
                    let ns_view = app_kit_window_handle.ns_view.as_ptr() as *mut AnyObject;
                    let ns_view = unsafe { &(*ns_view) };
                    if let Some(view) = ns_view.downcast_ref::<NSView>() {
                        let window = view.window().expect("failed to create window");
                        let context_view =
                            window.contentView().expect("failed to create content view");

                        let metal_rect = NSRect::new(
                            NSPoint::new(position_x as _, position_y as _),
                            NSSize::new(width as _, height as _),
                        );

                        let mtm = MainThreadMarker::new().expect("must be on the main thread");
                        let metal_view = NSView::initWithFrame(NSView::alloc(mtm), metal_rect);
                        metal_view.setWantsLayer(true);
                        let metal_layer = CAMetalLayer::new();
                        metal_view.setLayer(Some(&metal_layer));
                        context_view.addSubview(&metal_view);

                        let ptr = Retained::as_ptr(&metal_layer) as *mut std::ffi::c_void;

                        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(ptr);
                        Self {
                            surface: target,
                            width,
                            height,
                            position_x,
                            position_y,
                        }
                    } else {
                        panic!("where is my view?");
                    }
                }
                _ => {
                    unimplemented!()
                }
            }
        }
    }

    impl WgpuCompatibleSurface for SurfaceHelper {
        fn create_surface(self, instance: &wgpu::Instance) -> wgpu::Surface<'static> {
            unsafe {
                instance
                    .create_surface_unsafe(self.surface)
                    .expect("failed to create surface")
            }
        }

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }
}
