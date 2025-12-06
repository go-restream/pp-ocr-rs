#[cfg(feature = "server")]
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
#[cfg(feature = "server")]
use std::time::Instant;
#[cfg(feature = "server")]
use crate::server::server::AppState;

/// 访问日志中间件
#[cfg(feature = "server")]
pub async fn access_log_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();

    // 获取请求信息 - 使用克隆避免借用问题
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // 获取 User-Agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    // 获取客户端 IP
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // 执行请求
    let response = next.run(request).await;

    // 计算处理时间
    let duration = start_time.elapsed();
    let status = response.status();

    // 记录访问日志
    log::info!(
        "\"{} {} {}\" - {} - {}ms - \"{}\"",
        method,
        uri.path(),
        uri.query().unwrap_or(""),
        status.as_str(),
        duration.as_millis(),
        user_agent
    );

    log::debug!(
        "Client IP: {}, Method: {}, Path: {}, Status: {}, Duration: {:?}",
        client_ip,
        method,
        uri.path(),
        status.as_str(),
        duration
    );

    Ok(response)
}

/// 请求响应结构体，用于记录详细的请求信息
#[cfg(feature = "server")]
#[derive(Debug, Clone)]
pub struct RequestLog {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub status: u16,
    pub duration_ms: u128,
    pub user_agent: String,
    pub client_ip: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "server")]
impl RequestLog {
    pub fn new(
        method: String,
        path: String,
        query: Option<String>,
        status: u16,
        duration_ms: u128,
        user_agent: String,
        client_ip: String,
    ) -> Self {
        Self {
            method,
            path,
            query,
            status,
            duration_ms,
            user_agent,
            client_ip,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn to_log_string(&self) -> String {
        format!(
            "\"{} {} {}\" - {} - {}ms - \"{}\" - [{}]",
            self.method,
            self.path,
            self.query.as_deref().unwrap_or(""),
            self.status,
            self.duration_ms,
            self.user_agent,
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(not(feature = "server"))]
pub fn access_log_middleware() {
    // 空实现，避免编译错误
}