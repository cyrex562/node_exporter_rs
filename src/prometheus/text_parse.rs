use bytes::BytesMut;
use prometheus_client::proto::MetricFamily;
use std::collections::HashMap;
use std::io::{BufReader, Read};

// A StateFn is a function that represents a state in a state machine. By
// executing it, the state is progressed to the next state. The StateFn returns
// another StateFn, which represents the new state. The end state is represented
// by None.
type StateFn = Box<dyn Fn() -> Option<StateFn> + Send>;

// ParseError signals errors while parsing the simple and flat text-based
// exchange format.
#[derive(Debug)]
struct ParseError {
    line: usize,
    msg: String,
}

impl ParseError {
    fn new(line: usize, msg: String) -> Self {
        ParseError { line, msg }
    }
}

// Implement the Error trait for ParseError.
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "text format parsing error in line {}: {}",
            self.line, self.msg
        )
    }
}

impl std::error::Error for ParseError {}

struct TextParser<R: Read> {
    metric_families_by_name: HashMap<String, MetricFamily>,
    buf: BufReader<R>,       // Where the parsed input is read through.
    err: Option<ParseError>, // Most recent error.
    line_count: usize,       // Tracks the line count for error messages.
    current_byte: u8,        // The most recent byte read.
    current_token: BytesMut, // Re-used each time a token has to be gathered from multiple bytes.
    current_mf: Option<MetricFamily>,
    current_metric: Option<Metric>,
    current_label_pair: Option<LabelPair>,
    current_label_pairs: Vec<LabelPair>, // Temporarily stores label pairs while parsing a metric line.

    // The remaining member variables are only used for summaries/histograms.
    current_labels: HashMap<String, String>, // All labels including '__name__' but excluding 'quantile'/'le'
    // Summary specific.
    summaries: HashMap<u64, Metric>, // Key is created with labels_to_signature.
    current_quantile: f64,
    // Histogram specific.
    histograms: HashMap<u64, Metric>, // Key is created with labels_to_signature.
    current_bucket: f64,
    // These tell us if the currently processed line ends on '_count' or
    // '_sum' respectively and belong to a summary/histogram, representing the sample
    // count and sum of that summary/histogram.
    current_is_summary_count: bool,
    current_is_summary_sum: bool,
    current_is_histogram_count: bool,
    current_is_histogram_sum: bool,
    // These indicate if the metric name from the current line being parsed is inside
    // braces and if that metric name was found respectively.
    current_metric_is_inside_braces: bool,
    current_metric_inside_braces_is_present: bool,
}

impl<R: Read> TextParser<R> {
    fn new(reader: R) -> Self {
        TextParser {
            metric_families_by_name: HashMap::new(),
            buf: BufReader::new(reader),
            err: None,
            line_count: 0,
            current_byte: 0,
            current_token: BytesMut::new(),
            current_mf: None,
            current_metric: None,
            current_label_pair: None,
            current_label_pairs: Vec::new(),
            current_labels: HashMap::new(),
            summaries: HashMap::new(),
            current_quantile: 0.0,
            histograms: HashMap::new(),
            current_bucket: 0.0,
            current_is_summary_count: false,
            current_is_summary_sum: false,
            current_is_histogram_count: false,
            current_is_histogram_sum: false,
            current_metric_is_inside_braces: false,
            current_metric_inside_braces_is_present: false,
        }
    }

    pub fn text_to_metric_families(
        &mut self,
        reader: R,
    ) -> Result<HashMap<String, MetricFamily>, ParseError> {
        self.reset(reader);
        let mut next_state = self.start_of_line();
        while let Some(state) = next_state {
            next_state = state();
        }
        // Get rid of empty metric families.
        self.metric_families_by_name
            .retain(|_, mf| !mf.metric.is_empty());

        // If self.err is io::ErrorKind::UnexpectedEof now, we have run into a premature end of the input
        // stream. Turn this error into something nicer and more meaningful.
        if let Some(err) = &self.err {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                self.parse_error("unexpected end of input stream");
            }
        }

        if let Some(err) = self.err.take() {
            Err(err)
        } else {
            Ok(self.metric_families_by_name.clone())
        }
    }

    // fn reset(&mut self, reader: R) {
    //     self.buf = BufReader::new(reader);
    //     self.err = None;
    //     self.line_count = 0;
    //     self.current_byte = 0;
    //     self.current_token.clear();
    //     self.current_mf = None;
    //     self.current_metric = None;
    //     self.current_label_pair = None;
    //     self.current_label_pairs.clear();
    //     self.current_labels.clear();
    //     self.summaries.clear();
    //     self.current_quantile = 0.0;
    //     self.histograms.clear();
    //     self.current_bucket = 0.0;
    //     self.current_is_summary_count = false;
    //     self.current_is_summary_sum = false;
    //     self.current_is_histogram_count = false;
    //     self.current_is_histogram_sum = false;
    //     self.current_metric_is_inside_braces = false;
    //     self.current_metric_inside_braces_is_present = false;
    // }
    fn reset(&mut self, input: R) {
        self.metric_families_by_name = HashMap::new();
        self.buf = BufReader::new(input);
        self.err = None;
        self.line_count = 0;
        self.summaries.clear();
        self.histograms.clear();
        self.current_quantile = f64::NAN;
        self.current_bucket = f64::NAN;
        self.current_mf = None;
    }

    fn parse_error(&mut self, msg: &str) {
        self.err = Some(io::Error::new(io::ErrorKind::InvalidData, msg));
    }

    fn start_of_line(&mut self) -> Option<StateFn> {
        self.line_count += 1;
        self.current_metric_is_inside_braces = false;
        self.current_metric_inside_braces_is_present = false;

        if let Err(e) = self.skip_blank_tab() {
            // This is the only place that we expect to see io::ErrorKind::UnexpectedEof,
            // which is not an error but the signal that we are done.
            // Any other error that happens to align with the start of
            // a line is still an error.
            if e.kind() == io::ErrorKind::UnexpectedEof {
                self.err = None;
            } else {
                self.err = Some(e);
            }
            return None;
        }

        match self.current_byte {
            b'#' => Some(Self::start_comment),
            b'\n' => Some(Self::start_of_line), // Empty line, start the next one.
            b'{' => {
                self.current_metric_is_inside_braces = true;
                Some(Self::reading_labels)
            }
            _ => Some(Self::reading_metric_name),
        }
    }

    fn start_comment(&mut self) -> Option<StateFn> {
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_byte == b'\n' {
            return Some(Self::start_of_line);
        }
        if self.read_token_until_whitespace().is_err() {
            return None; // Unexpected end of input.
        }
        // If we have hit the end of line already, there is nothing left
        // to do. This is not considered a syntax error.
        if self.current_byte == b'\n' {
            return Some(Self::start_of_line);
        }
        let keyword = self.current_token.to_string();
        if keyword != "HELP" && keyword != "TYPE" {
            // Generic comment, ignore by fast forwarding to end of line.
            while self.current_byte != b'\n' {
                if let Err(e) = self
                    .buf
                    .read_exact(std::slice::from_mut(&mut self.current_byte))
                {
                    self.err = Some(e);
                    return None; // Unexpected end of input.
                }
            }
            return Some(Self::start_of_line);
        }
        // There is something. Next has to be a metric name.
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.read_token_as_metric_name().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_byte == b'\n' {
            // At the end of the line already.
            // Again, this is not considered a syntax error.
            return Some(Self::start_of_line);
        }
        if !is_blank_or_tab(self.current_byte) {
            self.parse_error("invalid metric name in comment");
            return None;
        }
        self.set_or_create_current_mf();
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_byte == b'\n' {
            // At the end of the line already.
            // Again, this is not considered a syntax error.
            return Some(Self::start_of_line);
        }
        match keyword.as_str() {
            "HELP" => Some(Self::reading_help),
            "TYPE" => Some(Self::reading_type),
            _ => panic!("code error: unexpected keyword {}", keyword),
        }
    }

    fn reading_metric_name(&mut self) -> Option<StateFn> {
        if self.read_token_as_metric_name().is_err() {
            return None;
        }
        if self.current_token.len() == 0 {
            self.parse_error("invalid metric name");
            return None;
        }
        self.set_or_create_current_mf();
        // Now is the time to fix the type if it hasn't happened yet.
        if self.current_mf.r#type.is_none() {
            self.current_mf.r#type = Some(dto::MetricType::Untyped);
        }
        self.current_metric = Some(dto::Metric::default());
        // Do not append the newly created current_metric to
        // current_mf.metrics right now. First wait if this is a summary,
        // and the metric exists already, which we can only know after
        // having read all the labels.
        if self.skip_blank_tab_if_current_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        Some(Self::reading_labels)
    }

    fn reading_labels(&mut self) -> Option<StateFn> {
        // Summaries/histograms are special. We have to reset the
        // current_labels map, current_quantile and current_bucket before starting to
        // read labels.
        if matches!(
            self.current_mf.r#type,
            Some(dto::MetricType::Summary) | Some(dto::MetricType::Histogram)
        ) {
            self.current_labels.clear();
            self.current_labels.insert(
                model::METRIC_NAME_LABEL.to_string(),
                self.current_mf.get_name().to_string(),
            );
            self.current_quantile = f64::NAN;
            self.current_bucket = f64::NAN;
        }
        if self.current_byte != b'{' {
            return Some(Self::reading_value);
        }
        Some(Self::start_label_name)
    }

    fn start_label_name(&mut self) -> Option<StateFn> {
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_byte == b'}' {
            self.current_metric
                .as_mut()?
                .label
                .extend(self.current_label_pairs.drain(..));
            if self.skip_blank_tab().is_err() {
                return None; // Unexpected end of input.
            }
            return Some(Self::reading_value);
        }
        if self.read_token_as_label_name().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_token.len() == 0 {
            self.parse_error(&format!(
                "invalid label name for metric {}",
                self.current_mf.get_name()
            ));
            return None;
        }
        if self.skip_blank_tab_if_current_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_byte != b'=' {
            if self.current_metric_is_inside_braces {
                if self.current_metric_inside_braces_is_present {
                    self.parse_error(&format!(
                        "multiple metric names for metric {}",
                        self.current_mf.get_name()
                    ));
                    return None;
                }
                match self.current_byte {
                    b',' => {
                        self.set_or_create_current_mf();
                        if self.current_mf.r#type.is_none() {
                            self.current_mf.r#type = Some(dto::MetricType::Untyped);
                        }
                        self.current_metric = Some(dto::Metric::default());
                        self.current_metric_inside_braces_is_present = true;
                        return Some(Self::start_label_name);
                    }
                    b'}' => {
                        self.set_or_create_current_mf();
                        if self.current_mf.r#type.is_none() {
                            self.current_mf.r#type = Some(dto::MetricType::Untyped);
                        }
                        self.current_metric = Some(dto::Metric::default());
                        self.current_metric
                            .as_mut()?
                            .label
                            .extend(self.current_label_pairs.drain(..));
                        if self.skip_blank_tab().is_err() {
                            return None; // Unexpected end of input.
                        }
                        return Some(Self::reading_value);
                    }
                    _ => {
                        self.parse_error(&format!(
                            "unexpected end of metric name {}",
                            self.current_byte
                        ));
                        return None;
                    }
                }
            }
            self.parse_error(&format!(
                "expected '=' after label name, found {}",
                self.current_byte
            ));
            self.current_label_pairs.clear();
            return None;
        }
        self.current_label_pair = Some(dto::LabelPair {
            name: Some(self.current_token.to_string()),
            ..Default::default()
        });
        if self.current_label_pair.as_ref()?.name.as_deref() == Some(model::METRIC_NAME_LABEL) {
            self.parse_error(&format!(
                "label name {} is reserved",
                model::METRIC_NAME_LABEL
            ));
            return None;
        }
        // Special summary/histogram treatment. Don't add 'quantile' and 'le'
        // labels to 'real' labels.
        if !(self.current_mf.get_type() == Some(dto::MetricType::Summary)
            && self.current_label_pair.as_ref()?.name.as_deref() == Some(model::QUANTILE_LABEL))
            && !(self.current_mf.get_type() == Some(dto::MetricType::Histogram)
                && self.current_label_pair.as_ref()?.name.as_deref() == Some(model::BUCKET_LABEL))
        {
            self.current_label_pairs
                .push(self.current_label_pair.clone().unwrap());
        }
        // Check for duplicate label names.
        let mut labels = std::collections::HashSet::new();
        for l in &self.current_label_pairs {
            let l_name = l.name.as_deref().unwrap();
            if !labels.insert(l_name) {
                self.parse_error(&format!(
                    "duplicate label names for metric {}",
                    self.current_mf.get_name()
                ));
                self.current_label_pairs.clear();
                return None;
            }
        }
        Some(Self::start_label_value)
    }

    fn start_label_value(&mut self) -> Option<StateFn> {
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.current_byte != b'"' {
            self.parse_error(&format!(
                "expected '\"' at start of label value, found {}",
                self.current_byte
            ));
            return None;
        }
        if self.read_token_as_label_value().is_err() {
            return None;
        }
        if !model::LabelValue::new(self.current_token.to_string()).is_valid() {
            self.parse_error(&format!(
                "invalid label value {}",
                self.current_token.to_string()
            ));
            return None;
        }
        self.current_label_pair.as_mut()?.value = Some(self.current_token.to_string());
        // Special treatment of summaries:
        // - Quantile labels are special, will result in dto::Quantile later.
        // - Other labels have to be added to current_labels for signature calculation.
        if self.current_mf.get_type() == Some(dto::MetricType::Summary) {
            if self.current_label_pair.as_ref()?.name.as_deref() == Some(model::QUANTILE_LABEL) {
                if let Err(e) = self
                    .current_label_pair
                    .as_ref()?
                    .value
                    .as_deref()
                    .unwrap()
                    .parse::<f64>()
                {
                    // Create a more helpful error message.
                    self.parse_error(&format!(
                        "expected float as value for 'quantile' label, got {}",
                        self.current_label_pair.as_ref()?.value.as_deref().unwrap()
                    ));
                    self.current_label_pairs.clear();
                    return None;
                }
                self.current_quantile = self
                    .current_label_pair
                    .as_ref()?
                    .value
                    .as_deref()
                    .unwrap()
                    .parse()
                    .unwrap();
            } else {
                self.current_labels.insert(
                    self.current_label_pair.as_ref()?.name.clone().unwrap(),
                    self.current_label_pair.as_ref()?.value.clone().unwrap(),
                );
            }
        }
        // Similar special treatment of histograms.
        if self.current_mf.get_type() == Some(dto::MetricType::Histogram) {
            if self.current_label_pair.as_ref()?.name.as_deref() == Some(model::BUCKET_LABEL) {
                if let Err(e) = self
                    .current_label_pair
                    .as_ref()?
                    .value
                    .as_deref()
                    .unwrap()
                    .parse::<f64>()
                {
                    // Create a more helpful error message.
                    self.parse_error(&format!(
                        "expected float as value for 'le' label, got {}",
                        self.current_label_pair.as_ref()?.value.as_deref().unwrap()
                    ));
                    return None;
                }
                self.current_bucket = self
                    .current_label_pair
                    .as_ref()?
                    .value
                    .as_deref()
                    .unwrap()
                    .parse()
                    .unwrap();
            } else {
                self.current_labels.insert(
                    self.current_label_pair.as_ref()?.name.clone().unwrap(),
                    self.current_label_pair.as_ref()?.value.clone().unwrap(),
                );
            }
        }
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        match self.current_byte {
            b',' => Some(Self::start_label_name),
            b'}' => {
                if self.current_mf.is_none() {
                    self.parse_error("invalid metric name");
                    return None;
                }
                self.current_metric
                    .as_mut()?
                    .label
                    .extend(self.current_label_pairs.drain(..));
                if self.skip_blank_tab().is_err() {
                    return None; // Unexpected end of input.
                }
                Some(Self::reading_value)
            }
            _ => {
                self.parse_error(&format!(
                    "unexpected end of label value {}",
                    self.current_label_pair.as_ref()?.value.as_deref().unwrap()
                ));
                self.current_label_pairs.clear();
                None
            }
        }
    }

    fn reading_value(&mut self) -> Option<StateFn> {
        // When we are here, we have read all the labels, so for the
        // special case of a summary/histogram, we can finally find out
        // if the metric already exists.
        if let Some(dto::MetricType::Summary) = self.current_mf.get_type() {
            let signature = model::labels_to_signature(&self.current_labels);
            if let Some(summary) = self.summaries.get(&signature) {
                self.current_metric = Some(summary.clone());
            } else {
                self.summaries
                    .insert(signature, self.current_metric.clone().unwrap());
                self.current_mf
                    .metric
                    .push(self.current_metric.clone().unwrap());
            }
        } else if let Some(dto::MetricType::Histogram) = self.current_mf.get_type() {
            let signature = model::labels_to_signature(&self.current_labels);
            if let Some(histogram) = self.histograms.get(&signature) {
                self.current_metric = Some(histogram.clone());
            } else {
                self.histograms
                    .insert(signature, self.current_metric.clone().unwrap());
                self.current_mf
                    .metric
                    .push(self.current_metric.clone().unwrap());
            }
        } else {
            self.current_mf
                .metric
                .push(self.current_metric.clone().unwrap());
        }

        if self.read_token_until_whitespace().is_err() {
            return None; // Unexpected end of input.
        }

        let value: f64 = match self.current_token.parse() {
            Ok(v) => v,
            Err(_) => {
                self.parse_error(&format!(
                    "expected float as value, got {}",
                    self.current_token
                ));
                return None;
            }
        };

        match self.current_mf.get_type() {
            Some(dto::MetricType::Counter) => {
                self.current_metric.as_mut()?.counter = Some(dto::Counter { value: Some(value) });
            }
            Some(dto::MetricType::Gauge) => {
                self.current_metric.as_mut()?.gauge = Some(dto::Gauge { value: Some(value) });
            }
            Some(dto::MetricType::Untyped) => {
                self.current_metric.as_mut()?.untyped = Some(dto::Untyped { value: Some(value) });
            }
            Some(dto::MetricType::Summary) => {
                if self.current_metric.as_mut()?.summary.is_none() {
                    self.current_metric.as_mut()?.summary = Some(dto::Summary::default());
                }
                if self.current_is_summary_count {
                    self.current_metric.as_mut()?.summary.as_mut()?.sample_count =
                        Some(value as u64);
                } else if self.current_is_summary_sum {
                    self.current_metric.as_mut()?.summary.as_mut()?.sample_sum = Some(value);
                } else if !self.current_quantile.is_nan() {
                    self.current_metric
                        .as_mut()?
                        .summary
                        .as_mut()?
                        .quantile
                        .push(dto::Quantile {
                            quantile: Some(self.current_quantile),
                            value: Some(value),
                        });
                }
            }
            Some(dto::MetricType::Histogram) => {
                if self.current_metric.as_mut()?.histogram.is_none() {
                    self.current_metric.as_mut()?.histogram = Some(dto::Histogram::default());
                }
                if self.current_is_histogram_count {
                    self.current_metric
                        .as_mut()?
                        .histogram
                        .as_mut()?
                        .sample_count = Some(value as u64);
                } else if self.current_is_histogram_sum {
                    self.current_metric.as_mut()?.histogram.as_mut()?.sample_sum = Some(value);
                } else if !self.current_bucket.is_nan() {
                    self.current_metric
                        .as_mut()?
                        .histogram
                        .as_mut()?
                        .bucket
                        .push(dto::Bucket {
                            upper_bound: Some(self.current_bucket),
                            cumulative_count: Some(value as u64),
                        });
                }
            }
            _ => {
                self.err = Some(format!(
                    "unexpected type for metric name {}",
                    self.current_mf.get_name()
                ));
            }
        }

        if self.current_byte == b'\n' {
            return Some(Self::start_of_line);
        }
        Some(Self::start_timestamp)
    }

    fn start_timestamp(&mut self) -> Option<StateFn> {
        if self.skip_blank_tab().is_err() {
            return None; // Unexpected end of input.
        }
        if self.read_token_until_whitespace().is_err() {
            return None; // Unexpected end of input.
        }
        let timestamp: i64 = match self.current_token.parse() {
            Ok(t) => t,
            Err(_) => {
                self.parse_error(&format!(
                    "expected integer as timestamp, got {}",
                    self.current_token
                ));
                return None;
            }
        };
        self.current_metric.as_mut()?.timestamp_ms = Some(timestamp);
        if self.read_token_until_newline(false).is_err() {
            return None; // Unexpected end of input.
        }
        if !self.current_token.is_empty() {
            self.parse_error(&format!(
                "spurious string after timestamp: {}",
                self.current_token
            ));
            return None;
        }
        Some(Self::start_of_line)
    }

    fn reading_help(&mut self) -> Option<StateFn> {
        if self.current_mf.help.is_some() {
            self.parse_error(&format!(
                "second HELP line for metric name {}",
                self.current_mf.get_name()
            ));
            return None;
        }
        // Rest of line is the docstring.
        if self.read_token_until_newline(true).is_err() {
            return None; // Unexpected end of input.
        }
        self.current_mf.help = Some(self.current_token.to_string());
        Some(Self::start_of_line)
    }

    fn reading_type(&mut self) -> Option<StateFn> {
        if self.current_mf.r#type.is_some() {
            self.parse_error(&format!(
                "second TYPE line for metric name {}, or TYPE reported after samples",
                self.current_mf.get_name()
            ));
            return None;
        }
        // Rest of line is the type.
        if self.read_token_until_newline(false).is_err() {
            return None; // Unexpected end of input.
        }
        let metric_type_str = self.current_token.to_uppercase();
        let metric_type = match dto::MetricType::from_str(&metric_type_str) {
            Ok(mt) => mt,
            Err(_) => {
                self.parse_error(&format!("unknown metric type {}", self.current_token));
                return None;
            }
        };
        self.current_mf.r#type = Some(metric_type);
        Some(Self::start_of_line)
    }

    fn parse_error(&mut self, msg: &str) {
        self.err = Some(ParseError {
            line: self.line_count,
            msg: msg.to_string(),
        });
    }

    fn skip_blank_tab(&mut self) -> Result<(), io::Error> {
        loop {
            self.current_byte = match self.buf.read_u8() {
                Ok(byte) => byte,
                Err(e) => return Err(e),
            };
            if !is_blank_or_tab(self.current_byte) {
                return Ok(());
            }
        }
    }

    fn skip_blank_tab_if_current_blank_tab(&mut self) -> Result<(), io::Error> {
        if is_blank_or_tab(self.current_byte) {
            self.skip_blank_tab()?;
        }
        Ok(())
    }

    fn read_token_until_whitespace(&mut self) -> Result<(), io::Error> {
        self.current_token.clear();
        while self.err.is_none()
            && !is_blank_or_tab(self.current_byte)
            && self.current_byte != b'\n'
        {
            self.current_token.push(self.current_byte);
            self.current_byte = self.buf.read_u8()?;
        }
        Ok(())
    }

    fn read_token_until_newline(
        &mut self,
        recognize_escape_sequence: bool,
    ) -> Result<(), io::Error> {
        self.current_token.clear();
        let mut escaped = false;
        while self.err.is_none() {
            if recognize_escape_sequence && escaped {
                match self.current_byte {
                    b'\\' => self.current_token.push(b'\\'),
                    b'n' => self.current_token.push(b'\n' as u8),
                    b'"' => self.current_token.push(b'"'),
                    _ => {
                        self.parse_error(&format!(
                            "invalid escape sequence '\\{}'",
                            self.current_byte as char
                        ));
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid escape sequence",
                        ));
                    }
                }
                escaped = false;
            } else {
                match self.current_byte {
                    b'\n' => return Ok(()),
                    b'\\' => escaped = true,
                    _ => self.current_token.push(self.current_byte),
                }
            }
            self.current_byte = self.buf.read_u8()?;
        }
        Ok(())
    }

    fn read_token_as_metric_name(&mut self) -> Result<(), io::Error> {
        self.current_token.clear();
        // A UTF-8 metric name must be quoted and may have escaped characters.
        let mut quoted = false;
        let mut escaped = false;
        if !is_valid_metric_name_start(self.current_byte) {
            return Ok(());
        }
        while self.err.is_none() {
            if escaped {
                match self.current_byte {
                    b'\\' => self.current_token.push(b'\\'),
                    b'n' => self.current_token.push(b'\n' as u8),
                    b'"' => self.current_token.push(b'"'),
                    _ => {
                        self.parse_error(&format!(
                            "invalid escape sequence '\\{}'",
                            self.current_byte as char
                        ));
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid escape sequence",
                        ));
                    }
                }
                escaped = false;
            } else {
                match self.current_byte {
                    b'"' => {
                        quoted = !quoted;
                        if !quoted {
                            self.current_byte = self.buf.read_u8()?;
                            return Ok(());
                        }
                    }
                    b'\n' => {
                        self.parse_error(&format!(
                            "metric name {} contains unescaped new-line",
                            String::from_utf8_lossy(&self.current_token)
                        ));
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "unescaped new-line in metric name",
                        ));
                    }
                    b'\\' => escaped = true,
                    _ => self.current_token.push(self.current_byte),
                }
            }
            self.current_byte = self.buf.read_u8()?;
            if !is_valid_metric_name_continuation(self.current_byte, quoted)
                || (!quoted && self.current_byte == b' ')
            {
                return Ok(());
            }
        }
        Ok(())
    }

    fn read_token_as_label_name(&mut self) -> Result<(), io::Error> {
        self.current_token.clear();
        // A UTF-8 label name must be quoted and may have escaped characters.
        let mut quoted = false;
        let mut escaped = false;
        if !is_valid_label_name_start(self.current_byte) {
            return Ok(());
        }
        while self.err.is_none() {
            if escaped {
                match self.current_byte {
                    b'\\' => self.current_token.push(b'\\'),
                    b'n' => self.current_token.push(b'\n' as u8),
                    b'"' => self.current_token.push(b'"'),
                    _ => {
                        self.parse_error(&format!(
                            "invalid escape sequence '\\{}'",
                            self.current_byte as char
                        ));
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid escape sequence",
                        ));
                    }
                }
                escaped = false;
            } else {
                match self.current_byte {
                    b'"' => {
                        quoted = !quoted;
                        if !quoted {
                            self.current_byte = self.buf.read_u8()?;
                            return Ok(());
                        }
                    }
                    b'\n' => {
                        self.parse_error(&format!(
                            "label name {} contains unescaped new-line",
                            String::from_utf8_lossy(&self.current_token)
                        ));
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "unescaped new-line in label name",
                        ));
                    }
                    b'\\' => escaped = true,
                    _ => self.current_token.push(self.current_byte),
                }
            }
            self.current_byte = self.buf.read_u8()?;
            if !is_valid_label_name_continuation(self.current_byte, quoted)
                || (!quoted && self.current_byte == b'=')
            {
                return Ok(());
            }
        }
        Ok(())
    }

    fn read_token_as_label_value(&mut self) -> Result<(), io::Error> {
        self.current_token.clear();
        let mut escaped = false;
        loop {
            self.current_byte = match self.buf.read_u8() {
                Ok(byte) => byte,
                Err(e) => return Err(e),
            };
            if escaped {
                match self.current_byte {
                    b'"' | b'\\' => self.current_token.push(self.current_byte),
                    b'n' => self.current_token.push(b'\n'),
                    _ => {
                        self.parse_error(&format!(
                            "invalid escape sequence '\\{}'",
                            self.current_byte as char
                        ));
                        self.current_label_pairs.clear();
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid escape sequence",
                        ));
                    }
                }
                escaped = false;
                continue;
            }
            match self.current_byte {
                b'"' => return Ok(()),
                b'\n' => {
                    self.parse_error(&format!(
                        "label value {} contains unescaped new-line",
                        String::from_utf8_lossy(&self.current_token)
                    ));
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unescaped new-line in label value",
                    ));
                }
                b'\\' => escaped = true,
                _ => self.current_token.push(self.current_byte),
            }
        }
    }

    fn set_or_create_current_mf(&mut self) {
        self.current_is_summary_count = false;
        self.current_is_summary_sum = false;
        self.current_is_histogram_count = false;
        self.current_is_histogram_sum = false;
        let name = self.current_token.to_string();
        if let Some(mf) = self.metric_families_by_name.get(&name) {
            self.current_mf = Some(mf.clone());
            return;
        }
        // Try out if this is a _sum or _count for a summary/histogram.
        let summary_name = summary_metric_name(&name);
        if let Some(mf) = self.metric_families_by_name.get(&summary_name) {
            if mf.get_type() == Some(dto::MetricType::Summary) {
                self.current_mf = Some(mf.clone());
                if is_count(&name) {
                    self.current_is_summary_count = true;
                }
                if is_sum(&name) {
                    self.current_is_summary_sum = true;
                }
                return;
            }
        }
        let histogram_name = histogram_metric_name(&name);
        if let Some(mf) = self.metric_families_by_name.get(&histogram_name) {
            if mf.get_type() == Some(dto::MetricType::Histogram) {
                self.current_mf = Some(mf.clone());
                if is_count(&name) {
                    self.current_is_histogram_count = true;
                }
                if is_sum(&name) {
                    self.current_is_histogram_sum = true;
                }
                return;
            }
        }
        let new_mf = dto::MetricFamily {
            name: Some(name.clone()),
            ..Default::default()
        };
        self.metric_families_by_name
            .insert(name.clone(), new_mf.clone());
        self.current_mf = Some(new_mf);
    }
}

fn is_valid_label_name_start(b: u8) -> bool {
    (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z') || b == b'_' || b == b'"'
}

fn is_valid_label_name_continuation(b: u8, quoted: bool) -> bool {
    is_valid_label_name_start(b)
        || (b >= b'0' && b <= b'9')
        || (quoted && std::str::from_utf8(&[b]).is_ok())
}

fn is_valid_metric_name_start(b: u8) -> bool {
    is_valid_label_name_start(b) || b == b':'
}

fn is_valid_metric_name_continuation(b: u8, quoted: bool) -> bool {
    is_valid_label_name_continuation(b, quoted) || b == b':'
}

fn is_blank_or_tab(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn is_count(name: &str) -> bool {
    name.len() > 6 && &name[name.len() - 6..] == "_count"
}

fn is_sum(name: &str) -> bool {
    name.len() > 4 && &name[name.len() - 4..] == "_sum"
}

fn is_bucket(name: &str) -> bool {
    name.len() > 7 && &name[name.len() - 7..] == "_bucket"
}

fn summary_metric_name(name: &str) -> String {
    if is_count(name) {
        name[..name.len() - 6].to_string()
    } else if is_sum(name) {
        name[..name.len() - 4].to_string()
    } else {
        name.to_string()
    }
}

fn histogram_metric_name(name: &str) -> String {
    if is_count(name) {
        name[..name.len() - 6].to_string()
    } else if is_sum(name) {
        name[..name.len() - 4].to_string()
    } else if is_bucket(name) {
        name[..name.len() - 7].to_string()
    } else {
        name.to_string()
    }
}

fn parse_float(s: &str) -> Result<f64, &'static str> {
    if s.contains('p') || s.contains('P') || s.contains('_') {
        Err("unsupported character in float")
    } else {
        s.parse::<f64>().map_err(|_| "invalid float")
    }
}
