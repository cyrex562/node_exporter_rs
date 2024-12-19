use std::collections::HashMap;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use hyper::Request;
use prometheus::Labels;
use std::sync::Mutex;

pub trait Option {
    fn apply(&self, opts: &mut Options);
}

pub type LabelValueFromCtx = Arc<dyn Fn(&Request<hyper::Body>) -> String + Send + Sync>;

pub struct Options {
    pub extra_methods: Vec<String>,
    pub get_exemplar_fn: Arc<dyn Fn(&Request<hyper::Body>) -> Labels + Send + Sync>,
    pub extra_labels_from_ctx: HashMap<String, LabelValueFromCtx>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            extra_methods: Vec::new(),
            get_exemplar_fn: Arc::new(|_| Labels::new()),
            extra_labels_from_ctx: HashMap::new(),
        }
    }
}

impl Options {
    pub fn empty_dynamic_labels(&self) -> Labels {
        let mut labels = Labels::new();
        for label in self.extra_labels_from_ctx.keys() {
            labels.insert(label.clone(), "".to_string());
        }
        labels
    }
}

pub struct OptionApplyFunc<F>(F);

impl<F> Option for OptionApplyFunc<F>
where
    F: Fn(&mut Options) + Send + Sync,
{
    fn apply(&self, opts: &mut Options) {
        (self.0)(opts)
    }
}

pub fn with_extra_methods(methods: Vec<String>) -> impl Option {
    OptionApplyFunc(move |opts: &mut Options| {
        opts.extra_methods = methods.clone();
    })
}

pub fn with_exemplar_from_context<F>(get_exemplar_fn: F) -> impl Option
where
    F: Fn(&Request<hyper::Body>) -> Labels + Send + Sync + 'static,
{
    OptionApplyFunc(move |opts: &mut Options| {
        opts.get_exemplar_fn = Arc::new(get_exemplar_fn);
    })
}

pub fn with_label_from_ctx<F>(name: String, value_fn: F) -> impl Option
where
    F: Fn(&Request<hyper::Body>) -> String + Send + Sync + 'static,
{
    OptionApplyFunc(move |opts: &mut Options| {
        opts.extra_labels_from_ctx.insert(name.clone(), Arc::new(value_fn));
    })
}