use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use derive_more::{Deref, DerefMut};

#[repr(u32)]
pub enum BuiltInTy {
    VertexIndex = 0,
    InstanceIndex = 1,
    Position = 2,
    FrontFacing = 3,
    FragDepth = 4,
    LocalInvocationId = 5,
    LocalInvocationIndex = 6,
    GlobalInvocationId = 7,
    WorkGroupId = 8,
    NumWorkGroups = 9,
    SampleIndex = 10,
    SampleMask = 11
}

#[derive(Default, Deref, DerefMut)]
pub struct BuiltIn<const TY: u32, T> {
    data: T
}

impl <const TY: u32, T> BuiltIn<TY, T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Deref, DerefMut)]
pub struct Location<const POSITION: u32, T> {
    data: T
}

impl <const POSITION: u32, T> Location<POSITION, T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Deref, DerefMut)]
pub struct Group<T> {
    data: T
}

impl <T> Group<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Deref, DerefMut)]
pub struct Binding<T> {
    data: T
}

impl <T> Binding<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Default)]
pub struct Uniform<T> {
    _phantom: PhantomData<T>
}

impl <T> Deref for Uniform<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { todo!() }
}

impl <T> DerefMut for Uniform<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { todo!() }
}

#[derive(Default)]
pub struct Storage<T> {
    _phantom: PhantomData<T>
}
