use crate::error::OdeResult;


pub struct TimeoutWrapper {
    pub timeout_ms: u64,
}

impl TimeoutWrapper {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    #[cfg(feature = "tokio")]
    pub async fn run<F, Fut, R>(&self, f: F) -> OdeResult<R>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = OdeResult<R>>,
    {
        let timeout_duration = Duration::from_millis(self.timeout_ms);

        tokio::time::timeout(timeout_duration, f())
            .await
            .map_err(|_| {
                OdeError::Timeout(format!(
                    "Conversion timed out after {}ms",
                    self.timeout_ms
                ))
            })?
    }

    pub fn run_sync<F, R>(&self, f: F) -> OdeResult<R>
    where
        F: FnOnce() -> OdeResult<R>,
    {
        #[cfg(feature = "tokio")]
        {
            let timeout_duration = Duration::from_millis(self.timeout_ms);

            let runtime = tokio::runtime::Handle::try_current()
                .ok()
                .or_else(|| tokio::runtime::Runtime::new().ok().map(|r| r.handle().clone()));

            if let Some(handle) = runtime {
                return handle.block_on(async move {
                    tokio::time::timeout(timeout_duration, f())
                        .await
                        .map_err(|_| {
                            OdeError::Timeout(format!(
                                "Conversion timed out after {}ms",
                                self.timeout_ms
                            ))
                        })?
                });
            }
        }

        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OdeError;

    #[test]
    fn test_timeout_wrapper_creation() {
        let wrapper = TimeoutWrapper::new(1000);
        assert_eq!(wrapper.timeout_ms, 1000);
    }

    #[test]
    fn test_timeout_sync_success() {
        let wrapper = TimeoutWrapper::new(1000);
        let result = wrapper.run_sync(|| Ok::<_, OdeError>(42));
        assert_eq!(result.unwrap(), 42);
    }
}