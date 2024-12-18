struct EncoderOption {
    with_created_lines: bool,
    with_unit: bool,
}

impl EncoderOption {
    fn new() -> Self {
        EncoderOption {
            with_created_lines: false,
            with_unit: false,
        }
    }
}

type EncoderOption = Box<dyn Fn(&mut EncoderOption)>;

fn with_created_lines() -> EncoderOption {
    Box::new(|t: &mut EncoderOption| {
        t.with_created_lines = true;
    })
}

fn with_unit() -> EncoderOption {
    Box::new(|t: &mut EncoderOption| {
        t.with_unit = true;
    })
}

fn metric_family_to_open_metrics(
    out: &mut dyn Write,
    in_: &ProtoMetricFamily,
    options: Vec<EncoderOptionFn>,
) -> io::Result<usize> {
    let mut to_om = EncoderOption::new();
    for option in options {
        option(&mut to_om);
    }

    let name = in_.get_name();
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "MetricFamily has no name",
        ));
    }

    let mut written = 0;
    let mut compliant_name = name.to_string();
    let metric_type = in_.get_field_type();

    if metric_type == ProtoMetricType::COUNTER && compliant_name.ends_with("_total") {
        compliant_name.truncate(compliant_name.len() - 6);
    }

    if to_om.with_unit
        && in_.unit.is_some()
        && !compliant_name.ends_with(&format!("_{}", in_.unit.as_ref().unwrap()))
    {
        compliant_name.push_str(&format!("_{}", in_.unit.as_ref().unwrap()));
    }

    if let Some(help) = &in_.help {
        written += writeln!(
            out,
            "# HELP {} {}",
            compliant_name,
            escape_string(help, true)
        )?;
    }

    written += writeln!(
        out,
        "# TYPE {} {}",
        compliant_name,
        metric_type_to_str(metric_type)
    )?;

    if to_om.with_unit && in_.unit.is_some() {
        written += writeln!(
            out,
            "# UNIT {} {}",
            compliant_name,
            escape_string(in_.unit.as_ref().unwrap(), true)
        )?;
    }

    for metric in &in_.metric {
        match metric_type {
            ProtoMetricType::COUNTER => {
                if let Some(counter) = &metric.counter {
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "",
                        metric,
                        "",
                        0.0,
                        counter.value,
                        0,
                        false,
                        counter.exemplar.as_ref(),
                    )?;
                    if to_om.with_created_lines && counter.created_timestamp.is_some() {
                        written += write_open_metrics_created(
                            out,
                            &compliant_name,
                            "_total",
                            metric,
                            "",
                            0.0,
                            counter.created_timestamp.as_ref().unwrap(),
                        )?;
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("expected counter in metric {} {}", compliant_name, metric),
                    ));
                }
            }
            ProtoMetricType::GAUGE => {
                if let Some(gauge) = &metric.gauge {
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "",
                        metric,
                        "",
                        0.0,
                        gauge.value,
                        0,
                        false,
                        None,
                    )?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("expected gauge in metric {} {}", compliant_name, metric),
                    ));
                }
            }
            ProtoMetricType::UNTYPED => {
                if let Some(untyped) = &metric.untyped {
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "",
                        metric,
                        "",
                        0.0,
                        untyped.value,
                        0,
                        false,
                        None,
                    )?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("expected untyped in metric {} {}", compliant_name, metric),
                    ));
                }
            }
            ProtoMetricType::SUMMARY => {
                if let Some(summary) = &metric.summary {
                    for quantile in &summary.quantile {
                        written += write_open_metrics_sample(
                            out,
                            &compliant_name,
                            "",
                            metric,
                            "quantile",
                            quantile.quantile,
                            quantile.value,
                            0,
                            false,
                            None,
                        )?;
                    }
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "_sum",
                        metric,
                        "",
                        0.0,
                        summary.sample_sum,
                        0,
                        false,
                        None,
                    )?;
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "_count",
                        metric,
                        "",
                        0.0,
                        0.0,
                        summary.sample_count,
                        true,
                        None,
                    )?;
                    if to_om.with_created_lines && summary.created_timestamp.is_some() {
                        written += write_open_metrics_created(
                            out,
                            &compliant_name,
                            "",
                            metric,
                            "",
                            0.0,
                            summary.created_timestamp.as_ref().unwrap(),
                        )?;
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("expected summary in metric {} {}", compliant_name, metric),
                    ));
                }
            }
            ProtoMetricType::HISTOGRAM => {
                if let Some(histogram) = &metric.histogram {
                    let mut inf_seen = false;
                    for bucket in &histogram.bucket {
                        written += write_open_metrics_sample(
                            out,
                            &compliant_name,
                            "_bucket",
                            metric,
                            "le",
                            bucket.upper_bound,
                            0.0,
                            bucket.cumulative_count,
                            true,
                            bucket.exemplar.as_ref(),
                        )?;
                        if bucket.upper_bound.is_infinite() {
                            inf_seen = true;
                        }
                    }
                    if !inf_seen {
                        written += write_open_metrics_sample(
                            out,
                            &compliant_name,
                            "_bucket",
                            metric,
                            "le",
                            f64::INFINITY,
                            0.0,
                            histogram.sample_count,
                            true,
                            None,
                        )?;
                    }
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "_sum",
                        metric,
                        "",
                        0.0,
                        histogram.sample_sum,
                        0,
                        false,
                        None,
                    )?;
                    written += write_open_metrics_sample(
                        out,
                        &compliant_name,
                        "_count",
                        metric,
                        "",
                        0.0,
                        0.0,
                        histogram.sample_count,
                        true,
                        None,
                    )?;
                    if to_om.with_created_lines && histogram.created_timestamp.is_some() {
                        written += write_open_metrics_created(
                            out,
                            &compliant_name,
                            "",
                            metric,
                            "",
                            0.0,
                            histogram.created_timestamp.as_ref().unwrap(),
                        )?;
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("expected histogram in metric {} {}", compliant_name, metric),
                    ));
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unexpected type in metric {} {}", compliant_name, metric),
                ));
            }
        }
    }

    Ok(written)
}

fn write_open_metrics_sample(
    out: &mut dyn Write,
    name: &str,
    suffix: &str,
    metric: &ProtoMetricFamily,
    additional_label_name: &str,
    additional_label_value: f64,
    float_value: f64,
    int_value: u64,
    use_int_value: bool,
    exemplar: Option<&ProtoMetricFamily>,
) -> io::Result<usize> {
    let mut written = 0;
    written += write_open_metrics_name_and_label_pairs(
        out,
        &format!("{}{}", name, suffix),
        &metric.label,
        additional_label_name,
        additional_label_value,
    )?;
    written += write!(out, " ")?;
    if use_int_value {
        written += write!(out, "{}", int_value)?;
    } else {
        written += write_open_metrics_float(out, float_value)?;
    }
    if let Some(timestamp) = &metric.timestamp_ms {
        written += write!(out, " ")?;
        written += write_open_metrics_float(out, *timestamp as f64 / 1000.0)?;
    }
    if let Some(exemplar) = exemplar {
        written += write_exemplar(out, exemplar)?;
    }
    written += write!(out, "\n")?;
    Ok(written)
}

fn write_open_metrics_name_and_label_pairs(
    out: &mut dyn Write,
    name: &str,
    labels: &[ProtoMetricFamily],
    additional_label_name: &str,
    additional_label_value: f64,
) -> io::Result<usize> {
    let mut written = 0;
    let mut separator = '{';

    if !name.is_empty() {
        if !is_valid_legacy_metric_name(name) {
            written += write!(out, "{}\"{}\"", separator, name)?;
            separator = ',';
        } else {
            written += write!(out, "{}", name)?;
        }
    }

    for label in labels {
        written += write!(
            out,
            "{}{}=\"{}\"",
            separator,
            label.get_name(),
            escape_string(label.get_value(), true)
        )?;
        separator = ',';
    }

    if !additional_label_name.is_empty() {
        written += write!(
            out,
            "{}{}=\"{}\"",
            separator, additional_label_name, additional_label_value
        )?;
    }

    if separator == ',' {
        written += write!(out, "}}")?;
    }

    Ok(written)
}

fn write_open_metrics_created(
    out: &mut dyn Write,
    name: &str,
    suffix_to_trim: &str,
    metric: &ProtoMetricFamily,
    additional_label_name: &str,
    additional_label_value: f64,
    created_timestamp: &Timestamp,
) -> io::Result<usize> {
    let mut written = 0;
    written += write_open_metrics_name_and_label_pairs(
        out,
        &format!("{}_created", name.trim_end_matches(suffix_to_trim)),
        &metric.label,
        additional_label_name,
        additional_label_value,
    )?;
    written += write!(out, " ")?;
    written += write_open_metrics_float(
        out,
        created_timestamp.seconds as f64 + created_timestamp.nanos as f64 / 1e9,
    )?;
    written += write!(out, "\n")?;
    Ok(written)
}

fn write_exemplar(out: &mut dyn Write, exemplar: &ProtoMetricFamily) -> io::Result<usize> {
    let mut written = 0;
    written += write!(out, " # ")?;
    written += write_open_metrics_name_and_label_pairs(out, "", &exemplar.label, "", 0.0)?;
    written += write!(out, " ")?;
    written += write_open_metrics_float(out, exemplar.value)?;
    if let Some(timestamp) = &exemplar.timestamp {
        written += write!(out, " ")?;
        written +=
            write_open_metrics_float(out, timestamp.seconds as f64 + timestamp.nanos as f64 / 1e9)?;
    }
    Ok(written)
}

fn write_open_metrics_float(out: &mut dyn Write, value: f64) -> io::Result<usize> {
    match value {
        1.0 => write!(out, "1.0"),
        0.0 => write!(out, "0.0"),
        -1.0 => write!(out, "-1.0"),
        f if f.is_nan() => write!(out, "NaN"),
        f if f.is_infinite() && f.is_sign_positive() => write!(out, "+Inf"),
        f if f.is_infinite() && f.is_sign_negative() => write!(out, "-Inf"),
        _ => {
            let mut buf = itoa::Buffer::new();
            let s = buf.format(value);
            if !s.contains('.') && !s.contains('e') {
                write!(out, "{}.0", s)
            } else {
                write!(out, "{}", s)
            }
        }
    }
}

fn escape_string(s: &str, escape_double_quote: bool) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '\\' => escaped.push_str(r"\\"),
            '\n' => escaped.push_str(r"\n"),
            '"' if escape_double_quote => escaped.push_str(r#"\""#),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn metric_type_to_str(metric_type: ProtoMetricType) -> &'static str {
    match metric_type {
        ProtoMetricType::COUNTER => "counter",
        ProtoMetricType::GAUGE => "gauge",
        ProtoMetricType::SUMMARY => "summary",
        ProtoMetricType::UNTYPED => "unknown",
        ProtoMetricType::HISTOGRAM => "histogram",
        _ => "unknown",
    }
}

fn is_valid_legacy_metric_name(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
}

fn write_open_metrics_created(
    out: &mut dyn Write,
    name: &str,
    suffix_to_trim: &str,
    metric: &ProtoMetricFamily,
    additional_label_name: &str,
    additional_label_value: f64,
    created_timestamp: &Timestamp,
) -> io::Result<usize> {
    let mut written = 0;
    written += write_open_metrics_name_and_label_pairs(
        out,
        &format!("{}_created", name.trim_end_matches(suffix_to_trim)),
        &metric.label,
        additional_label_name,
        additional_label_value,
    )?;
    written += write!(out, " ")?;
    written += write_open_metrics_float(
        out,
        created_timestamp.seconds as f64 + created_timestamp.nanos as f64 / 1e9,
    )?;
    written += write!(out, "\n")?;
    Ok(written)
}

fn write_exemplar(out: &mut dyn Write, exemplar: &ProtoMetricFamily) -> io::Result<usize> {
    let mut written = 0;
    written += write!(out, " # ")?;
    written += write_open_metrics_name_and_label_pairs(out, "", &exemplar.label, "", 0.0)?;
    written += write!(out, " ")?;
    written += write_open_metrics_float(out, exemplar.value)?;
    if let Some(timestamp) = &exemplar.timestamp {
        written += write!(out, " ")?;
        written +=
            write_open_metrics_float(out, timestamp.seconds as f64 + timestamp.nanos as f64 / 1e9)?;
    }
    Ok(written)
}

fn write_open_metrics_float(out: &mut dyn Write, value: f64) -> io::Result<usize> {
    match value {
        1.0 => write!(out, "1.0"),
        0.0 => write!(out, "0.0"),
        -1.0 => write!(out, "-1.0"),
        f if f.is_nan() => write!(out, "NaN"),
        f if f.is_infinite() && f.is_sign_positive() => write!(out, "+Inf"),
        f if f.is_infinite() && f.is_sign_negative() => write!(out, "-Inf"),
        _ => {
            let mut buf = ryu::Buffer::new();
            let s = buf.format(value);
            if !s.contains('.') && !s.contains('e') {
                write!(out, "{}.0", s)
            } else {
                write!(out, "{}", s)
            }
        }
    }
}

fn write_uint(out: &mut dyn Write, value: u64) -> io::Result<usize> {
    let mut buf = itoa::Buffer::new();
    let s = buf.format(value);
    write!(out, "{}", s)
}
