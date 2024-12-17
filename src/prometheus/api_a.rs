use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use serde_json::de::Deserializer;
use serde_json::ser::Serializer;
use serde_json::Value;
use std::ptr::NonNull;
use std::str::FromStr;

fn init() {
    serde_json::ser::Serializer::with_formatter(
        "model.SamplePair",
        marshal_sample_pair_json,
        marshal_json_is_empty,
    );
    serde_json::de::Deserializer::with_formatter("model.SamplePair", unmarshal_sample_pair_json);
    serde_json::ser::Serializer::with_formatter(
        "model.SampleHistogramPair",
        marshal_sample_histogram_pair_json,
        marshal_json_is_empty,
    );
    serde_json::de::Deserializer::with_formatter(
        "model.SampleHistogramPair",
        unmarshal_sample_histogram_pair_json,
    );
    serde_json::ser::Serializer::with_formatter(
        "model.SampleStream",
        marshal_sample_stream_json,
        marshal_json_is_empty,
    ); // Only needed for benchmark.
    serde_json::de::Deserializer::with_formatter(
        "model.SampleStream",
        unmarshal_sample_stream_json,
    ); // Only needed for benchmark.
}

// Placeholder functions for the JSON (de)serialization
fn marshal_sample_pair_json<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Implementation here
    unimplemented!()
}

fn unmarshal_sample_pair_json(ptr: NonNull<model::SamplePair>, iter: &mut serde_json::Value) {
    let p = unsafe { ptr.as_mut() };

    if !iter.is_array() {
        eprintln!("unmarshal model.SamplePair: SamplePair must be [timestamp, value]");
        return;
    }

    let array = iter.as_array().unwrap();
    if array.len() != 2 {
        eprintln!("unmarshal model.SamplePair: SamplePair must be [timestamp, value]");
        return;
    }

    let t = array[0].as_str().unwrap_or("");
    if let Err(err) = p.timestamp.unmarshal_json(t.as_bytes()) {
        eprintln!("unmarshal model.SamplePair: {}", err);
        return;
    }

    let f = array[1].as_str().unwrap_or("");
    match f64::from_str(f) {
        Ok(value) => p.value = model::SampleValue(value),
        Err(err) => {
            eprintln!("unmarshal model.SamplePair: {}", err);
            return;
        }
    }
}

fn marshal_sample_histogram_pair_json(
    ptr: NonNull<model::SampleHistogramPair>,
    serializer: &mut Serializer,
) -> Result<(), serde_json::Error> {
    let p = unsafe { ptr.as_ref() };
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(&p.timestamp)?;
    seq.serialize_element(&p.histogram)?;
    seq.end()
}

fn unmarshal_sample_histogram_pair_json(ptr: NonNull<model::SampleHistogramPair>, iter: &Value) {
    let p = unsafe { ptr.as_mut() };

    if !iter.is_array() {
        eprintln!("unmarshal model.SampleHistogramPair: SampleHistogramPair must be [timestamp, {{histogram}}]");
        return;
    }

    let array = iter.as_array().unwrap();
    if array.len() != 2 {
        eprintln!("unmarshal model.SampleHistogramPair: SampleHistogramPair must be [timestamp, {{histogram}}]");
        return;
    }

    let t = array[0].as_str().unwrap_or("");
    if let Err(err) = p.timestamp.unmarshal_json(t.as_bytes()) {
        eprintln!("unmarshal model.SampleHistogramPair: {}", err);
        return;
    }

    let histogram = array[1].as_object().unwrap();
    let mut h = model::SampleHistogram::default();
    p.histogram = Some(h);

    for (key, value) in histogram.iter() {
        match key.as_str() {
            "count" => {
                let f = value.as_str().unwrap_or("");
                match f64::from_str(f) {
                    Ok(val) => h.count = model::FloatString(val),
                    Err(err) => {
                        eprintln!("unmarshal model.SampleHistogramPair: count of histogram is not a float: {}", err);
                        return;
                    }
                }
            }
            "sum" => {
                let f = value.as_str().unwrap_or("");
                match f64::from_str(f) {
                    Ok(val) => h.sum = model::FloatString(val),
                    Err(err) => {
                        eprintln!("unmarshal model.SampleHistogramPair: sum of histogram is not a float: {}", err);
                        return;
                    }
                }
            }
            "buckets" => {
                let buckets = value.as_array().unwrap();
                for bucket in buckets {
                    match unmarshal_histogram_bucket(bucket) {
                        Ok(b) => h.buckets.push(b),
                        Err(err) => {
                            eprintln!("unmarshal model.HistogramBucket: {}", err);
                            return;
                        }
                    }
                }
            }
            _ => {
                eprintln!(
                    "unmarshal model.SampleHistogramPair: unexpected key in histogram: {}",
                    key
                );
                return;
            }
        }
    }

    if array.len() > 2 {
        eprintln!("unmarshal model.SampleHistogramPair: SampleHistogramPair has too many values, must be [timestamp, {{histogram}}]");
        return;
    }
}

fn unmarshal_histogram_bucket(value: &Value) -> Result<model::HistogramBucket, String> {
    // Implementation here
    unimplemented!()
}
fn marshal_sample_stream_json<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Implementation here
    unimplemented!()
}

fn unmarshal_sample_stream_json<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    // Implementation here
    unimplemented!()
}

fn marshal_json_is_empty<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Implementation here
    unimplemented!()
}

fn marshal_sample_pair_json(
    ptr: NonNull<model::SamplePair>,
    serializer: &mut Serializer,
) -> Result<(), serde_json::Error> {
    let p = unsafe { ptr.as_ref() };
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(&p.timestamp)?;
    seq.serialize_element(&(p.value.0 as f64))?;
    seq.end()
}
