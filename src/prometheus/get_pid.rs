#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
fn get_pid_fn() -> impl Fn() -> Result<i32, Box<dyn std::error::Error>> {
    let pid = std::process::id() as i32;
    move || Ok(pid)
}