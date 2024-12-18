use prometheus::{Collector, Counter, Desc, Gauge, Opts, proto};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Placeholder for the database connection type
struct DbConnection;

impl DbConnection {
    fn stats(&self) -> DbStats {
        // Implement this method to retrieve database statistics
        DbStats {
            max_open_connections: 100,
            open_connections: 50,
            in_use_connections: 30,
            idle_connections: 20,
            wait_count: 200,
            wait_duration: Duration::from_secs(60),
            max_idle_closed: 10,
            max_idle_time_closed: 5,
            max_lifetime_closed: 2,
        }
    }
}

struct DbStats {
    max_open_connections: u32,
    open_connections: u32,
    in_use_connections: u32,
    idle_connections: u32,
    wait_count: u64,
    wait_duration: Duration,
    max_idle_closed: u64,
    max_idle_time_closed: u64,
    max_lifetime_closed: u64,
}

pub struct DbStatsCollector {
    db: Arc<DbConnection>,
    max_open_connections: Gauge,
    open_connections: Gauge,
    in_use_connections: Gauge,
    idle_connections: Gauge,
    wait_count: Counter,
    wait_duration: Counter,
    max_idle_closed: Counter,
    max_idle_time_closed: Counter,
    max_lifetime_closed: Counter,
}

impl DbStatsCollector {
    pub fn new(db: Arc<DbConnection>, db_name: &str) -> Self {
        let fq_name = |name: &str| format!("rust_sql_{}", name);
        let mut labels = HashMap::new();
        labels.insert("db_name".to_string(), db_name.to_string());

        DbStatsCollector {
            db,
            max_open_connections: Gauge::with_opts(
                Opts::new(fq_name("max_open_connections"), "Maximum number of open connections to the database.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            open_connections: Gauge::with_opts(
                Opts::new(fq_name("open_connections"), "The number of established connections both in use and idle.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            in_use_connections: Gauge::with_opts(
                Opts::new(fq_name("in_use_connections"), "The number of connections currently in use.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            idle_connections: Gauge::with_opts(
                Opts::new(fq_name("idle_connections"), "The number of idle connections.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            wait_count: Counter::with_opts(
                Opts::new(fq_name("wait_count_total"), "The total number of connections waited for.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            wait_duration: Counter::with_opts(
                Opts::new(fq_name("wait_duration_seconds_total"), "The total time blocked waiting for a new connection.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            max_idle_closed: Counter::with_opts(
                Opts::new(fq_name("max_idle_closed_total"), "The total number of connections closed due to SetMaxIdleConns.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            max_idle_time_closed: Counter::with_opts(
                Opts::new(fq_name("max_idle_time_closed_total"), "The total number of connections closed due to SetConnMaxIdleTime.")
                    .const_labels(labels.clone()),
            )
            .unwrap(),
            max_lifetime_closed: Counter::with_opts(
                Opts::new(
                    fq_name("max_lifetime_closed_total"),
                    "The total number of connections closed due to SetConnMaxLifetime.",
                )
                .const_labels(labels),
            )
            .unwrap(),
        }
    }
}

impl Collector for DbStatsCollector {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            self.max_open_connections.desc(),
            self.open_connections.desc(),
            self.in_use_connections.desc(),
            self.idle_connections.desc(),
            self.wait_count.desc(),
            self.wait_duration.desc(),
            self.max_idle_closed.desc(),
            self.max_idle_time_closed.desc(),
            self.max_lifetime_closed.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let stats = self.db.stats();

        self.max_open_connections.set(stats.max_open_connections as f64);
        self.open_connections.set(stats.open_connections as f64);
        self.in_use_connections.set(stats.in_use_connections as f64);
        self.idle_connections.set(stats.idle_connections as f64);
        self.wait_count.inc_by(stats.wait_count as f64);
        self.wait_duration.inc_by(stats.wait_duration.as_secs_f64());
        self.max_idle_closed.inc_by(stats.max_idle_closed as f64);
        self.max_idle_time_closed.inc_by(stats.max_idle_time_closed as f64);
        self.max_lifetime_closed.inc_by(stats.max_lifetime_closed as f64);

        let mut mfs = Vec::new();
        mfs.extend(self.max_open_connections.collect());
        mfs.extend(self.open_connections.collect());
        mfs.extend(self.in_use_connections.collect());
        mfs.extend(self.idle_connections.collect());
        mfs.extend(self.wait_count.collect());
        mfs.extend(self.wait_duration.collect());
        mfs.extend(self.max_idle_closed.collect());
        mfs.extend(self.max_idle_time_closed.collect());
        mfs.extend(self.max_lifetime_closed.collect());

        mfs
    }
}