use rtnetlink::new_connection;
use slog::warn;
use std::collections::HashMap;
use std::error::Error;

struct ARPCollector {
    device_filter: DeviceFilter,
    entries: Desc,
    logger: slog::Logger,
}

impl ARPCollector {
    fn new(logger: slog::Logger) -> Result<Self, Box<dyn Error>> {
        let entries = Desc::new(
            "arp_entries".to_string(),
            "ARP entries by device".to_string(),
            vec!["device".to_string()],
            HashMap::new(),
        )?;

        Ok(ARPCollector {
            device_filter: DeviceFilter::new(),
            entries,
            logger,
        })
    }

    async fn get_total_arp_entries_rtnl(&self) -> Result<HashMap<String, u32>, Box<dyn Error>> {
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);

        let mut neighbors = handle.neigh().get().execute();
        let mut entries = HashMap::new();

        while let Some(neigh) = neighbors.try_next().await? {
            if let Some(device) = neigh.header.ifindex {
                let device_name = handle
                    .link()
                    .get(device)
                    .execute()
                    .try_next()
                    .await?
                    .unwrap()
                    .attrs()
                    .name
                    .clone();
                *entries.entry(device_name).or_insert(0) += 1;
            }
        }

        Ok(entries)
    }

    async fn update(&self, ch: &mut prometheus::proto::MetricFamily) -> Result<(), Box<dyn Error>> {
        let enumerated_entry = self.get_total_arp_entries_rtnl().await?;

        for (device, entry_count) in enumerated_entry {
            if self.device_filter.ignored(&device) {
                continue;
            }
            let mut m = prometheus::proto::Metric::default();
            m.set_label(protobuf::RepeatedField::from_vec(vec![
                prometheus::proto::LabelPair {
                    name: "device".to_string(),
                    value: device,
                    ..Default::default()
                },
            ]));
            m.set_gauge(prometheus::proto::Gauge {
                value: entry_count as f64,
                ..Default::default()
            });
            ch.set_metric(protobuf::RepeatedField::from_vec(vec![m]));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Collector for ARPCollector {
    async fn collect(&self) -> Vec<MetricFamily> {
        let mut metric_families = Vec::new();
        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name("arp_entries".to_string());
        mf.set_help("ARP entries by device".to_string());
        mf.set_field_type(prometheus::proto::MetricType::GAUGE);

        if let Err(err) = self.update(&mut mf).await {
            warn!(self.logger, "Failed to update ARP entries: {}", err);
        }

        metric_families.push(mf);
        metric_families
    }
}

struct DeviceFilter {
    exclude: Vec<String>,
    include: Vec<String>,
}

impl DeviceFilter {
    fn new() -> Self {
        DeviceFilter {
            exclude: vec![],
            include: vec![],
        }
    }

    fn ignored(&self, device: &str) -> bool {
        if !self.include.is_empty() {
            return !self.include.iter().any(|d| d == device);
        }
        if !self.exclude.is_empty() {
            return self.exclude.iter().any(|d| d == device);
        }
        false
    }
}

// #[tokio::main]
// async fn main() {
//     let decorator = slog_term::TermDecorator::new().build();
//     let drain = slog_term::CompactFormat::new(decorator).build().fuse();
//     let drain = slog_async::Async::new(drain).build().fuse();
//     let logger = slog::Logger::root(drain, o!());

//     let arp_collector = ARPCollector::new(logger.clone()).unwrap();
//     let registry = Registry::new_custom(Some("node".to_string()), None).unwrap();
//     registry.register(Box::new(arp_collector)).unwrap();

//     let encoder = TextEncoder::new();
//     let metric_families = registry.gather();
//     let mut buffer = Vec::new();
//     encoder.encode(&metric_families, &mut buffer).unwrap();
//     println!("{}", String::from_utf8(buffer).unwrap());
// }
