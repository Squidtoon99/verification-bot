use deadpool_redis::{Config, Pool, Runtime};

#[derive(Clone)]
pub struct Context {
    pub redis: Pool,
}

impl Context {
    pub fn new<S: ToString>(redis_url: S) -> Self {
        let cfg = Config::from_url(redis_url.to_string());
        let redis = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

        Self { redis }
    }
}
