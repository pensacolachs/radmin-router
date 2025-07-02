use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Clone)]
pub struct Context<Extra: Clone> {
    pub params: Vec<String>,
    pub ex: Arc<Extra>,
}

impl<Extra: Clone> Debug for Context<Extra>
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
