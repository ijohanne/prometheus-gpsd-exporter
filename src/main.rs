mod gpsd;
mod metrics;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use prometheus::Encoder;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use warp::Filter;

use crate::metrics::Metrics;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about = "Prometheus exporter for gpsd")]
struct Args {
    #[clap(short = 'H', long, default_value = "localhost")]
    hostname: String,

    #[clap(short = 'p', long, default_value_t = 2947)]
    port: u16,

    #[clap(short = 'E', long = "exporter-port", default_value_t = 9015)]
    exporter_port: u16,

    #[clap(short = 'L', long = "listen-address", default_value = "::")]
    listen_address: String,

    #[clap(short = 't', long, default_value_t = 10)]
    timeout: u64,

    #[clap(long = "retry-delay", default_value_t = 10)]
    retry_delay: u64,

    #[clap(long = "max-retry-delay", default_value_t = 300)]
    max_retry_delay: u64,
}

fn with_metrics(
    metrics: Arc<Mutex<Metrics>>,
) -> impl Filter<Extract = (Arc<Mutex<Metrics>>,), Error = Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}

async fn metrics_handler(metrics: Arc<Mutex<Metrics>>) -> Result<impl warp::Reply, Infallible> {
    let m = metrics.lock().await;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = m.registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(warp::reply::with_header(
        buffer,
        "content-type",
        encoder.format_type(),
    ))
}

async fn gpsd_loop(args: Args, metrics: Arc<Mutex<Metrics>>) {
    let mut retry_delay = args.retry_delay;

    loop {
        let addr = format!("{}:{}", args.hostname, args.port);
        tracing::info!("connecting to gpsd at {addr}");

        match tokio::time::timeout(
            Duration::from_secs(args.timeout),
            TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(stream)) => {
                tracing::info!("connected to gpsd at {addr}");
                retry_delay = args.retry_delay;

                if let Err(e) = handle_gpsd_stream(stream, &metrics).await {
                    tracing::warn!("gpsd connection lost: {e}");
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("failed to connect to gpsd at {addr}: {e}");
            }
            Err(_) => {
                tracing::warn!("connection to gpsd at {addr} timed out");
            }
        }

        tracing::info!("reconnecting in {retry_delay}s");
        tokio::time::sleep(Duration::from_secs(retry_delay)).await;
        retry_delay = (retry_delay * 2).min(args.max_retry_delay);
    }
}

async fn handle_gpsd_stream(
    mut stream: TcpStream,
    metrics: &Arc<Mutex<Metrics>>,
) -> Result<()> {
    let watch_cmd = b"?WATCH={\"enable\":true,\"json\":true}\n";
    stream.write_all(watch_cmd).await?;

    let (reader, _writer) = stream.split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if let Some(msg) = gpsd::parse_message(&line) {
            let m = metrics.lock().await;
            m.process(&msg);
        }
    }

    anyhow::bail!("gpsd closed connection")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let metrics = Arc::new(Mutex::new(Metrics::new()));

    let gpsd_metrics = metrics.clone();
    let gpsd_args = args.clone();
    tokio::spawn(async move {
        gpsd_loop(gpsd_args, gpsd_metrics).await;
    });

    let metrics_route = warp::path!("metrics")
        .and(warp::get())
        .and(with_metrics(metrics))
        .and_then(metrics_handler);

    let addr: std::net::SocketAddr = format!("{}:{}", args.listen_address, args.exporter_port)
        .parse()
        .expect("valid listen address");

    tracing::info!("starting metrics server on {addr}");
    warp::serve(metrics_route).run(addr).await;

    Ok(())
}
