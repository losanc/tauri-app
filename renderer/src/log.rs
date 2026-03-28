#[macro_export]
macro_rules! my_print{
    ($($arg:tt)*) => {

        #[cfg(not(target_arch = "wasm32"))]
        println!($($arg)*);

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&alloc::format!($($arg)*).into());
    };
}
