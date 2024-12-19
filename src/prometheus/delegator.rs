use std::io::{self, Read};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use hyper::body::HttpBody;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::spawn;
use tokio::time::Duration;

const CLOSE_NOTIFIER: usize = 1 << 0;
const FLUSHER: usize = 1 << 1;
const HIJACKER: usize = 1 << 2;
const READER_FROM: usize = 1 << 3;
const PUSHER: usize = 1 << 4;

#[derive(Clone)]
struct ResponseWriterDelegator {
    inner: hyper::Response<Body>,
    status: Arc<AtomicI64>,
    written: Arc<AtomicI64>,
    wrote_header: Arc<Mutex<bool>>,
    observe_write_header: Option<Arc<dyn Fn(i64) + Send + Sync>>,
}

impl ResponseWriterDelegator {
    fn new(inner: hyper::Response<Body>, observe_write_header: Option<Arc<dyn Fn(i64) + Send + Sync>>) -> Self {
        Self {
            inner,
            status: Arc::new(AtomicI64::new(0)),
            written: Arc::new(AtomicI64::new(0)),
            wrote_header: Arc::new(Mutex::new(false)),
            observe_write_header,
        }
    }

    async fn write_header(&self, code: i64) {
        let mut wrote_header = self.wrote_header.lock().await;
        if let Some(ref observe) = self.observe_write_header {
            if !*wrote_header {
                observe(code);
            }
        }
        self.status.store(code, Ordering::SeqCst);
        *wrote_header = true;
    }

    async fn write(&self, bytes: &[u8]) -> io::Result<usize> {
        if !*self.wrote_header.lock().await {
            self.write_header(200).await;
        }
        let n = self.inner.body_mut().write(bytes).await?;
        self.written.fetch_add(n as i64, Ordering::SeqCst);
        Ok(n)
    }
}

impl hyper::body::HttpBody for ResponseWriterDelegator {
    type Data = hyper::body::Bytes;
    type Error = hyper::Error;

    fn poll_data(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Data, Self::Error>>> {
        self.inner.body_mut().poll_data(cx)
    }

    fn poll_trailers(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
        self.inner.body_mut().poll_trailers(cx)
    }
}

struct CloseNotifierDelegator(ResponseWriterDelegator);
struct FlusherDelegator(ResponseWriterDelegator);
struct HijackerDelegator(ResponseWriterDelegator);
struct ReaderFromDelegator(ResponseWriterDelegator);
struct PusherDelegator(ResponseWriterDelegator);

impl CloseNotifierDelegator {
    fn close_notify(&self) -> tokio::sync::oneshot::Receiver<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        tx.send(()).unwrap();
        rx
    }
}

impl FlusherDelegator {
    async fn flush(&self) {
        if !*self.0.wrote_header.lock().await {
            self.0.write_header(200).await;
        }
        self.0.inner.body_mut().flush().await.unwrap();
    }
}

impl HijackerDelegator {
    fn hijack(&self) -> io::Result<(TcpStream, io::BufReader<TcpStream>)> {
        unimplemented!()
    }
}

impl ReaderFromDelegator {
    async fn read_from<R: Read>(&self, reader: &mut R) -> io::Result<u64> {
        if !*self.0.wrote_header.lock().await {
            self.0.write_header(200).await;
        }
        let mut buf = vec![0; 8192];
        let mut total = 0;
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            self.0.write(&buf[..n]).await?;
            total += n as u64;
        }
        Ok(total)
    }
}

impl PusherDelegator {
    async fn push(&self, target: &str, opts: &hyper::header::HeaderMap) -> io::Result<()> {
        unimplemented!()
    }
}

type Delegator = Arc<dyn hyper::body::HttpBody<Data = hyper::body::Bytes, Error = hyper::Error> + Send + Sync>;

fn pick_delegator(flags: usize, d: ResponseWriterDelegator) -> Delegator {
    match flags {
        0 => Arc::new(d),
        CLOSE_NOTIFIER => Arc::new(CloseNotifierDelegator(d)),
        FLUSHER => Arc::new(FlusherDelegator(d)),
        HIJACKER => Arc::new(HijackerDelegator(d)),
        READER_FROM => Arc::new(ReaderFromDelegator(d)),
        PUSHER => Arc::new(PusherDelegator(d)),
        _ => unimplemented!(),
    }
}

async fn new_delegator(w: hyper::Response<Body>, observe_write_header: Option<Arc<dyn Fn(i64) + Send + Sync>>) -> Delegator {
    let d = ResponseWriterDelegator::new(w, observe_write_header);

    let mut flags = 0;
    if w.extensions().get::<hyper::body::HttpBody>().is_some() {
        flags |= CLOSE_NOTIFIER;
    }
    if w.extensions().get::<hyper::body::HttpBody>().is_some() {
        flags |= FLUSHER;
    }
    if w.extensions().get::<hyper::body::HttpBody>().is_some() {
        flags |= HIJACKER;
    }
    if w.extensions().get::<hyper::body::HttpBody>().is_some() {
        flags |= READER_FROM;
    }
    if w.extensions().get::<hyper::body::HttpBody>().is_some() {
        flags |= PUSHER;
    }

    pick_delegator(flags, d)
}