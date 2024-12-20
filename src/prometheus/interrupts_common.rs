use prometheus::{self, core::{Collector, Desc, Opts, ValueType}};
use slog::Logger;
use regex::Regex;

struct InterruptsCollector {
    desc: TypedDesc,
    logger: Logger,
    name_filter: DeviceFilter,
    include_zeros: bool,
}

impl InterruptsCollector {
    fn new(logger: Logger) -> Result<Self, String> {
        let desc = TypedDesc {
            desc: Desc::new(
                "node_interrupts_total",
                "Interrupt details.",
                vec!["cpu", "type", "device"],
                HashMap::new(),
            ),
            value_type: ValueType::Counter,
        };

        let name_filter = DeviceFilter::new(
            Regex::new(&interrupts_exclude).unwrap(),
            Regex::new(&interrupts_include).unwrap(),
        );

        Ok(InterruptsCollector {
            desc,
            logger,
            name_filter,
            include_zeros: interrupts_include_zeros,
        })
    }
}

lazy_static! {
    static ref INTERRUPTS_INCLUDE: String = std::env::var("COLLECTOR_INTERRUPTS_NAME_INCLUDE").unwrap_or_default();
    static ref INTERRUPTS_EXCLUDE: String = std::env::var("COLLECTOR_INTERRUPTS_NAME_EXCLUDE").unwrap_or_default();
    static ref INTERRUPTS_INCLUDE_ZEROS: bool = std::env::var("COLLECTOR_INTERRUPTS_INCLUDE_ZEROS").unwrap_or("true".to_string()).parse().unwrap();
}