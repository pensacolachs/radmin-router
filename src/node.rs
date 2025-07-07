use crate::route::Route;
use crate::segment::Segment;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Node<Extra: Send + Sync> {
    pub route: Option<Route<Extra>>,
    pub children: HashMap<Segment, Node<Extra>>,
}

impl<Extra: Send + Sync> Clone for Node<Extra> {
    fn clone(&self) -> Self {
        Self {
            route: Clone::clone(&self.route),
            children: Clone::clone(&self.children),
        }
    }
}

impl<Extra: Send + Sync> Default for Node<Extra> {
    fn default() -> Self {
        Self {
            route: None,
            children: HashMap::new(),
        }
    }
}

impl<Extra: Send + Sync> Node<Extra> {
    pub fn append(&mut self, route: Route<Extra>) {
        let mut current = self;

        for segment in route.path.0.clone() {
            current = current
                .children
                .entry(segment)
                .or_insert(Node::<Extra>::default());
        }

        current.route = Some(route);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::*;

    #[test]
    fn default() {
        let node = Node::<()>::default();
        assert!(node.children.is_empty());
        assert!(node.route.is_none());
    }

    #[test]
    fn insert_root() {
        let mut root = Node::<()>::default();

        root.append(Route::new(Path::from(vec![])));
        assert!(root.route.is_some());
    }

    #[test]
    fn insert_create_nested() {
        let mut root = Node::<()>::default();
        let path = vec![
            Segment::Literal("test".into()),
            Segment::Literal("path".into()),
        ];

        root.append(Route::new(path.clone()));

        assert!(root.route.is_none());
        assert_eq!(root.children.len(), 1);

        let child = root.children.get(&path[0]);
        assert!(child.is_some());
        let child = child.unwrap();
        assert!(child.route.is_none());
        assert_eq!(child.children.len(), 1);

        let grandchild = child.children.get(&path[1]);
        assert!(grandchild.is_some());
        let grandchild = grandchild.unwrap();
        assert!(grandchild.route.is_some());
        assert!(grandchild.children.is_empty());
    }
}
