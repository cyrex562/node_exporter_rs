use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

const CLOCKS_PER_SEC: f64 = libc::sysconf(libc::_SC_CLK_TCK) as f64;

struct StatCollector {
    cpu: Desc,
    logger: Logger,
}

impl StatCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cpu: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "seconds_total"),
                "Seconds the CPUs spent in each mode.",
                vec!["cpu".to_string(), "mode".to_string()],
                None,
            )?,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let mut count: libc::c_uint = 0;
        let mut cpuload: *mut libc::processor_cpu_load_info_t = ptr::null_mut();
        let mut ncpu: libc::natural_t = 0;

        let status = unsafe {
            libc::host_processor_info(
                libc::mach_host_self(),
                libc::PROCESSOR_CPU_LOAD_INFO,
                &mut ncpu,
                &mut cpuload as *mut _ as *mut _,
                &mut count,
            )
        };

        if status != libc::KERN_SUCCESS {
            return Err(format!("host_processor_info error={}", status).into());
        }

        let target = unsafe { libc::mach_task_self() };
        let address = cpuload as libc::vm_address_t;
        let size = (ncpu as usize) * std::mem::size_of::<libc::processor_cpu_load_info_data_t>();
        let buf = unsafe { std::slice::from_raw_parts(cpuload as *const u8, size) };

        let mut cpu_ticks = [0u32; libc::CPU_STATE_MAX as usize];
        let mut bbuf = std::io::Cursor::new(buf);

        for i in 0..ncpu as usize {
            bbuf.read_exact(bytemuck::cast_slice_mut(&mut cpu_ticks))?;
            for (k, v) in [("user", libc::CPU_STATE_USER), ("system", libc::CPU_STATE_SYSTEM), ("nice", libc::CPU_STATE_NICE), ("idle", libc::CPU_STATE_IDLE)].iter() {
                ch(prometheus::core::MetricFamily::new(
                    self.cpu.clone(),
                    prometheus::proto::MetricType::COUNTER,
                    cpu_ticks[*v as usize] as f64 / CLOCKS_PER_SEC,
                    vec![i.to_string(), k.to_string()],
                ));
            }
        }

        unsafe {
            libc::vm_deallocate(target, address, size as libc::vm_size_t);
        }

        Ok(())
    }
}

impl Collector for StatCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.cpu);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect CPU metrics", o!("error" => e.to_string()));
        }
    }
}