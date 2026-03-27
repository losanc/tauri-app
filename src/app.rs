use leptos::html::Canvas;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use std::io::Read;
use wasm_bindgen::prelude::*;
use wgpu::{CurrentSurfaceTexture, SurfaceTarget};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct Greet {
    name: String,
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize)]
struct Args {
    input: Greet,
}

#[component]
pub fn HelloView() -> impl IntoView {
    view! {
        <h1>"Welcome to the homepage"</h1>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());

    // leptos::logging::log!("!!!!!!!!!! {contents}");

    let update_name = move |ev| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };
    let canvas_ref = NodeRef::<Canvas>::new();

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();

        let canvas2 = canvas_ref.get().unwrap();

        let instance = wgpu::Instance::default();
        let width = canvas2.width();
        let height = canvas2.height();
        let canvas = SurfaceTarget::Canvas(canvas2);

        let surface = instance.create_surface(canvas).unwrap();

        {
            spawn_local(async move {
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions::default())
                    .await
                    .unwrap();
                leptos::logging::log!("!!!!!!!!!! {}", adapter.get_info().name);

                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor::default())
                    .await
                    .unwrap();
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: surface.get_capabilities(&adapter).formats[0],
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 1,
                };
                surface.configure(&device, &config);

                let frame = surface.get_current_texture();
                if let CurrentSurfaceTexture::Success(texture) = frame {
                    let view = texture.texture.create_view(&Default::default());

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                    {
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
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

                    queue.submit(Some(encoder.finish()));

                    texture.present();
                }
            });
        }

        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                return;
                // name = "srgmkthmtk".to_string();
            }
            let usergreet = Greet {
                name: name.clone(),
                x: 2235430,
                y: 3224,
            };
            let user = serde_wasm_bindgen::to_value(&Args { input: usergreet }).unwrap();
            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

            let new_msg = invoke("greet", user).await.as_string().unwrap();

            set_greet_msg.set(new_msg);
        });
    };

    view! {

            <main class="container">
                <h1>"Welcome to Tauri + Leptos"</h1>

                <div class="row">
                    <a href="https://tauri.app" target="_blank">
                        <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                    </a>
                    <a href="https://docs.rs/leptos/" target="_blank">
                        <img src="public/leptos.svg" class="logo leptos" alt="Leptos logo"/>
                    </a>
                </div>
                <p>"Click on the Tauri and Leptos logos to learn more."</p>

                <form class="row" on:submit=greet>
                    <input
                        id="greet-input"
                        placeholder="Enter a name..."
                        on:input=update_name
                    />
                    <button type="submit">"Greet"</button>
                </form>
                <p>{ move || greet_msg.get() }</p>


    <canvas
                width="400"
                height="400"
                style="border: 1px solid black"
                node_ref=canvas_ref
            />

            </main>
        }
}
