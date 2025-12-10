use crate::config::CallbackConfig;
use crate::prelude::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Callback 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallbackEvent {
    ExperimentStarted,
    ExperimentCompleted,
    ExperimentFailed,
    ExperimentAborted,
}

/// Callback 请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackPayload {
    pub experiment: String,
    pub event: CallbackEvent,
    pub status: String,
    pub report_url: Option<String>,
    pub error: Option<String>,
    pub timestamp: String,
}

impl CallbackPayload {
    pub fn new(
        experiment: String,
        event: CallbackEvent,
        status: String,
        report_url: Option<String>,
        error: Option<String>,
    ) -> Self {
        CallbackPayload {
            experiment,
            event,
            status,
            report_url,
            error,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// Callback 服务
pub struct CallbackService {
    config: CallbackConfig,
    client: reqwest::blocking::Client,
}

impl CallbackService {
    pub fn new(config: CallbackConfig) -> Self {
        let timeout = std::time::Duration::from_secs(config.timeout_secs());
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to create HTTP client");

        CallbackService { config, client }
    }

    /// 发送 callback 通知（带重试）
    pub fn notify(&self, url: &str, payload: CallbackPayload) -> Fallible<()> {
        let max_retries = self.config.retry_count();
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                info!(
                    "retrying callback to {} (attempt {}/{})",
                    url,
                    attempt + 1,
                    max_retries + 1
                );
                // Simple backoff: wait 1 second * attempt number
                std::thread::sleep(std::time::Duration::from_secs(attempt as u64));
            }

            match self.send_once(url, &payload) {
                Ok(()) => {
                    info!("callback sent successfully to {}", url);
                    return Ok(());
                }
                Err(e) => {
                    warn!("callback to {} failed: {}", url, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("callback failed without error details")))
    }

    fn send_once(&self, url: &str, payload: &CallbackPayload) -> Fallible<()> {
        let response = self
            .client
            .post(url)
            .json(payload)
            .send()
            .context("failed to send callback request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "callback returned non-success status: {}",
                response.status()
            );
        }

        Ok(())
    }

    /// 批量发送通知
    pub fn notify_all(&self, urls: &[String], payload: CallbackPayload) -> Fallible<()> {
        let mut errors = Vec::new();

        for url in urls {
            if let Err(e) = self.notify(url, payload.clone()) {
                errors.push((url.clone(), e));
            }
        }

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|(url, e)| format!("{}: {}", url, e))
                .collect();
            anyhow::bail!("some callbacks failed:\n{}", error_messages.join("\n"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiments::Status;

    #[test]
    fn test_callback_payload_serialization() {
        let payload = CallbackPayload::new(
            "test-exp".to_string(),
            CallbackEvent::ExperimentStarted,
            Status::Running.to_string(),
            None,
            None,
        );

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"experiment\":\"test-exp\""));
        assert!(json.contains("\"event\":\"experiment_started\""));
        assert!(json.contains("\"status\":\"running\""));

        // Verify roundtrip
        let parsed: CallbackPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.experiment, "test-exp");
        assert_eq!(parsed.status, Status::Running.to_string());
    }

    #[test]
    fn test_callback_event_serialization() {
        let event = CallbackEvent::ExperimentCompleted;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"experiment_completed\"");

        let parsed: CallbackEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            CallbackEvent::ExperimentCompleted => {}
            _ => panic!("unexpected event type"),
        }
    }

    #[test]
    fn test_callback_service_creation() {
        let config = CallbackConfig {
            timeout_secs: Some(10),
            retry_count: Some(3),
        };
        let service = CallbackService::new(config);
        assert_eq!(service.config.timeout_secs(), 10);
        assert_eq!(service.config.retry_count(), 3);
    }
}
