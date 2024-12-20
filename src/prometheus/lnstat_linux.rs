use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct LnstatCollector {
    logger: Logger,
}

impl LnstatCollector {
    fn new(logger: Logger) -> Result<Self, String> {
        Ok(LnstatCollector { logger })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), String> {
        let fs = procfs::ProcFs::new().map_err(|e| format!("failed to open procfs: {}", e))?;
        let net_stats = fs.net_stat().map_err(|e| format!("lnstat error: {}", e))?;

        for net_stat_file in net_stats {
            let label_names = vec!["subsystem".to_string(), "cpu".to_string()];
            for (header, stats) in net_stat_file.stats {
                for (cpu, value) in stats {
                    let label_values = vec![net_stat_file.filename.clone(), cpu.to_string()];
                    ch(Box::new(prometheus::Counter::new(
                        Desc::new(
                            format!("node_lnstat_{}_total", header),
                            "linux network cache stats".to_string(),
                            label_names.clone(),
                            HashMap::new(),
                        ),
                        value as f64,
                        label_values,
                    )));
                }
            }
        }
        Ok(())
    }
}

mod procfs {
    use std::fs;
    use std::path::Path;
    use std::collections::HashMap;

    pub struct ProcFs {
        proc: String,
    }

    impl ProcFs {
        pub fn new() -> Result<Self, std::io::Error> {
            Ok(ProcFs { proc: "/proc".to_string() })
        }

        pub fn net_stat(&self) -> Result<Vec<NetStatFile>, std::io::Error> {
            let path = Path::new(&self.proc).join("net/stat");
            let mut net_stat_files = Vec::new();

            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let filename = entry.file_name().into_string().unwrap();
                let content = fs::read_to_string(entry.path())?;
                let stats = parse_net_stat(&content)?;
                net_stat_files.push(NetStatFile { filename, stats });
            }

            Ok(net_stat_files)
        }
    }

    pub struct NetStatFile {
        pub filename: String,
        pub stats: HashMap<String, HashMap<usize, u64>>,
    }

    fn parse_net_stat(content: &str) -> Result<HashMap<String, HashMap<usize, u64>>, std::io::Error> {
        let mut stats = HashMap::new();
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 2 {
            return Ok(stats);
        }

        let headers: Vec<&str> = lines[0].split_whitespace().collect();
        for (cpu, line) in lines[1..].iter().enumerate() {
            let values: Vec<&str> = line.split_whitespace().collect();
            for (i, header) in headers.iter().enumerate() {
                let value = values[i].parse().unwrap_or(0);
                stats.entry(header.to_string()).or_insert_with(HashMap::new).insert(cpu, value);
            }
        }

        Ok(stats)
    }
}