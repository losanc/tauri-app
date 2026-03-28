use leptos::html::Canvas;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use renderer::GpuContext;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
    let update_name = move |ev| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };
    let canvas_ref = NodeRef::<Canvas>::new();

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        let canvas = canvas_ref.get().unwrap();

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
            let width = canvas.width();
            let height = canvas.height();
            let gpu_context = GpuContext::init_wgpu(canvas).await;

            gpu_context.render(2).await;
        });
    };

    view! {

        <main class="container">
            <h1>"Welcome to Tauri + Leptos"</h1>

            <form class="row" on:submit=greet>
                <input
                    id="greet-input"
                    placeholder="Enter a name..."
                    on:input=update_name
                />
                <button type="submit">"Greet"</button>
            </form>
            <p>{ move || greet_msg.get() }</p>

            <canvas width="400" height="400" style="border: 1px solid black"
            node_ref=canvas_ref/>

        </main>
    }
}
