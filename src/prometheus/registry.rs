use lazy_static::lazy_static;
use std::sync::Arc;
use std::sync::Mutex;

const CAP_METRIC_CHAN: usize = 1000;
const CAP_DESC_CHAN: usize = 10;

lazy_static! {
    static ref DEFAULT_REGISTRY: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
    pub static ref DEFAULT_REGISTERER: Arc<Mutex<dyn Registerer>> = DEFAULT_REGISTRY.clone();
    pub static ref DEFAULT_GATHERER: Arc<Mutex<dyn Gatherer>> = DEFAULT_REGISTRY.clone();
}

struct Registry {
    mtx: RwLock<()>,
    collectors_by_id: HashMap<u64, Box<dyn Collector>>,
    desc_ids: HashMap<u64, ()>,
    dim_hashes_by_name: HashMap<String, u64>,
    unchecked_collectors: Vec<Box<dyn Collector>>,
    pedantic_checks_enabled: bool,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            mtx: RwLock::new(()),
            collectors_by_id: HashMap::new(),
            desc_ids: HashMap::new(),
            dim_hashes_by_name: HashMap::new(),
            unchecked_collectors: Vec::new(),
            pedantic_checks_enabled: false,
        }
    }

    pub fn new_pedantic_registry() -> Self {
        let mut registry = Registry::new();
        registry.pedantic_checks_enabled = true;
        registry
    }

    pub fn must_register(&self, collectors: Vec<Box<dyn Collector>>) {
        for c in collectors {
            if let Err(err) = self.register(c) {
                panic!("{}", err);
            }
        }
    }

    pub fn register(&self, c: Box<dyn Collector>) -> Result<(), Box<dyn std::error::Error>> {
        let desc_chan = std::sync::mpsc::channel();
        let (desc_sender, desc_receiver) = desc_chan;
        let mut new_desc_ids = HashMap::new();
        let mut new_dim_hashes_by_name = HashMap::new();
        let mut collector_id: u64 = 0;
        let mut duplicate_desc_err: Option<Box<dyn std::error::Error>> = None;

        std::thread::spawn(move || {
            c.describe(desc_sender);
            drop(desc_sender);
        });

        let _lock = self.mtx.write().unwrap();
        defer! {
            for _ in desc_receiver {}
            drop(_lock);
        }

        for desc in desc_receiver {
            if let Some(err) = desc.err() {
                return Err(Box::new(fmt::Error::new(
                    fmt::Error,
                    format!("descriptor {} is invalid: {}", desc, err),
                )));
            }

            if self.desc_ids.contains_key(&desc.id()) {
                duplicate_desc_err = Some(Box::new(fmt::Error::new(fmt::Error, format!("descriptor {} already exists with the same fully-qualified name and const label values", desc))));
            }

            if !new_desc_ids.contains_key(&desc.id()) {
                new_desc_ids.insert(desc.id(), ());
                collector_id ^= desc.id();
            }

            if let Some(dim_hash) = self.dim_hashes_by_name.get(&desc.fq_name()) {
                if *dim_hash != desc.dim_hash() {
                    return Err(Box::new(fmt::Error::new(fmt::Error, format!("a previously registered descriptor with the same fully-qualified name as {} has different label names or a different help string", desc))));
                }
                continue;
            }

            if let Some(dim_hash) = new_dim_hashes_by_name.get(&desc.fq_name()) {
                if *dim_hash != desc.dim_hash() {
                    return Err(Box::new(fmt::Error::new(fmt::Error, format!("descriptors reported by collector have inconsistent label names or help strings for the same fully-qualified name, offender is {}", desc))));
                }
                continue;
            }
            new_dim_hashes_by_name.insert(desc.fq_name(), desc.dim_hash());
        }

        if new_desc_ids.is_empty() {
            self.unchecked_collectors.push(c);
            return Ok(());
        }

        if let Some(existing) = self.collectors_by_id.get(&collector_id) {
            return Err(Box::new(AlreadyRegisteredError {
                existing_collector: existing.clone(),
                new_collector: c,
            }));
        }

        if let Some(err) = duplicate_desc_err {
            return Err(err);
        }

        self.collectors_by_id.insert(collector_id, c);
        for hash in new_desc_ids.keys() {
            self.desc_ids.insert(*hash, ());
        }
        for (name, dim_hash) in new_dim_hashes_by_name {
            self.dim_hashes_by_name.insert(name, dim_hash);
        }
        Ok(())
    }

    pub fn unregister(&self, c: Box<dyn Collector>) -> bool {
        let (desc_sender, desc_receiver) = std::sync::mpsc::channel();
        let mut desc_ids = HashMap::new();
        let mut collector_id: u64 = 0;

        std::thread::spawn(move || {
            c.describe(desc_sender);
            drop(desc_sender);
        });

        for desc in desc_receiver {
            if !desc_ids.contains_key(&desc.id()) {
                collector_id ^= desc.id();
                desc_ids.insert(desc.id(), ());
            }
        }

        {
            let _read_lock = self.mtx.read().unwrap();
            if !self.collectors_by_id.contains_key(&collector_id) {
                return false;
            }
        }

        let _write_lock = self.mtx.write().unwrap();

        self.collectors_by_id.remove(&collector_id);
        for id in desc_ids.keys() {
            self.desc_ids.remove(id);
        }
        // dim_hashes_by_name is left untouched as those must be consistent
        // throughout the lifetime of a program.
        true
    }

    pub fn gather(&self) -> Result<Vec<MetricFamily>, MultiError> {
        let _read_lock = self.mtx.read().unwrap();

        if self.collectors_by_id.is_empty() && self.unchecked_collectors.is_empty() {
            // Fast path.
            return Ok(Vec::new());
        }

        let (checked_metric_sender, checked_metric_receiver) = channel();
        let (unchecked_metric_sender, unchecked_metric_receiver) = channel();
        let mut metric_hashes = HashMap::new();
        let mut errs = MultiError::new();
        let mut registered_desc_ids = HashMap::new();

        let goroutine_budget =
            AtomicUsize::new(self.collectors_by_id.len() + self.unchecked_collectors.len());
        let metric_families_by_name = Arc::new(Mutex::new(HashMap::new()));
        let checked_collectors = Arc::new(Mutex::new(Vec::new()));
        let unchecked_collectors = Arc::new(Mutex::new(Vec::new()));

        for collector in self.collectors_by_id.values() {
            checked_collectors.lock().unwrap().push(collector.clone());
        }
        for collector in &self.unchecked_collectors {
            unchecked_collectors.lock().unwrap().push(collector.clone());
        }

        if self.pedantic_checks_enabled {
            for id in self.desc_ids.keys() {
                registered_desc_ids.insert(*id, ());
            }
        }

        drop(_read_lock);

        let collect_worker = || loop {
            let collector = {
                let mut checked_collectors = checked_collectors.lock().unwrap();
                if let Some(collector) = checked_collectors.pop() {
                    collector
                } else {
                    let mut unchecked_collectors = unchecked_collectors.lock().unwrap();
                    if let Some(collector) = unchecked_collectors.pop() {
                        collector
                    } else {
                        break;
                    }
                }
            };
            collector.collect(checked_metric_sender.clone());
            goroutine_budget.fetch_sub(1, Ordering::SeqCst);
        };

        // Start the first worker now to make sure at least one is running.
        thread::spawn(collect_worker);

        // Close checked_metric_receiver and unchecked_metric_receiver once all collectors are collected.
        thread::spawn(move || {
            while goroutine_budget.load(Ordering::SeqCst) > 0 {
                thread::yield_now();
            }
            drop(checked_metric_sender);
            drop(unchecked_metric_sender);
        });

        // Drain checked_metric_receiver and unchecked_metric_receiver in case of premature return.
        defer! {
            for _ in checked_metric_receiver {}
            for _ in unchecked_metric_receiver {}
        }

        // Copy the channel references so we can nil them out later to remove them from the select statements below.
        let mut cmc = Some(checked_metric_receiver);
        let mut umc = Some(unchecked_metric_receiver);

        loop {
            select! {
                metric = cmc.as_ref().unwrap().recv() => match metric {
                    Ok(metric) => {
                        errs.append(process_metric(
                            metric, &metric_families_by_name,
                            &mut metric_hashes,
                            &registered_desc_ids,
                        ));
                    }
                    Err(_) => cmc = None,
                },
                metric = umc.as_ref().unwrap().recv() => match metric {
                    Ok(metric) => {
                        errs.append(process_metric(
                            metric, &metric_families_by_name,
                            &mut metric_hashes,
                            &None,
                        ));
                    }
                    Err(_) => umc = None,
                },
                default => {
                    if goroutine_budget.load(Ordering::SeqCst) <= 0 || checked_collectors.lock().unwrap().is_empty() && unchecked_collectors.lock().unwrap().is_empty() {
                        break;
                    }
                    thread::spawn(collect_worker);
                    goroutine_budget.fetch_sub(1, Ordering::SeqCst);
                    thread::yield_now();
                }
            }
            if cmc.is_none() && umc.is_none() {
                break;
            }
        }

        let metric_families_by_name = Arc::try_unwrap(metric_families_by_name)
            .unwrap()
            .into_inner()
            .unwrap();
        Ok(internal::normalize_metric_families(metric_families_by_name))
    }

    pub fn describe(&self, ch: &mut Sender<Desc>) {
        let _read_lock = self.mtx.read().unwrap();

        // Only report the checked Collectors; unchecked collectors don't report any Desc.
        for collector in self.collectors_by_id.values() {
            collector.describe(ch.clone());
        }
    }

    pub fn collect(&self, ch: &mut Sender<Metric>) {
        let _read_lock = self.mtx.read().unwrap();

        for collector in self.collectors_by_id.values() {
            collector.collect(ch.clone());
        }
        for collector in &self.unchecked_collectors {
            collector.collect(ch.clone());
        }
    }
}

pub trait Registerer {
    /// Registers a new Collector to be included in metrics collection.
    /// It returns an error if the descriptors provided by the Collector are invalid
    /// or if they — in combination with descriptors of already registered Collectors —
    /// do not fulfill the consistency and uniqueness criteria described in the documentation of metric.Desc.
    ///
    /// If the provided Collector is equal to a Collector already registered
    /// (which includes the case of re-registering the same Collector), the
    /// returned error is an instance of AlreadyRegisteredError, which
    /// contains the previously registered Collector.
    ///
    /// A Collector whose Describe method does not yield any Desc is treated
    /// as unchecked. Registration will always succeed. No check for
    /// re-registering (see previous paragraph) is performed. Thus, the
    /// caller is responsible for not double-registering the same unchecked
    /// Collector, and for providing a Collector that will not cause
    /// inconsistent metrics on collection. (This would lead to scrape errors.)
    fn register(&self, collector: Box<dyn Collector>) -> Result<(), AlreadyRegisteredError>;

    /// Works like Register but registers any number of Collectors and panics upon the first registration that causes an error.
    fn must_register(&self, collectors: Vec<Box<dyn Collector>>);

    /// Unregisters the Collector that equals the Collector passed in as an argument.
    /// (Two Collectors are considered equal if their Describe method yields the same set of descriptors.)
    /// The function returns whether a Collector was unregistered. Note that an unchecked
    /// Collector cannot be unregistered (as its Describe method does not yield any descriptor).
    ///
    /// Note that even after unregistering, it will not be possible to
    /// register a new Collector that is inconsistent with the unregistered
    /// Collector, e.g. a Collector collecting metrics with the same name but
    /// a different help string. The rationale here is that the same registry
    /// instance must only collect consistent metrics throughout its lifetime.
    fn unregister(&self, collector: Box<dyn Collector>) -> bool;
}

pub struct AlreadyRegisteredError {
    pub existing_collector: Box<dyn Collector>,
    pub new_collector: Box<dyn Collector>,
}

impl std::fmt::Debug for AlreadyRegisteredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "duplicate metrics collector registration attempted")
    }
}

impl std::fmt::Display for AlreadyRegisteredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "duplicate metrics collector registration attempted")
    }
}

impl std::error::Error for AlreadyRegisteredError {}

pub trait Gatherer {
    /// Calls the Collect method of the registered Collectors and then
    /// gathers the collected metrics into a lexicographically sorted slice
    /// of uniquely named MetricFamily protobufs. Gather ensures that the
    /// returned slice is valid and self-consistent so that it can be used
    /// for valid exposition. As an exception to the strict consistency
    /// requirements described for metric.Desc, Gather will tolerate
    /// different sets of label names for metrics of the same metric family.
    ///
    /// Even if an error occurs, Gather attempts to gather as many metrics as
    /// possible. Hence, if a non-nil error is returned, the returned
    /// MetricFamily slice could be nil (in case of a fatal error that
    /// prevented any meaningful metric collection) or contain a number of
    /// MetricFamily protobufs, some of which might be incomplete, and some
    /// might be missing altogether. The returned error (which might be a
    /// MultiError) explains the details. Note that this is mostly useful for
    /// debugging purposes. If the gathered protobufs are to be used for
    /// exposition in actual monitoring, it is almost always better to not
    /// expose an incomplete result and instead disregard the returned
    /// MetricFamily protobufs in case the returned error is non-nil.
    fn gather(&self) -> Result<Vec<MetricFamily>, MultiError>;
}

pub struct MetricFamily {
    // Define the fields for MetricFamily
}

pub struct MultiError {
    errors: Vec<Box<dyn std::error::Error>>,
}

impl MultiError {
    pub fn new() -> Self {
        MultiError { errors: Vec::new() }
    }

    pub fn append(&mut self, err: Box<dyn std::error::Error>) {
        self.errors.push(err);
    }

    pub fn maybe_unwrap(self) -> Result<(), Self> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Debug for MultiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error(s) occurred:", self.errors.len())?;
        for err in &self.errors {
            write!(f, "\n* {}", err)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for MultiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error(s) occurred:", self.errors.len())?;
        for err in &self.errors {
            write!(f, "\n* {}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for MultiError {}

struct ProcessCollectorOpts;

impl Default for ProcessCollectorOpts {
    fn default() -> Self {
        ProcessCollectorOpts
    }
}

struct ProcessCollector;

impl ProcessCollector {
    pub fn new(_opts: ProcessCollectorOpts) -> Self {
        ProcessCollector
    }
}

struct GoCollector;

impl GoCollector {
    pub fn new() -> Self {
        GoCollector
    }
}

trait Registerer {
    fn register(&self, collector: Box<dyn Collector>) -> Result<(), String>;
    fn must_register(&self, collector: Box<dyn Collector>);
}

trait Gatherer {
    fn gather(&self) -> Vec<MetricFamily>;
}

trait Collector {
    fn describe(&self, descs: &mut Vec<Desc>);
    fn collect(&self, metrics: &mut Vec<Metric>);
}

struct MetricFamily;
// struct Desc;
struct Metric;

pub fn register(c: Box<dyn Collector>) -> Result<(), Box<dyn std::error::Error>> {
    DEFAULT_REGISTERER.lock().unwrap().register(c)
}

pub fn must_register(cs: Vec<Box<dyn Collector>>) {
    DEFAULT_REGISTERER.lock().unwrap().must_register(cs);
}

pub fn unregister(c: Box<dyn Collector>) -> bool {
    DEFAULT_REGISTERER.lock().unwrap().unregister(c)
}

pub type GathererFunc =
    Box<dyn Fn() -> Result<Vec<MetricFamily>, Box<dyn std::error::Error>> + Send + Sync>;

impl Gatherer for GathererFunc {
    fn gather(&self) -> Result<Vec<MetricFamily>, MultiError> {
        self()
    }
}

pub struct AlreadyRegisteredError {
    pub existing_collector: Box<dyn Collector>,
    pub new_collector: Box<dyn Collector>,
}

impl std::fmt::Display for AlreadyRegisteredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "duplicate metrics collector registration attempted")
    }
}

impl std::fmt::Debug for AlreadyRegisteredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "duplicate metrics collector registration attempted")
    }
}

impl std::error::Error for AlreadyRegisteredError {}

pub struct MultiError(Vec<Box<dyn std::error::Error>>);

impl std::fmt::Display for MultiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "");
        }
        write!(f, "{} error(s) occurred:", self.0.len())?;
        for err in &self.0 {
            write!(f, "\n* {}", err)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for MultiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl std::error::Error for MultiError {}

impl MultiError {
    pub fn append(&mut self, err: Box<dyn std::error::Error>) {
        self.errors.push(err);
    }

    pub fn maybe_unwrap(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.errors.len() {
            0 => Ok(()),
            1 => Err(self.errors.into_iter().next().unwrap()),
            _ => Err(Box::new(self)),
        }
    }
}

pub fn process_metric(
    metric: Metric,
    metric_families_by_name: &mut HashMap<String, MetricFamily>,
    metric_hashes: &mut HashMap<u64, ()>,
    registered_desc_ids: Option<&HashMap<u64, ()>>,
) -> Result<(), Box<dyn Error>> {
    let desc = metric.desc();
    // Wrapped metrics collected by an unchecked Collector can have an invalid Desc.
    if let Some(err) = desc.err() {
        return Err(Box::new(fmt::Error::new(fmt::Error, err)));
    }
    let mut dto_metric = Metric::default();
    metric.write(&mut dto_metric)?;

    let metric_family = metric_families_by_name
        .entry(desc.fq_name().clone())
        .or_insert_with(|| {
            let mut mf = MetricFamily::default();
            mf.set_name(desc.fq_name().clone());
            mf.set_help(desc.help().clone());
            mf
        });

    if metric_family.get_help() != desc.help() {
        return Err(Box::new(fmt::Error::new(
            fmt::Error,
            format!(
                "collected metric {} {} has help {} but should have {}",
                desc.fq_name(),
                dto_metric,
                desc.help(),
                metric_family.get_help()
            ),
        )));
    }

    match metric_family.get_field_type() {
        MetricType::Counter => {
            if dto_metric.counter.is_none() {
                return Err(Box::new(fmt::Error::new(
                    fmt::Error,
                    format!(
                        "collected metric {} {} should be a Counter",
                        desc.fq_name(),
                        dto_metric
                    ),
                )));
            }
        }
        MetricType::Gauge => {
            if dto_metric.gauge.is_none() {
                return Err(Box::new(fmt::Error::new(
                    fmt::Error,
                    format!(
                        "collected metric {} {} should be a Gauge",
                        desc.fq_name(),
                        dto_metric
                    ),
                )));
            }
        }
        MetricType::Summary => {
            if dto_metric.summary.is_none() {
                return Err(Box::new(fmt::Error::new(
                    fmt::Error,
                    format!(
                        "collected metric {} {} should be a Summary",
                        desc.fq_name(),
                        dto_metric
                    ),
                )));
            }
        }
        MetricType::Untyped => {
            if dto_metric.untyped.is_none() {
                return Err(Box::new(fmt::Error::new(
                    fmt::Error,
                    format!(
                        "collected metric {} {} should be Untyped",
                        desc.fq_name(),
                        dto_metric
                    ),
                )));
            }
        }
        MetricType::Histogram => {
            if dto_metric.histogram.is_none() {
                return Err(Box::new(fmt::Error::new(
                    fmt::Error,
                    format!(
                        "collected metric {} {} should be a Histogram",
                        desc.fq_name(),
                        dto_metric
                    ),
                )));
            }
        }
        _ => panic!("encountered MetricFamily with invalid type"),
    }

    if let Some(registered_desc_ids) = registered_desc_ids {
        if !registered_desc_ids.contains_key(&desc.id()) {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                    "collected metric {} {} with unregistered descriptor {}",
                    metric_family.get_name(),
                    dto_metric,
                    desc
                ),
            )));
        }
        check_desc_consistency(metric_family, &dto_metric, &desc)?;
    }

    check_metric_consistency(metric_family, &dto_metric, metric_hashes)?;
    metric_family.metric.push(dto_metric);
    Ok(())
}

pub fn check_suffix_collisions(
    mf: &MetricFamily,
    mfs: &HashMap<String, MetricFamily>,
) -> Result<(), Box<dyn Error>> {
    let new_name = mf.get_name();
    let new_type = mf.get_field_type();
    let mut new_name_without_suffix = String::new();

    if new_name.ends_with("_count") {
        new_name_without_suffix = new_name[..new_name.len() - 6].to_string();
    } else if new_name.ends_with("_sum") {
        new_name_without_suffix = new_name[..new_name.len() - 4].to_string();
    } else if new_name.ends_with("_bucket") {
        new_name_without_suffix = new_name[..new_name.len() - 7].to_string();
    }

    if !new_name_without_suffix.is_empty() {
        if let Some(existing_mf) = mfs.get(&new_name_without_suffix) {
            match existing_mf.get_field_type() {
                MetricType::Summary => {
                    if !new_name.ends_with("_bucket") {
                        return Err(Box::new(fmt::Error::new(fmt::Error, format!(
                            "collected metric named {} collides with previously collected summary named {}",
                            new_name, new_name_without_suffix
                        ))));
                    }
                }
                MetricType::Histogram => {
                    return Err(Box::new(fmt::Error::new(fmt::Error, format!(
                        "collected metric named {} collides with previously collected histogram named {}",
                        new_name, new_name_without_suffix
                    ))));
                }
                _ => {}
            }
        }
    }

    if new_type == MetricType::Summary || new_type == MetricType::Histogram {
        if mfs.contains_key(&(new_name.to_string() + "_count")) {
            return Err(Box::new(fmt::Error::new(fmt::Error, format!(
                "collected histogram or summary named {} collides with previously collected metric named {}",
                new_name, new_name.to_string() + "_count"
            ))));
        }
        if mfs.contains_key(&(new_name.to_string() + "_sum")) {
            return Err(Box::new(fmt::Error::new(fmt::Error, format!(
                "collected histogram or summary named {} collides with previously collected metric named {}",
                new_name, new_name.to_string() + "_sum"
            ))));
        }
    }

    if new_type == MetricType::Histogram {
        if mfs.contains_key(&(new_name.to_string() + "_bucket")) {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                "collected histogram named {} collides with previously collected metric named {}",
                new_name, new_name.to_string() + "_bucket"
            ),
            )));
        }
    }

    Ok(())
}

pub fn check_metric_consistency(
    metric_family: &MetricFamily,
    dto_metric: &Metric,
    metric_hashes: &mut HashMap<u64, ()>,
) -> Result<(), Box<dyn Error>> {
    let name = metric_family.get_name();

    // Type consistency with metric family.
    if (metric_family.get_field_type() == MetricType::Gauge && dto_metric.gauge.is_none())
        || (metric_family.get_field_type() == MetricType::Counter && dto_metric.counter.is_none())
        || (metric_family.get_field_type() == MetricType::Summary && dto_metric.summary.is_none())
        || (metric_family.get_field_type() == MetricType::Histogram
            && dto_metric.histogram.is_none())
        || (metric_family.get_field_type() == MetricType::Untyped && dto_metric.untyped.is_none())
    {
        return Err(Box::new(fmt::Error::new(
            fmt::Error,
            format!(
                "collected metric {} {{ {:?} }} is not a {:?}",
                name,
                dto_metric,
                metric_family.get_field_type()
            ),
        )));
    }

    let mut previous_label_name = String::new();
    for label_pair in dto_metric.get_label() {
        let label_name = label_pair.get_name();
        if label_name == previous_label_name {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                    "collected metric {} {{ {:?} }} has two or more labels with the same name: {}",
                    name, dto_metric, label_name
                ),
            )));
        }
        if !check_label_name(label_name) {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                    "collected metric {} {{ {:?} }} has a label with an invalid name: {}",
                    name, dto_metric, label_name
                ),
            )));
        }
        if dto_metric.summary.is_some() && label_name == "quantile" {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                    "collected metric {} {{ {:?} }} must not have an explicit {} label",
                    name, dto_metric, "quantile"
                ),
            )));
        }
        if !str::from_utf8(label_pair.get_value().as_bytes()).is_ok() {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                "collected metric {} {{ {:?} }} has a label named {} whose value is not utf8: {:?}",
                name, dto_metric, label_name, label_pair.get_value()
            ),
            )));
        }
        previous_label_name = label_name.to_string();
    }

    // Is the metric unique (i.e. no other metric with the same name and the same labels)?
    let mut hasher = Xxh3::new();
    hasher.update(name.as_bytes());
    hasher.update(b"\x00");
    // Make sure label pairs are sorted. We depend on it for the consistency check.
    let mut labels = dto_metric.get_label().clone();
    labels.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for lp in labels {
        hasher.update(lp.get_name().as_bytes());
        hasher.update(b"\x00");
        hasher.update(lp.get_value().as_bytes());
        hasher.update(b"\x00");
    }
    if let Some(timestamp) = dto_metric.timestamp_ms {
        hasher.update(timestamp.to_string().as_bytes());
        hasher.update(b"\x00");
    }
    let h_sum = hasher.digest();
    if metric_hashes.contains_key(&h_sum) {
        return Err(Box::new(fmt::Error::new(fmt::Error, format!(
            "collected metric {} {{ {:?} }} was collected before with the same name and label values",
            name, dto_metric
        ))));
    }
    metric_hashes.insert(h_sum, ());
    Ok(())
}

pub fn check_desc_consistency(
    metric_family: &MetricFamily,
    dto_metric: &Metric,
    desc: &Desc,
) -> Result<(), Box<dyn Error>> {
    // Desc help consistency with metric family help.
    if metric_family.get_help() != desc.get_help() {
        return Err(Box::new(fmt::Error::new(
            fmt::Error,
            format!(
                "collected metric {} {:?} has help {} but should have {}",
                metric_family.get_name(),
                dto_metric,
                metric_family.get_help(),
                desc.get_help()
            ),
        )));
    }

    // Is the desc consistent with the content of the metric?
    let mut lps_from_desc: Vec<LabelPair> = desc.get_const_label_pairs().clone();
    for l in desc.get_variable_labels() {
        lps_from_desc.push(LabelPair {
            name: l.clone(),
            value: String::new(),
        });
    }

    if lps_from_desc.len() != dto_metric.get_label().len() {
        return Err(Box::new(fmt::Error::new(
            fmt::Error,
            format!(
                "labels in collected metric {} {:?} are inconsistent with descriptor {:?}",
                metric_family.get_name(),
                dto_metric,
                desc
            ),
        )));
    }

    lps_from_desc.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    for (i, lp_from_desc) in lps_from_desc.iter().enumerate() {
        let lp_from_metric = &dto_metric.get_label()[i];
        if lp_from_desc.get_name() != lp_from_metric.get_name()
            || (!lp_from_desc.get_value().is_empty()
                && lp_from_desc.get_value() != lp_from_metric.get_value())
        {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                format!(
                    "labels in collected metric {} {:?} are inconsistent with descriptor {:?}",
                    metric_family.get_name(),
                    dto_metric,
                    desc
                ),
            )));
        }
    }
    Ok(())
}

pub trait TransactionalGatherer {
    /// Gather returns metrics in a lexicographically sorted slice
    /// of uniquely named MetricFamily protobufs. Gather ensures that the
    /// returned slice is valid and self-consistent so that it can be used
    /// for valid exposition. As an exception to the strict consistency
    /// requirements described for metric.Desc, Gather will tolerate
    /// different sets of label names for metrics of the same metric family.
    ///
    /// Even if an error occurs, Gather attempts to gather as many metrics as
    /// possible. Hence, if a non-nil error is returned, the returned
    /// MetricFamily slice could be nil (in case of a fatal error that
    /// prevented any meaningful metric collection) or contain a number of
    /// MetricFamily protobufs, some of which might be incomplete, and some
    /// might be missing altogether. The returned error (which might be a
    /// MultiError) explains the details. Note that this is mostly useful for
    /// debugging purposes. If the gathered protobufs are to be used for
    /// exposition in actual monitoring, it is almost always better to not
    /// expose an incomplete result and instead disregard the returned
    /// MetricFamily protobufs in case the returned error is non-nil.
    ///
    /// Important: done is expected to be triggered (even if the error occurs!)
    /// once caller does not need returned slice of MetricFamily.
    fn gather(&self) -> (Vec<MetricFamily>, Box<dyn FnOnce()>, Result<(), MultiError>);
}

impl TransactionalGatherer for MultiTRegistry {
    fn gather(&self) -> (Vec<MetricFamily>, Box<dyn FnOnce()>, Result<(), MultiError>) {
        let mut errs = MultiError::new();

        let mut mfs: Vec<MetricFamily> = Vec::new();
        let mut d_fns: Vec<Box<dyn FnOnce()>> = Vec::with_capacity(self.t_gatherers.len());

        for g in &self.t_gatherers {
            let (m, d, err) = g.gather();
            errs.append(Box::new(err));

            mfs.extend(m);
            d_fns.push(Box::new(d));
        }

        mfs.sort_by(|a, b| a.get_name().cmp(b.get_name()));

        let done = Box::new(move || {
            for d in d_fns {
                d();
            }
        });

        (mfs, done, errs.maybe_unwrap())
    }
}

pub fn to_transactional_gatherer(g: Box<dyn Gatherer>) -> Box<dyn TransactionalGatherer> {
    Box::new(NoTransactionGatherer { g })
}

struct NoTransactionGatherer {
    g: Box<dyn Gatherer>,
}

pub struct MultiTRegistry {
    t_gatherers: Vec<Box<dyn TransactionalGatherer>>,
}

impl MultiTRegistry {
    pub fn new(t_gatherers: Vec<Box<dyn TransactionalGatherer>>) -> Self {
        MultiTRegistry { t_gatherers }
    }
}

pub trait Collector: Send + Sync {
    fn describe(&self, descs: Sender<Desc>);
    fn collect(&self, metrics: Sender<Metric>);
}

pub fn write_to_textfile(filename: &str, g: &dyn Gatherer) -> io::Result<()> {
    let tmp_path = Path::new(filename)
        .parent()
        .unwrap()
        .join(format!("{}.tmp", filename));
    let mut tmp_file = File::create(&tmp_path)?;

    let mfs = g
        .gather()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    for mf in mfs {
        let mut buffer = Vec::new();
        expfmt::metric_family_to_text(&mut buffer, &mf)?;
        tmp_file.write_all(&buffer)?;
    }

    tmp_file.sync_all()?;
    fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o644))?;
    fs::rename(tmp_path, filename)?;

    Ok(())
}

#[derive(Default)]
pub struct Metric {
    pub counter: Option<Counter>,
    pub gauge: Option<Gauge>,
    pub summary: Option<Summary>,
    pub untyped: Option<Untyped>,
    pub histogram: Option<Histogram>,
}

impl Metric {
    pub fn write(&self, dto_metric: &mut Metric) -> Result<(), Box<dyn Error>> {
        // Implement the method to write the metric
        Ok(())
    }
}

pub enum MetricType {
    Counter,
    Gauge,
    Summary,
    Untyped,
    Histogram,
}

pub type Gatherers = Vec<Box<dyn Gatherer>>;

impl TransactionalGatherer for NoTransactionGatherer {
    fn gather(&self) -> (Vec<MetricFamily>, Box<dyn FnOnce()>, Result<(), MultiError>) {
        match self.g.gather() {
            Ok(mfs) => (mfs, Box::new(|| {}), Ok(())),
            Err(err) => (Vec::new(), Box::new(|| {}), Err(MultiError::from(err))),
        }
    }
}

impl From<Box<dyn std::error::Error>> for MultiError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        let mut multi_error = MultiError::new();
        multi_error.append(err);
        multi_error
    }
}
