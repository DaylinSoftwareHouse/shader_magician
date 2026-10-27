use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct BindlessArray<T>(pub Box<[T]>);

/// A runtime-sized array, `array<T>` in WGSL.  Only valid as the last (or only) member
/// of a storage buffer, so use it as a `#[read]`/`#[write]` group field.
#[derive(Deref, DerefMut)]
pub struct RuntimeArray<T>(pub Box<[T]>);
