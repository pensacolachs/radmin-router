use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct Context<Extra> {
    pub params: Vec<String>,
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

impl<Extra> Debug for Context<Extra>
where
    Extra: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("params", &self.params)
            .field("ex", &self.ex)
            .finish()
    }
}
