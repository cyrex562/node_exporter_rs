use prometheus::{self, core::{Collector, Desc, Metric, Opts}};
use slog::Logger;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::sync::Arc;

const FILE_FD_STAT_SUBSYSTEM: &str = "filefd";

struct FileFDStatCollector {
    logger: Arc<Logger>,
}

impl FileFDStatCollector {
    fn new(logger: Arc<Logger>) -> Self {
        FileFDStatCollector { logger }
    }

    fn parse_file_fd_stats(filename: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let parts: Vec<&str> = content.trim().split('\t').collect();
        if parts.len() < 3 {
            return Err(format!("unexpected number of file stats in {}", filename).into());
        }

        let mut file_fd_stat = HashMap::new();
        file_fd_stat.insert("allocated".to_string(), parts[0].to_string());
        file_fd_stat.insert("maximum".to_string(), parts[2].to_string());

        Ok(file_fd_stat)
    }
}

impl Collector for FileFDStatCollector {
    fn describe(&self, descs: &mut Vec<&Desc>) {
        descs.push(&Desc::new(
            Opts::new("filefd_stats", "File descriptor statistics")
                .namespace(namespace)
                .subsystem(FILE_FD_STAT_SUBSYSTEM),
        ));
    }

    fn collect(&self, mfs: &mut Vec<MetricFamily>) {
        match Self::parse_file_fd_stats("/proc/sys/fs/file-nr") {
            Ok(file_fd_stat) => {
                for (name, value) in file_fd_stat {
                    if let Ok(v) = value.parse::<f64>() {
                        let desc = Desc::new(
                            Opts::new(&name, &format!("File descriptor statistics: {}.", name))
                                .namespace(namespace)
                                .subsystem(FILE_FD_STAT_SUBSYSTEM),
                        );
                        mfs.push(prometheus::new_const_metric(&desc, prometheus::proto::MetricType::GAUGE, v, &[]));
                    } else {
                        self.logger.error("invalid value in file-nr", o!("value" => value));
                    }
                }
            }
            Err(err) => {
                self.logger.error("couldn't get file-nr", o!("error" => err.to_string()));
            }
        }
    }
}