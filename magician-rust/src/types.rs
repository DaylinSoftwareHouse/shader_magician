use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct BindlessArray<T>(pub Box<[T]>);