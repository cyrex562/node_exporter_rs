use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use prometheus::{proto::MetricFamily, Encoder, Gatherer, TextEncoder};
use tokio::sync::Mutex;
use tokio::time::interval;

const DEFAULT_INTERVAL: Duration = Duration::from_secs(15);
const MILLISECONDS_PER_SECOND: i64 = 1000;

#[derive(Clone, Copy)]
enum HandlerErrorHandling {
    ContinueOnError,
    AbortOnError,
}

struct Config {
    use_tags: bool,
    url: String,
    prefix: String,
    interval: Duration,
    timeout: Duration,
    gatherer: Arc<dyn Gatherer>,
    logger: Option<Arc<dyn Logger>>,
    error_handling: HandlerErrorHandling,
}

struct Bridge {
    use_tags: bool,
    url: String,
    prefix: String,
    interval: Duration,
    timeout: Duration,
    error_handling: HandlerErrorHandling,
    logger: Option<Arc<dyn Logger>>,
    gatherer: Arc<dyn Gatherer>,
}

impl Bridge {
    fn new(config: Config) -> Result<Self, &'static str> {
        if config.url.is_empty() {
            return Err("missing URL");
        }

        Ok(Self {
            use_tags: config.use_tags,
            url: config.url,
            prefix: config.prefix,
            interval: config.interval,
            timeout: config.timeout,
            error_handling: config.error_handling,
            logger: config.logger,
            gatherer: config.gatherer,
        })
    }

    async fn run(&self, ctx: tokio::sync::CancellationToken) {
        let mut ticker = interval(self.interval);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(err) = self.push().await {
                        if let Some(logger) = &self.logger {
                            logger.println(&format!("error pushing to Graphite: {:?}", err));
                        }
                    }
                }
                _ = ctx.cancelled() => {
                    return;
                }
            }
        }
    }

    async fn push(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mfs = self.gatherer.gather()?;
        if mfs.is_empty() {
            match self.error_handling {
                HandlerErrorHandling::AbortOnError => return Err("no metrics gathered".into()),
                HandlerErrorHandling::ContinueOnError => {
                    if let Some(logger) = &self.logger {
                        logger.println("continue on error: no metrics gathered");
                    }
                }
            }
        }

        let mut stream = TcpStream::connect_timeout(&self.url.parse()?, self.timeout)?;
        stream.set_write_timeout(Some(self.timeout))?;
        let mut writer = BufWriter::new(stream);

        write_metrics(&mut writer, &mfs, self.use_tags, &self.prefix)?;
        writer.flush()?;

        Ok(())
    }
}

fn write_metrics<W: Write>(
    writer: &mut W,
    mfs: &[MetricFamily],
    use_tags: bool,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(mfs, &mut buffer)?;

    let samples = String::from_utf8(buffer)?;
    let mut buf_writer = BufWriter::new(writer);

    for line in samples.lines() {
        if !prefix.is_empty() {
            buf_writer.write_all(prefix.as_bytes())?;
            buf_writer.write_all(b".")?;
        }
        buf_writer.write_all(line.as_bytes())?;
        buf_writer.write_all(b"\n")?;
    }

    Ok(())
}

trait Logger: Send + Sync {
    fn println(&self, v: &str);
}

// #[tokio::main]
// async fn main() {
//     let gatherer = Arc::new(prometheus::default_registry().gatherer());
//     let config = Config {
//         use_tags: false,
//         url: "127.0.0.1:2003".to_string(),
//         prefix: "".to_string(),
//         interval: DEFAULT_INTERVAL,
//         timeout: DEFAULT_INTERVAL,
//         gatherer,
//         logger: None,
//         error_handling: HandlerErrorHandling::ContinueOnError,
//     };

//     let bridge = Bridge::new(config).unwrap();
//     let ctx = tokio::sync::CancellationToken::new();
//     bridge.run(ctx).await;
// }
