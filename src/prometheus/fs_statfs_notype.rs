// is_real_proc returns true on architectures that don't have a Type argument
// in their statfs struct
fn is_real_proc(_mount_point: &str) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}