use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;
use sysfs::SysFs;

const DRM_COLLECTOR_SUBSYSTEM: &str = "drm";

struct DrmCollector {
    fs: SysFs,
    logger: Logger,
    card_info: Desc,
    gpu_busy_percent: Desc,
    memory_gtt_size: Desc,
    memory_gtt_used: Desc,
    memory_visible_vram_size: Desc,
    memory_visible_vram_used: Desc,
    memory_vram_size: Desc,
    memory_vram_used: Desc,
}

impl DrmCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = SysFs::new("/sys")?;

        Ok(Self {
            fs,
            logger,
            card_info: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "card_info"),
                "Card information",
                vec!["card", "memory_vendor", "power_performance_level", "unique_id", "vendor"],
                None,
            )?,
            gpu_busy_percent: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "gpu_busy_percent"),
                "How busy the GPU is as a percentage.",
                vec!["card"],
                None,
            )?,
            memory_gtt_size: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "memory_gtt_size_bytes"),
                "The size of the graphics translation table (GTT) block in bytes.",
                vec!["card"],
                None,
            )?,
            memory_gtt_used: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "memory_gtt_used_bytes"),
                "The used amount of the graphics translation table (GTT) block in bytes.",
                vec!["card"],
                None,
            )?,
            memory_visible_vram_size: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "memory_vis_vram_size_bytes"),
                "The size of visible VRAM in bytes.",
                vec!["card"],
                None,
            )?,
            memory_visible_vram_used: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "memory_vis_vram_used_bytes"),
                "The used amount of visible VRAM in bytes.",
                vec!["card"],
                None,
            )?,
            memory_vram_size: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "memory_vram_size_bytes"),
                "The size of VRAM in bytes.",
                vec!["card"],
                None,
            )?,
            memory_vram_used: Desc::new(
                prometheus::core::build_fq_name("namespace", DRM_COLLECTOR_SUBSYSTEM, "memory_vram_used_bytes"),
                "The used amount of VRAM in bytes.",
                vec!["card"],
                None,
            )?,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        self.update_amd_cards(ch)
    }

    fn update_amd_cards(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let vendor = "amd";
        let stats = self.fs.class_drm_card_amdgpu_stats()?;

        for s in stats {
            ch(prometheus::core::MetricFamily::new(
                self.card_info.clone(),
                prometheus::proto::MetricType::GAUGE,
                1.0,
                vec![s.name, s.memory_vram_vendor, s.power_dpm_force_performance_level, s.unique_id, vendor.to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.gpu_busy_percent.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.gpu_busy_percent as f64,
                vec![s.name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.memory_gtt_size.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.memory_gtt_size as f64,
                vec![s.name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.memory_gtt_used.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.memory_gtt_used as f64,
                vec![s.name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.memory_vram_size.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.memory_vram_size as f64,
                vec![s.name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.memory_vram_used.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.memory_vram_used as f64,
                vec![s.name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.memory_visible_vram_size.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.memory_visible_vram_size as f64,
                vec![s.name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.memory_visible_vram_used.clone(),
                prometheus::proto::MetricType::GAUGE,
                s.memory_visible_vram_used as f64,
                vec![s.name.clone()],
            ));
        }

        Ok(())
    }
}

impl Collector for DrmCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.card_info);
        descs(&self.gpu_busy_percent);
        descs(&self.memory_gtt_size);
        descs(&self.memory_gtt_used);
        descs(&self.memory_visible_vram_size);
        descs(&self.memory_visible_vram_used);
        descs(&self.memory_vram_size);
        descs(&self.memory_vram_used);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect DRM metrics", o!("error" => e.to_string()));
        }
    }
}