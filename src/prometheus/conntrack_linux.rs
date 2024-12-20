use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::fs;
use std::io::{self, ErrorKind};

struct ConntrackCollector {
    current: Desc,
    limit: Desc,
    found: Desc,
    invalid: Desc,
    ignore: Desc,
    insert: Desc,
    insert_failed: Desc,
    drop: Desc,
    early_drop: Desc,
    search_restart: Desc,
    logger: Logger,
}

struct ConntrackStatistics {
    found: u64,
    invalid: u64,
    ignore: u64,
    insert: u64,
    insert_failed: u64,
    drop: u64,
    early_drop: u64,
    search_restart: u64,
}

impl ConntrackCollector {
    fn new(logger: Logger) -> Self {
        Self {
            current: Desc::new(
                "nf_conntrack_entries".to_string(),
                "Number of currently allocated flow entries for connection tracking.".to_string(),
                vec![],
                None,
            ).unwrap(),
            limit: Desc::new(
                "nf_conntrack_entries_limit".to_string(),
                "Maximum size of connection tracking table.".to_string(),
                vec![],
                None,
            ).unwrap(),
            found: Desc::new(
                "nf_conntrack_stat_found".to_string(),
                "Number of searched entries which were successful.".to_string(),
                vec![],
                None,
            ).unwrap(),
            invalid: Desc::new(
                "nf_conntrack_stat_invalid".to_string(),
                "Number of packets seen which can not be tracked.".to_string(),
                vec![],
                None,
            ).unwrap(),
            ignore: Desc::new(
                "nf_conntrack_stat_ignore".to_string(),
                "Number of packets seen which are already connected to a conntrack entry.".to_string(),
                vec![],
                None,
            ).unwrap(),
            insert: Desc::new(
                "nf_conntrack_stat_insert".to_string(),
                "Number of entries inserted into the list.".to_string(),
                vec![],
                None,
            ).unwrap(),
            insert_failed: Desc::new(
                "nf_conntrack_stat_insert_failed".to_string(),
                "Number of entries for which list insertion was attempted but failed.".to_string(),
                vec![],
                None,
            ).unwrap(),
            drop: Desc::new(
                "nf_conntrack_stat_drop".to_string(),
                "Number of packets dropped due to conntrack failure.".to_string(),
                vec![],
                None,
            ).unwrap(),
            early_drop: Desc::new(
                "nf_conntrack_stat_early_drop".to_string(),
                "Number of dropped conntrack entries to make room for new ones, if maximum table size was reached.".to_string(),
                vec![],
                None,
            ).unwrap(),
            search_restart: Desc::new(
                "nf_conntrack_stat_search_restart".to_string(),
                "Number of conntrack table lookups which had to be restarted due to hashtable resizes.".to_string(),
                vec![],
                None,
            ).unwrap(),
            logger,
        }
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let value = read_uint_from_file("/proc/sys/net/netfilter/nf_conntrack_count")?;
        ch(prometheus::core::MetricFamily::new(
            self.current.clone(),
            prometheus::proto::MetricType::GAUGE,
            value as f64,
            vec![],
        ));

        let value = read_uint_from_file("/proc/sys/net/netfilter/nf_conntrack_max")?;
        ch(prometheus::core::MetricFamily::new(
            self.limit.clone(),
            prometheus::proto::MetricType::GAUGE,
            value as f64,
            vec![],
        ));

        let stats = get_conntrack_statistics()?;
        ch(prometheus::core::MetricFamily::new(
            self.found.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.found as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.invalid.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.invalid as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.ignore.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.ignore as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.insert.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.insert as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.insert_failed.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.insert_failed as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.drop.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.drop as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.early_drop.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.early_drop as f64,
            vec![],
        ));
        ch(prometheus::core::MetricFamily::new(
            self.search_restart.clone(),
            prometheus::proto::MetricType::GAUGE,
            stats.search_restart as f64,
            vec![],
        ));
        Ok(())
    }

    fn handle_err(&self, err: io::Error) -> io::Error {
        if err.kind() == ErrorKind::NotFound {
            self.logger.debug("conntrack probably not loaded");
            return io::Error::new(ErrorKind::Other, "no data");
        }
        io::Error::new(ErrorKind::Other, format!("failed to retrieve conntrack stats: {}", err))
    }
}

fn read_uint_from_file(path: &str) -> Result<u64, io::Error> {
    let content = fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}

fn get_conntrack_statistics() -> Result<ConntrackStatistics, Box<dyn std::error::Error>> {
    let mut stats = ConntrackStatistics {
        found: 0,
        invalid: 0,
        ignore: 0,
        insert: 0,
        insert_failed: 0,
        drop: 0,
        early_drop: 0,
        search_restart: 0,
    };

    let fs = procfs::ProcFs::new()?;
    let conn_stats = fs.conntrack_stat()?;

    for stat in conn_stats {
        stats.found += stat.found;
        stats.invalid += stat.invalid;
        stats.ignore += stat.ignore;
        stats.insert += stat.insert;
        stats.insert_failed += stat.insert_failed;
        stats.drop += stat.drop;
        stats.early_drop += stat.early_drop;
        stats.search_restart += stat.search_restart;
    }

    Ok(stats)
}