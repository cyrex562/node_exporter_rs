use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::sleep;
use std::time::Duration;

fn atomic_update_float(bits: &AtomicU64, update_func: impl Fn(f64) -> f64) {
    const MAX_BACKOFF: Duration = Duration::from_millis(320);
    const INITIAL_BACKOFF: Duration = Duration::from_millis(10);
    let mut backoff = INITIAL_BACKOFF;

    loop {
        let loaded_bits = bits.load(Ordering::Relaxed);
        let old_float = f64::from_bits(loaded_bits);
        let new_float = update_func(old_float);
        let new_bits = new_float.to_bits();

        if bits.compare_and_swap(loaded_bits, new_bits, Ordering::SeqCst) == loaded_bits {
            break;
        } else {
            sleep(backoff);
            backoff = (backoff * 2).min(MAX_BACKOFF);
        }
    }
}