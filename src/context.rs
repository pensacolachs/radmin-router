use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// Context for an incoming request.
pub struct Context<Extra> {
    /// Parameters corresponding to dynamic route segments.
    pub params: Vec<String>,
    /// Shared pointer to router-level extra data (shared state).
    pub ex: Arc<Extra>,
}

impl<Extra> Clone for Context<Extra> {
    fn clone(&self) -> Self {
        Self {
            params: Clone::clone(&self.params),
            ex: Arc::clone(&self.ex),
        }
    }
}

impl<Extra: Debug> Debug for Context<Extra> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("params", &self.params)
            .field("ex", &self.ex)
            .finish()
    }
}
