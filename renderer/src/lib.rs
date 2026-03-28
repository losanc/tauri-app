#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;
use alloc::vec;
mod log;
pub mod surface_helper;
use wgpu::{Adapter, Device, Instance, Queue, Surface};

use crate::surface_helper::WgpuCompatibleSurface;

pub struct GpuContext {
    instance: Instance,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl GpuContext {
    pub async fn init_wgpu(user_surface: impl WgpuCompatibleSurface) -> GpuContext {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("create adapter failed");
        my_print!("device name: {}", adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("create device failed");
        let width = user_surface.width();
        let height = user_surface.height();
        let surface = user_surface.create_surface(&instance);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let caps = surface.get_capabilities(&adapter);

        my_print!("{:?}", caps);

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }

    pub async fn render(&self, color: i32) {
        let frame = self.surface.get_current_texture();
        match frame {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => {
                let view = surface_texture.texture.create_view(&Default::default());

                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(match color {
                                    0 => wgpu::Color::RED,
                                    _ => wgpu::Color::GREEN,
                                }),
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

                self.queue.submit(Some(encoder.finish()));

                surface_texture.present();
            }
            _ => {}
        }
    }
}
