//! Zero-overhead TCP Health & Metrics Server (برگرفته از bot-health-metrics-server)

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{info, warn};

pub struct HealthServer {
    is_ready: Arc<AtomicBool>,
    started_at: Instant,
}

impl HealthServer {
    pub fn new() -> Self {
        Self {
            is_ready: Arc::new(AtomicBool::new(true)),
            started_at: Instant::now(),
        }
    }

    pub fn set_ready(&self, ready: bool) {
        self.is_ready.store(ready, Ordering::Relaxed);
    }

    /// اجرای سرور سلامت در پس‌زمینه با سوکت خام Tokio TCP بدون سربار فریم‌ورک‌های وب سنگین
    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;
        info!("🩺 سرور پایش سلامت و وضعیت سرویس گردآفرید روی پورت {} فعال شد.", port);

        let is_ready = self.is_ready.clone();
        let started_at = self.started_at;

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, _)) => {
                        let ready = is_ready.load(Ordering::Relaxed);
                        let uptime = started_at.elapsed().as_secs();

                        tokio::spawn(async move {
                            let mut buf = [0u8; 1024];
                            let n = match stream.read(&mut buf).await {
                                Ok(n) if n > 0 => n,
                                _ => return,
                            };
                            let req = String::from_utf8_lossy(&buf[..n]);

                            if req.contains("GET /metrics") {
                                let body = format!(
                                    "# HELP gordafarid_uptime_seconds Bot uptime in seconds\n\
                                     # TYPE gordafarid_uptime_seconds counter\n\
                                     gordafarid_uptime_seconds {}\n\
                                     # HELP gordafarid_healthy Health status (1=healthy, 0=unhealthy)\n\
                                     # TYPE gordafarid_healthy gauge\n\
                                     gordafarid_healthy 1\n",
                                    uptime
                                );
                                let resp = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    body.len(),
                                    body
                                );
                                let _ = stream.write_all(resp.as_bytes()).await;
                            } else {
                                // Default /health
                                let body = serde_json::json!({
                                    "status": if ready { "healthy" } else { "unhealthy" },
                                    "healthy": ready,
                                    "uptime_secs": uptime,
                                    "app": "Gordafarid",
                                    "version": env!("CARGO_PKG_VERSION"),
                                }).to_string();

                                let status_code = if ready { "200 OK" } else { "503 Service Unavailable" };
                                let resp = format!(
                                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    status_code,
                                    body.len(),
                                    body
                                );
                                let _ = stream.write_all(resp.as_bytes()).await;
                            }
                        });
                    }
                    Err(e) => {
                        warn!("خطا در پذیرش اتصال سرور سلامت: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
