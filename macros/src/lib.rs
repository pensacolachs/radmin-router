pub use macro_impl::{box_future, CaseIterable};

pub trait CaseIterable: 'static + Sized {
    const ALL_CASES: &'static [Self];
}

#[derive(CaseIterable)]
enum A {
    B
}