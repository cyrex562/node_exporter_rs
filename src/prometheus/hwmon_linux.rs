use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use regex::Regex;
use prometheus::{self, core::{Collector, Desc, Metric, Opts}};
use std::time::Duration;

lazy_static! {
    static ref HWMON_INVALID_METRIC_CHARS: Regex = Regex::new("[^a-z0-9:_]").unwrap();
    static ref HWMON_FILENAME_FORMAT: Regex = Regex::new(r"^(?P<type>[^0-9]+)(?P<id>[0-9]*)?(_(?P<property>.+))?$").unwrap();
    static ref HWMON_SENSOR_TYPES: Vec<&'static str> = vec![
        "vrm", "beep_enable", "update_interval", "in", "cpu", "fan",
        "pwm", "temp", "curr", "power", "energy", "humidity",
        "intrusion",
    ];
}

struct HwMonCollector {
    device_filter: DeviceFilter,
    sensor_filter: DeviceFilter,
    logger: slog::Logger,
}

impl HwMonCollector {
    fn new(logger: slog::Logger) -> Self {
        HwMonCollector {
            logger,
            device_filter: DeviceFilter::new(),
            sensor_filter: DeviceFilter::new(),
        }
    }

    fn clean_metric_name(name: &str) -> String {
        let lower = name.to_lowercase();
        let replaced = HWMON_INVALID_METRIC_CHARS.replace_all(&lower, "_");
        replaced.trim_matches('_').to_string()
    }

    fn add_value_file(data: &mut HashMap<String, HashMap<String, String>>, sensor: &str, prop: &str, file: &Path) {
        if let Ok(raw) = fs::read_to_string(file) {
            let value = raw.trim().to_string();
            data.entry(sensor.to_string()).or_default().insert(prop.to_string(), value);
        }
    }

    fn sys_read_file(file: &Path) -> Result<Vec<u8>, std::io::Error> {
        let mut f = fs::File::open(file)?;
        let mut b = vec![0; 128];
        let n = nix::unistd::read(f.as_raw_fd(), &mut b)?;
        if n < 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "failed to read file"));
        }
        Ok(b[..n].to_vec())
    }

    fn explode_sensor_filename(filename: &str) -> Option<(String, i32, String)> {
        let matches = HWMON_FILENAME_FORMAT.captures(filename)?;
        let sensor_type = matches.name("type")?.as_str().to_string();
        let sensor_property = matches.name("property").map_or("", |m| m.as_str()).to_string();
        let sensor_num = matches.name("id").map_or(0, |m| m.as_str().parse().unwrap_or(0));
        Some((sensor_type, sensor_num, sensor_property))
    }

    fn collect_sensor_data(dir: &Path, data: &mut HashMap<String, HashMap<String, String>>) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let filename = entry.file_name().into_string().unwrap();
            if let Some((sensor_type, sensor_num, sensor_property)) = Self::explode_sensor_filename(&filename) {
                if HWMON_SENSOR_TYPES.contains(&sensor_type.as_str()) {
                    Self::add_value_file(data, &format!("{}{}", sensor_type, sensor_num), &sensor_property, &entry.path());
                }
            }
        }
        Ok(())
    }

    fn update_hwmon(&self, ch: &mut dyn FnMut(Box<dyn Metric>), dir: &Path) -> Result<(), std::io::Error> {
        let hwmon_name = self.hwmon_name(dir)?;

        if self.device_filter.ignored(&hwmon_name) {
            self.logger.debug("ignoring hwmon chip", &["chip", &hwmon_name]);
            return Ok(());
        }

        let mut data = HashMap::new();
        Self::collect_sensor_data(dir, &mut data)?;
        if dir.join("device").exists() {
            Self::collect_sensor_data(&dir.join("device"), &mut data)?;
        }

        if let Ok(hwmon_chip_name) = self.hwmon_human_readable_chip_name(dir) {
            let desc = Desc::new(
                "node_hwmon_chip_names",
                "Annotation metric for human-readable chip names",
                vec!["chip", "chip_name"],
                HashMap::new(),
            );
            ch(Box::new(prometheus::Gauge::new(desc, 1.0, vec![hwmon_name.clone(), hwmon_chip_name])));
        }

        for (sensor, sensor_data) in data {
            if self.sensor_filter.ignored(&format!("{};{}", hwmon_name, sensor)) {
                self.logger.debug("ignoring sensor", &["sensor", &sensor]);
                continue;
            }

            let (sensor_type, _, _) = Self::explode_sensor_filename(&sensor).unwrap();
            let labels = vec![hwmon_name.clone(), sensor.clone()];

            if let Some(label_text) = sensor_data.get("label") {
                let label = label_text.to_string();
                let desc = Desc::new("node_hwmon_sensor_label", "Label for given chip and sensor", vec!["chip", "sensor", "label"], HashMap::new());
                ch(Box::new(prometheus::Gauge::new(desc, 1.0, vec![hwmon_name.clone(), sensor.clone(), label])));
            }

            // Handle specific sensor types and properties
            // ...

            // Fallback, just dump the metric as is
            for (element, value) in sensor_data {
                if element == "label" {
                    continue;
                }
                let name = if element == "input" && sensor_data.contains_key("") {
                    format!("node_hwmon_{}_input", sensor_type)
                } else if element.is_empty() {
                    format!("node_hwmon_{}", sensor_type)
                } else {
                    format!("node_hwmon_{}_{}", sensor_type, Self::clean_metric_name(&element))
                };
                if let Ok(parsed_value) = value.parse::<f64>() {
                    let desc = Desc::new(&name, &format!("Hardware monitor {} element {}", sensor_type, element), vec!["chip", "sensor"], HashMap::new());
                    ch(Box::new(prometheus::Gauge::new(desc, parsed_value, labels.clone())));
                }
            }
        }

        Ok(())
    }

    fn hwmon_name(&self, dir: &Path) -> Result<String, std::io::Error> {
        let device_path = fs::read_link(dir.join("device"))?;
        let (dev_path_prefix, dev_name) = device_path.parent().unwrap().file_name().unwrap().to_str().unwrap().split_at(1);
        let clean_dev_name = Self::clean_metric_name(dev_name);
        let clean_dev_type = Self::clean_metric_name(dev_path_prefix);

        if !clean_dev_type.is_empty() && !clean_dev_name.is_empty() {
            return Ok(format!("{}_{}", clean_dev_type, clean_dev_name));
        }

        if !clean_dev_name.is_empty() {
            return Ok(clean_dev_name);
        }

        let sysname_raw = fs::read_to_string(dir.join("name"))?;
        if !sysname_raw.is_empty() {
            let clean_name = Self::clean_metric_name(&sysname_raw);
            if !clean_name.is_empty() {
                return Ok(clean_name);
            }
        }

        let real_dir = fs::read_link(dir)?;
        let name = real_dir.file_name().unwrap().to_str().unwrap();
        let clean_name = Self::clean_metric_name(name);
        if !clean_name.is_empty() {
            return Ok(clean_name);
        }

        Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Could not derive a monitoring name for {:?}", dir)))
    }

    fn hwmon_human_readable_chip_name(&self, dir: &Path) -> Result<String, std::io::Error> {
        let sysname_raw = fs::read_to_string(dir.join("name"))?;
        if !sysname_raw.is_empty() {
            let clean_name = Self::clean_metric_name(&sysname_raw);
            if !clean_name.is_empty() {
                return Ok(clean_name);
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Could not derive a human-readable chip type for {:?}", dir)))
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), std::io::Error> {
        let hwmon_path_name = Path::new("/sys/class/hwmon");

        for entry in fs::read_dir(hwmon_path_name)? {
            let entry = entry?;
            let hwmon_xpath_name = hwmon_path_name.join(entry.file_name());
            let file_info = fs::symlink_metadata(&hwmon_xpath_name)?;

            if file_info.file_type().is_symlink() {
                if let Ok(file_info) = fs::metadata(&hwmon_xpath_name) {
                    if !file_info.is_dir() {
                        continue;
                    }
                }
            }

            if !file_info.is_dir() {
                continue;
            }

            if let Err(err) = self.update_hwmon(ch, &hwmon_xpath_name) {
                return Err(err);
            }
        }

        Ok(())
    }
}