use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct MdadmCollector {
    logger: Logger,
}

impl MdadmCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MdadmCollector { logger })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn std::error::Error>> {
        let fs = procfs::ProcFs::new().map_err(|e| format!("failed to open procfs: {}", e))?;
        let md_stats = fs.md_stat().map_err(|e| format!("error parsing mdstatus: {}", e))?;

        for md_stat in md_stats {
            self.logger.debug("collecting metrics for device", &["device", &md_stat.name]);

            let state_vals = HashMap::from([
                (md_stat.activity_state.clone(), 1.0),
            ]);

            ch(Box::new(prometheus::Gauge::new(
                disks_total_desc(),
                md_stat.disks_total as f64,
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                disks_desc("active"),
                md_stat.disks_active as f64,
                vec![md_stat.name.clone(), "active".to_string()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                disks_desc("failed"),
                md_stat.disks_failed as f64,
                vec![md_stat.name.clone(), "failed".to_string()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                disks_desc("spare"),
                md_stat.disks_spare as f64,
                vec![md_stat.name.clone(), "spare".to_string()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                active_desc(),
                *state_vals.get("active").unwrap_or(&0.0),
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                inactive_desc(),
                *state_vals.get("inactive").unwrap_or(&0.0),
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                recovering_desc(),
                *state_vals.get("recovering").unwrap_or(&0.0),
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                resync_desc(),
                *state_vals.get("resyncing").unwrap_or(&0.0),
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                check_desc(),
                *state_vals.get("checking").unwrap_or(&0.0),
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                blocks_total_desc(),
                md_stat.blocks_total as f64,
                vec![md_stat.name.clone()],
            )));
            ch(Box::new(prometheus::Gauge::new(
                blocks_synced_desc(),
                md_stat.blocks_synced as f64,
                vec![md_stat.name.clone()],
            )));
        }

        Ok(())
    }
}

fn active_desc() -> Desc {
    Desc::new(
        "node_md_state",
        "Indicates the state of md-device.",
        vec!["device".to_string()],
        HashMap::from([("state".to_string(), "active".to_string())]),
    )
}

fn inactive_desc() -> Desc {
    Desc::new(
        "node_md_state",
        "Indicates the state of md-device.",
        vec!["device".to_string()],
        HashMap::from([("state".to_string(), "inactive".to_string())]),
    )
}

fn recovering_desc() -> Desc {
    Desc::new(
        "node_md_state",
        "Indicates the state of md-device.",
        vec!["device".to_string()],
        HashMap::from([("state".to_string(), "recovering".to_string())]),
    )
}

fn resync_desc() -> Desc {
    Desc::new(
        "node_md_state",
        "Indicates the state of md-device.",
        vec!["device".to_string()],
        HashMap::from([("state".to_string(), "resync".to_string())]),
    )
}

fn check_desc() -> Desc {
    Desc::new(
        "node_md_state",
        "Indicates the state of md-device.",
        vec!["device".to_string()],
        HashMap::from([("state".to_string(), "check".to_string())]),
    )
}

fn disks_desc(state: &str) -> Desc {
    Desc::new(
        "node_md_disks",
        "Number of active/failed/spare disks of device.",
        vec!["device".to_string(), "state".to_string()],
        HashMap::new(),
    )
}

fn disks_total_desc() -> Desc {
    Desc::new(
        "node_md_disks_required",
        "Total number of disks of device.",
        vec!["device".to_string()],
        HashMap::new(),
    )
}

fn blocks_total_desc() -> Desc {
    Desc::new(
        "node_md_blocks",
        "Total number of blocks on device.",
        vec!["device".to_string()],
        HashMap::new(),
    )
}

fn blocks_synced_desc() -> Desc {
    Desc::new(
        "node_md_blocks_synced",
        "Number of blocks synced on device.",
        vec!["device".to_string()],
        HashMap::new(),
    )
}