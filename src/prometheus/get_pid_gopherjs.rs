#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
fn get_pid_fn() -> impl Fn() -> Result<i32, Box<dyn std::error::Error>> {
    || Ok(1)
}