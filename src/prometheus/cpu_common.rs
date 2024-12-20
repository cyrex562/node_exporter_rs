use prometheus::{self, core::Desc, Opts};

const CPU_COLLECTOR_SUBSYSTEM: &str = "cpu";

lazy_static! {
    static ref NODE_CPU_SECONDS_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", CPU_COLLECTOR_SUBSYSTEM, "seconds_total"),
        "Seconds the CPUs spent in each mode.",
        vec!["cpu".to_string(), "mode".to_string()],
        None,
    ).unwrap();
}