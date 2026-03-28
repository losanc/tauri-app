use renderer::{GpuContext, surface_helper::native::SurfaceHelper};
use tauri::{Manager, RunEvent, WebviewWindow};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Greet {
    name: String,
    x: i32,
    y: i32,
}

#[tauri::command]
fn greet(input: Greet) -> String {
    format!("Hello, {} ! You've been greeted from Rust!", input.name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let tauri_surface = SurfaceHelper::new(&window, 200, 200, 20, 20);
            let gpu_context = pollster::block_on(GpuContext::init_wgpu(tauri_surface));
            app.manage(gpu_context);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            RunEvent::MainEventsCleared => {
                let gpu_context = app_handle.state::<GpuContext>();
                pollster::block_on(gpu_context.render(0));
            }
            _ => {}
        });
}
