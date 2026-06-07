//! A Rust implementation of the WGSL math library.
//!
//! This module provides 1-to-1 mappings of WGSL's built-in vector and matrix types,
//! constructors, operations, and mathematical functions. All free functions mirror
//! their WGSL counterparts exactly. Constructors are exposed as `new` methods inside
//! `impl` blocks rather than as bare constructor calls, which is the only intentional
//! deviation from the WGSL spec.
//!
//! # Type Mapping
//! | WGSL         | Rust (this module) |
//! |--------------|--------------------|
//! | vec2<f32>    | Vec2               |
//! | vec3<f32>    | Vec3               |
//! | vec4<f32>    | Vec4               |
//! | vec2<f64>    | DVec2              |
//! | vec3<f64>    | DVec3              |
//! | vec4<f64>    | DVec4              |
//! | vec2<i32>    | IVec2              |
//! | vec3<i32>    | IVec3              |
//! | vec4<i32>    | IVec4              |
//! | vec2<u32>    | UVec2              |
//! | vec3<u32>    | UVec3              |
//! | vec4<u32>    | UVec4              |
//! | vec2<bool>   | BVec2              |
//! | vec3<bool>   | BVec3              |
//! | vec4<bool>   | BVec4              |
//! | mat2x2<f32>  | Mat2               |
//! | mat3x3<f32>  | Mat3               |
//! | mat4x4<f32>  | Mat4               |
//! | mat2x2<f64>  | DMat2              |
//! | mat3x3<f64>  | DMat3              |
//! | mat4x4<f64>  | DMat4              |

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
// ─────────────────────────────────────────────────────────────────────────────
// Re-export glam for users who need lower-level access.
// ─────────────────────────────────────────────────────────────────────────────
pub use glam;

// ─────────────────────────────────────────────────────────────────────────────
// Vector types
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec2(pub glam::Vec2);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec3(pub glam::Vec3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vec4(pub glam::Vec4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DVec2(pub glam::DVec2);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DVec3(pub glam::DVec3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DVec4(pub glam::DVec4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Eq, Hash)]
pub struct IVec2(pub glam::IVec2);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Eq, Hash)]
pub struct IVec3(pub glam::IVec3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Eq, Hash)]
pub struct IVec4(pub glam::IVec4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Eq, Hash)]
pub struct UVec2(pub glam::UVec2);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Eq, Hash)]
pub struct UVec3(pub glam::UVec3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Eq, Hash)]
pub struct UVec4(pub glam::UVec4);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BVec2(pub glam::BVec2);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BVec3(pub glam::BVec3);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BVec4(pub glam::BVec4);

// ─────────────────────────────────────────────────────────────────────────────
// Matrix types
// ─────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Mat2(pub glam::Mat2);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Mat3(pub glam::Mat3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Mat4(pub glam::Mat4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DMat2(pub glam::DMat2);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DMat3(pub glam::DMat3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct DMat4(pub glam::DMat4);

// ─────────────────────────────────────────────────────────────────────────────
// Vec2 – constructors & accessors
// ─────────────────────────────────────────────────────────────────────────────
impl Vec2 {
    /// `vec2<f32>(x, y)`
    #[inline] pub fn new(x: f32, y: f32) -> Self { Self(glam::Vec2::new(x, y)) }
    /// `vec2<f32>(scalar)` – splat
    #[inline] pub fn splat(v: f32) -> Self { Self(glam::Vec2::splat(v)) }

    #[inline] pub fn x(self) -> f32 { self.0.x }
    #[inline] pub fn y(self) -> f32 { self.0.y }
    #[inline] pub fn xx(self) -> Self { Self::new(self.0.x, self.0.x) }
    #[inline] pub fn xy(self) -> Self { Self::new(self.0.x, self.0.y) }
    #[inline] pub fn yx(self) -> Self { Self::new(self.0.y, self.0.x) }
    #[inline] pub fn yy(self) -> Self { Self::new(self.0.y, self.0.y) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Vec3 – constructors & accessors
// ─────────────────────────────────────────────────────────────────────────────
impl Vec3 {
    #[inline] pub fn new(x: f32, y: f32, z: f32) -> Self { Self(glam::Vec3::new(x, y, z)) }
    #[inline] pub fn splat(v: f32) -> Self { Self(glam::Vec3::splat(v)) }
    /// Construct from vec2 + z
    #[inline] pub fn from_vec2_z(xy: Vec2, z: f32) -> Self { Self(glam::Vec3::new(xy.0.x, xy.0.y, z)) }

    #[inline] pub fn x(self) -> f32 { self.0.x }
    #[inline] pub fn y(self) -> f32 { self.0.y }
    #[inline] pub fn z(self) -> f32 { self.0.z }

    // Common swizzles
    #[inline] pub fn xy(self) -> Vec2 { Vec2::new(self.0.x, self.0.y) }
    #[inline] pub fn xz(self) -> Vec2 { Vec2::new(self.0.x, self.0.z) }
    #[inline] pub fn yz(self) -> Vec2 { Vec2::new(self.0.y, self.0.z) }
    #[inline] pub fn xyz(self) -> Self { self }
    #[inline] pub fn zyx(self) -> Self { Self::new(self.0.z, self.0.y, self.0.x) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Vec4 – constructors & accessors
// ─────────────────────────────────────────────────────────────────────────────
impl Vec4 {
    #[inline] pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self(glam::Vec4::new(x, y, z, w)) }
    #[inline] pub fn splat(v: f32) -> Self { Self(glam::Vec4::splat(v)) }
    #[inline] pub fn from_vec3_w(xyz: Vec3, w: f32) -> Self { Self(glam::Vec4::new(xyz.0.x, xyz.0.y, xyz.0.z, w)) }
    #[inline] pub fn from_vec2_zw(xy: Vec2, z: f32, w: f32) -> Self { Self(glam::Vec4::new(xy.0.x, xy.0.y, z, w)) }

    #[inline] pub fn x(self) -> f32 { self.0.x }
    #[inline] pub fn y(self) -> f32 { self.0.y }
    #[inline] pub fn z(self) -> f32 { self.0.z }
    #[inline] pub fn w(self) -> f32 { self.0.w }

    #[inline] pub fn xy(self) -> Vec2 { Vec2::new(self.0.x, self.0.y) }
    #[inline] pub fn xyz(self) -> Vec3 { Vec3::new(self.0.x, self.0.y, self.0.z) }
    #[inline] pub fn xyzw(self) -> Self { self }
    #[inline] pub fn wzyx(self) -> Self { Self::new(self.0.w, self.0.z, self.0.y, self.0.x) }
}

// ─────────────────────────────────────────────────────────────────────────────
// DVec2 / DVec3 / DVec4
// ─────────────────────────────────────────────────────────────────────────────
impl DVec2 {
    #[inline] pub fn new(x: f64, y: f64) -> Self { Self(glam::DVec2::new(x, y)) }
    #[inline] pub fn splat(v: f64) -> Self { Self(glam::DVec2::splat(v)) }
    #[inline] pub fn x(self) -> f64 { self.0.x }
    #[inline] pub fn y(self) -> f64 { self.0.y }
}

impl DVec3 {
    #[inline] pub fn new(x: f64, y: f64, z: f64) -> Self { Self(glam::DVec3::new(x, y, z)) }
    #[inline] pub fn splat(v: f64) -> Self { Self(glam::DVec3::splat(v)) }
    #[inline] pub fn x(self) -> f64 { self.0.x }
    #[inline] pub fn y(self) -> f64 { self.0.y }
    #[inline] pub fn z(self) -> f64 { self.0.z }
    #[inline] pub fn xy(self) -> DVec2 { DVec2::new(self.0.x, self.0.y) }
}

impl DVec4 {
    #[inline] pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self { Self(glam::DVec4::new(x, y, z, w)) }
    #[inline] pub fn splat(v: f64) -> Self { Self(glam::DVec4::splat(v)) }
    #[inline] pub fn x(self) -> f64 { self.0.x }
    #[inline] pub fn y(self) -> f64 { self.0.y }
    #[inline] pub fn z(self) -> f64 { self.0.z }
    #[inline] pub fn w(self) -> f64 { self.0.w }
    #[inline] pub fn xy(self) -> DVec2 { DVec2::new(self.0.x, self.0.y) }
    #[inline] pub fn xyz(self) -> DVec3 { DVec3::new(self.0.x, self.0.y, self.0.z) }
}

// ─────────────────────────────────────────────────────────────────────────────
// IVec2 / IVec3 / IVec4
// ─────────────────────────────────────────────────────────────────────────────
impl IVec2 {
    #[inline] pub fn new(x: i32, y: i32) -> Self { Self(glam::IVec2::new(x, y)) }
    #[inline] pub fn splat(v: i32) -> Self { Self(glam::IVec2::splat(v)) }
    #[inline] pub fn x(self) -> i32 { self.0.x }
    #[inline] pub fn y(self) -> i32 { self.0.y }
}

impl IVec3 {
    #[inline] pub fn new(x: i32, y: i32, z: i32) -> Self { Self(glam::IVec3::new(x, y, z)) }
    #[inline] pub fn splat(v: i32) -> Self { Self(glam::IVec3::splat(v)) }
    #[inline] pub fn x(self) -> i32 { self.0.x }
    #[inline] pub fn y(self) -> i32 { self.0.y }
    #[inline] pub fn z(self) -> i32 { self.0.z }
    #[inline] pub fn xy(self) -> IVec2 { IVec2::new(self.0.x, self.0.y) }
}

impl IVec4 {
    #[inline] pub fn new(x: i32, y: i32, z: i32, w: i32) -> Self { Self(glam::IVec4::new(x, y, z, w)) }
    #[inline] pub fn splat(v: i32) -> Self { Self(glam::IVec4::splat(v)) }
    #[inline] pub fn x(self) -> i32 { self.0.x }
    #[inline] pub fn y(self) -> i32 { self.0.y }
    #[inline] pub fn z(self) -> i32 { self.0.z }
    #[inline] pub fn w(self) -> i32 { self.0.w }
    #[inline] pub fn xy(self) -> IVec2 { IVec2::new(self.0.x, self.0.y) }
    #[inline] pub fn xyz(self) -> IVec3 { IVec3::new(self.0.x, self.0.y, self.0.z) }
}

// ─────────────────────────────────────────────────────────────────────────────
// UVec2 / UVec3 / UVec4
// ─────────────────────────────────────────────────────────────────────────────
impl UVec2 {
    #[inline] pub fn new(x: u32, y: u32) -> Self { Self(glam::UVec2::new(x, y)) }
    #[inline] pub fn splat(v: u32) -> Self { Self(glam::UVec2::splat(v)) }
    #[inline] pub fn x(self) -> u32 { self.0.x }
    #[inline] pub fn y(self) -> u32 { self.0.y }
}

impl UVec3 {
    #[inline] pub fn new(x: u32, y: u32, z: u32) -> Self { Self(glam::UVec3::new(x, y, z)) }
    #[inline] pub fn splat(v: u32) -> Self { Self(glam::UVec3::splat(v)) }
    #[inline] pub fn x(self) -> u32 { self.0.x }
    #[inline] pub fn y(self) -> u32 { self.0.y }
    #[inline] pub fn z(self) -> u32 { self.0.z }
    #[inline] pub fn xy(self) -> UVec2 { UVec2::new(self.0.x, self.0.y) }
}

impl UVec4 {
    #[inline] pub fn new(x: u32, y: u32, z: u32, w: u32) -> Self { Self(glam::UVec4::new(x, y, z, w)) }
    #[inline] pub fn splat(v: u32) -> Self { Self(glam::UVec4::splat(v)) }
    #[inline] pub fn x(self) -> u32 { self.0.x }
    #[inline] pub fn y(self) -> u32 { self.0.y }
    #[inline] pub fn z(self) -> u32 { self.0.z }
    #[inline] pub fn w(self) -> u32 { self.0.w }
    #[inline] pub fn xy(self) -> UVec2 { UVec2::new(self.0.x, self.0.y) }
    #[inline] pub fn xyz(self) -> UVec3 { UVec3::new(self.0.x, self.0.y, self.0.z) }
}

// ─────────────────────────────────────────────────────────────────────────────
// BVec2 / BVec3 / BVec4
// ─────────────────────────────────────────────────────────────────────────────
impl BVec2 {
    #[inline] pub fn new(x: bool, y: bool) -> Self { Self(glam::BVec2::new(x, y)) }
    #[inline] pub fn splat(v: bool) -> Self { Self(glam::BVec2::new(v, v)) }
    #[inline] pub fn x(self) -> bool { self.0.x }
    #[inline] pub fn y(self) -> bool { self.0.y }
}

impl BVec3 {
    #[inline] pub fn new(x: bool, y: bool, z: bool) -> Self { Self(glam::BVec3::new(x, y, z)) }
    #[inline] pub fn splat(v: bool) -> Self { Self(glam::BVec3::new(v, v, v)) }
    #[inline] pub fn x(self) -> bool { self.0.x }
    #[inline] pub fn y(self) -> bool { self.0.y }
    #[inline] pub fn z(self) -> bool { self.0.z }
}

impl BVec4 {
    #[inline] pub fn new(x: bool, y: bool, z: bool, w: bool) -> Self { Self(glam::BVec4::new(x, y, z, w)) }
    #[inline] pub fn splat(v: bool) -> Self { Self(glam::BVec4::new(v, v, v, v)) }
    #[inline] pub fn x(self) -> bool { self.0.x }
    #[inline] pub fn y(self) -> bool { self.0.y }
    #[inline] pub fn z(self) -> bool { self.0.z }
    #[inline] pub fn w(self) -> bool { self.0.w }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mat2 / Mat3 / Mat4 – constructors
// ─────────────────────────────────────────────────────────────────────────────
impl Mat2 {
    /// Construct from two column vectors (WGSL is column-major).
    #[inline] pub fn new(col0: Vec2, col1: Vec2) -> Self {
        Self(glam::Mat2::from_cols(col0.0, col1.0))
    }
    #[inline] pub fn from_cols_array(a: &[f32; 4]) -> Self { Self(glam::Mat2::from_cols_array(a)) }
    #[inline] pub fn identity() -> Self { Self(glam::Mat2::IDENTITY) }
    #[inline] pub fn zero() -> Self { Self(glam::Mat2::ZERO) }
    #[inline] pub fn col(&self, i: usize) -> Vec2 { Vec2(self.0.col(i)) }
}

impl Mat3 {
    #[inline] pub fn new(col0: Vec3, col1: Vec3, col2: Vec3) -> Self {
        Self(glam::Mat3::from_cols(col0.0, col1.0, col2.0))
    }
    #[inline] pub fn from_cols_array(a: &[f32; 9]) -> Self { Self(glam::Mat3::from_cols_array(a)) }
    #[inline] pub fn identity() -> Self { Self(glam::Mat3::IDENTITY) }
    #[inline] pub fn zero() -> Self { Self(glam::Mat3::ZERO) }
    #[inline] pub fn col(&self, i: usize) -> Vec3 { Vec3(self.0.col(i)) }
}

impl Mat4 {
    #[inline] pub fn new(col0: Vec4, col1: Vec4, col2: Vec4, col3: Vec4) -> Self {
        Self(glam::Mat4::from_cols(col0.0, col1.0, col2.0, col3.0))
    }
    #[inline] pub fn from_cols_array(a: &[f32; 16]) -> Self { Self(glam::Mat4::from_cols_array(a)) }
    #[inline] pub fn identity() -> Self { Self(glam::Mat4::IDENTITY) }
    #[inline] pub fn zero() -> Self { Self(glam::Mat4::ZERO) }
    #[inline] pub fn col(&self, i: usize) -> Vec4 { Vec4(self.0.col(i)) }
}

impl DMat2 {
    #[inline] pub fn new(col0: DVec2, col1: DVec2) -> Self {
        Self(glam::DMat2::from_cols(col0.0, col1.0))
    }
    #[inline] pub fn from_cols_array(a: &[f64; 4]) -> Self { Self(glam::DMat2::from_cols_array(a)) }
    #[inline] pub fn identity() -> Self { Self(glam::DMat2::IDENTITY) }
    #[inline] pub fn zero() -> Self { Self(glam::DMat2::ZERO) }
    #[inline] pub fn col(&self, i: usize) -> DVec2 { DVec2(self.0.col(i)) }
}

impl DMat3 {
    #[inline] pub fn new(col0: DVec3, col1: DVec3, col2: DVec3) -> Self {
        Self(glam::DMat3::from_cols(col0.0, col1.0, col2.0))
    }
    #[inline] pub fn from_cols_array(a: &[f64; 9]) -> Self { Self(glam::DMat3::from_cols_array(a)) }
    #[inline] pub fn identity() -> Self { Self(glam::DMat3::IDENTITY) }
    #[inline] pub fn zero() -> Self { Self(glam::DMat3::ZERO) }
    #[inline] pub fn col(&self, i: usize) -> DVec3 { DVec3(self.0.col(i)) }
}

impl DMat4 {
    #[inline] pub fn new(col0: DVec4, col1: DVec4, col2: DVec4, col3: DVec4) -> Self {
        Self(glam::DMat4::from_cols(col0.0, col1.0, col2.0, col3.0))
    }
    #[inline] pub fn from_cols_array(a: &[f64; 16]) -> Self { Self(glam::DMat4::from_cols_array(a)) }
    #[inline] pub fn identity() -> Self { Self(glam::DMat4::IDENTITY) }
    #[inline] pub fn zero() -> Self { Self(glam::DMat4::ZERO) }
    #[inline] pub fn col(&self, i: usize) -> DVec4 { DVec4(self.0.col(i)) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arithmetic operator impls  (Vec2 shown; pattern repeats for all types)
// ─────────────────────────────────────────────────────────────────────────────

macro_rules! impl_vec_ops {
    ($T:ty, $inner:ty, $scalar:ty) => {
        impl Add for $T { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self(self.0 + rhs.0) } }
        impl Sub for $T { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self(self.0 - rhs.0) } }
        impl Mul for $T { type Output = Self; #[inline] fn mul(self, rhs: Self) -> Self { Self(self.0 * rhs.0) } }
        impl Div for $T { type Output = Self; #[inline] fn div(self, rhs: Self) -> Self { Self(self.0 / rhs.0) } }
        impl Mul<$scalar> for $T { type Output = Self; #[inline] fn mul(self, s: $scalar) -> Self { Self(self.0 * s) } }
        impl Mul<$T> for $scalar { type Output = $T; #[inline] fn mul(self, v: $T) -> $T { <$T>::from(self * v.0) } }
        impl Div<$scalar> for $T { type Output = Self; #[inline] fn div(self, s: $scalar) -> Self { Self(self.0 / s) } }
        impl AddAssign for $T { #[inline] fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; } }
        impl SubAssign for $T { #[inline] fn sub_assign(&mut self, rhs: Self) { self.0 -= rhs.0; } }
        impl MulAssign for $T { #[inline] fn mul_assign(&mut self, rhs: Self) { self.0 *= rhs.0; } }
        impl DivAssign for $T { #[inline] fn div_assign(&mut self, rhs: Self) { self.0 /= rhs.0; } }
    };
}

macro_rules! impl_vec_neg {
    ($T:ty) => {
        impl Neg for $T { type Output = Self; #[inline] fn neg(self) -> Self { Self(-self.0) } }
    };
}

impl_vec_ops!(Vec2, glam::Vec2, f32);
impl_vec_ops!(Vec3, glam::Vec3, f32);
impl_vec_ops!(Vec4, glam::Vec4, f32);
impl_vec_ops!(DVec2, glam::DVec2, f64);
impl_vec_ops!(DVec3, glam::DVec3, f64);
impl_vec_ops!(DVec4, glam::DVec4, f64);
impl_vec_ops!(IVec2, glam::IVec2, i32);
impl_vec_ops!(IVec3, glam::IVec3, i32);
impl_vec_ops!(IVec4, glam::IVec4, i32);
impl_vec_ops!(UVec2, glam::UVec2, u32);
impl_vec_ops!(UVec3, glam::UVec3, u32);
impl_vec_ops!(UVec4, glam::UVec4, u32);

impl_vec_neg!(Vec2);
impl_vec_neg!(Vec3);
impl_vec_neg!(Vec4);
impl_vec_neg!(DVec2);
impl_vec_neg!(DVec3);
impl_vec_neg!(DVec4);
impl_vec_neg!(IVec2);
impl_vec_neg!(IVec3);
impl_vec_neg!(IVec4);

// Matrix arithmetic
macro_rules! impl_mat_ops {
    ($Mat:ty, $Vec:ty, $scalar:ty) => {
        impl Add for $Mat { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self(self.0 + rhs.0) } }
        impl Sub for $Mat { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self(self.0 - rhs.0) } }
        impl Mul for $Mat { type Output = Self; #[inline] fn mul(self, rhs: Self) -> Self { Self(self.0 * rhs.0) } }
        impl Mul<$Vec> for $Mat { type Output = $Vec; #[inline] fn mul(self, v: $Vec) -> $Vec { <$Vec>::from(self.0 * v.0) } }
        impl Mul<$scalar> for $Mat { type Output = Self; #[inline] fn mul(self, s: $scalar) -> Self { Self(self.0 * s) } }
        impl Neg for $Mat { type Output = Self; #[inline] fn neg(self) -> Self { Self(-self.0) } }
    };
}

impl_mat_ops!(Mat2, Vec2, f32);
impl_mat_ops!(Mat3, Vec3, f32);
impl_mat_ops!(Mat4, Vec4, f32);
impl_mat_ops!(DMat2, DVec2, f64);
impl_mat_ops!(DMat3, DVec3, f64);
impl_mat_ops!(DMat4, DVec4, f64);

// ─────────────────────────────────────────────────────────────────────────────
// WGSL Logical built-ins
// ─────────────────────────────────────────────────────────────────────────────

/// `all(e)` – returns true if every component of a bool vector is true.
pub fn all_bvec2(e: BVec2) -> bool { e.0.x && e.0.y }
pub fn all_bvec3(e: BVec3) -> bool { e.0.x && e.0.y && e.0.z }
pub fn all_bvec4(e: BVec4) -> bool { e.0.x && e.0.y && e.0.z && e.0.w }

/// `any(e)` – returns true if any component of a bool vector is true.
pub fn any_bvec2(e: BVec2) -> bool { e.0.x || e.0.y }
pub fn any_bvec3(e: BVec3) -> bool { e.0.x || e.0.y || e.0.z }
pub fn any_bvec4(e: BVec4) -> bool { e.0.x || e.0.y || e.0.z || e.0.w }

/// `select(f, t, cond)` – scalar version.
#[inline] pub fn select<T>(f: T, t: T, cond: bool) -> T { if cond { t } else { f } }

/// `select(f, t, cond)` – component-wise Vec2.
#[inline] pub fn select_vec2(f: Vec2, t: Vec2, cond: BVec2) -> Vec2 {
    Vec2::new(select(f.0.x, t.0.x, cond.0.x), select(f.0.y, t.0.y, cond.0.y))
}
/// `select(f, t, cond)` – component-wise Vec3.
#[inline] pub fn select_vec3(f: Vec3, t: Vec3, cond: BVec3) -> Vec3 {
    Vec3::new(select(f.0.x, t.0.x, cond.0.x), select(f.0.y, t.0.y, cond.0.y), select(f.0.z, t.0.z, cond.0.z))
}
/// `select(f, t, cond)` – component-wise Vec4.
#[inline] pub fn select_vec4(f: Vec4, t: Vec4, cond: BVec4) -> Vec4 {
    Vec4::new(
        select(f.0.x, t.0.x, cond.0.x), select(f.0.y, t.0.y, cond.0.y),
        select(f.0.z, t.0.z, cond.0.z), select(f.0.w, t.0.w, cond.0.w),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// WGSL Numeric / Math built-ins
// All scalar variants accept f32; vector variants are component-wise wrappers.
// Names match WGSL exactly.
// ─────────────────────────────────────────────────────────────────────────────

// ── abs ──────────────────────────────────────────────────────────────────────
#[inline] pub fn abs_f32(e: f32) -> f32 { e.abs() }
#[inline] pub fn abs_i32(e: i32) -> i32 { e.abs() }
#[inline] pub fn abs_u32(e: u32) -> u32 { e }
#[inline] pub fn abs_vec2(e: Vec2) -> Vec2 { Vec2(e.0.abs()) }
#[inline] pub fn abs_vec3(e: Vec3) -> Vec3 { Vec3(e.0.abs()) }
#[inline] pub fn abs_vec4(e: Vec4) -> Vec4 { Vec4(e.0.abs()) }
#[inline] pub fn abs_ivec2(e: IVec2) -> IVec2 { IVec2(glam::IVec2::new(e.0.x.abs(), e.0.y.abs())) }
#[inline] pub fn abs_ivec3(e: IVec3) -> IVec3 { IVec3(glam::IVec3::new(e.0.x.abs(), e.0.y.abs(), e.0.z.abs())) }
#[inline] pub fn abs_ivec4(e: IVec4) -> IVec4 { IVec4(glam::IVec4::new(e.0.x.abs(), e.0.y.abs(), e.0.z.abs(), e.0.w.abs())) }

// ── acos ─────────────────────────────────────────────────────────────────────
#[inline] pub fn acos(e: f32) -> f32 { e.acos() }
#[inline] pub fn acos_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.acos(), e.0.y.acos()) }
#[inline] pub fn acos_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.acos(), e.0.y.acos(), e.0.z.acos()) }
#[inline] pub fn acos_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.acos(), e.0.y.acos(), e.0.z.acos(), e.0.w.acos()) }

// ── acosh ────────────────────────────────────────────────────────────────────
#[inline] pub fn acosh(e: f32) -> f32 { e.acosh() }
#[inline] pub fn acosh_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.acosh(), e.0.y.acosh()) }
#[inline] pub fn acosh_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.acosh(), e.0.y.acosh(), e.0.z.acosh()) }
#[inline] pub fn acosh_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.acosh(), e.0.y.acosh(), e.0.z.acosh(), e.0.w.acosh()) }

// ── asin ─────────────────────────────────────────────────────────────────────
#[inline] pub fn asin(e: f32) -> f32 { e.asin() }
#[inline] pub fn asin_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.asin(), e.0.y.asin()) }
#[inline] pub fn asin_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.asin(), e.0.y.asin(), e.0.z.asin()) }
#[inline] pub fn asin_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.asin(), e.0.y.asin(), e.0.z.asin(), e.0.w.asin()) }

// ── asinh ────────────────────────────────────────────────────────────────────
#[inline] pub fn asinh(e: f32) -> f32 { e.asinh() }
#[inline] pub fn asinh_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.asinh(), e.0.y.asinh()) }
#[inline] pub fn asinh_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.asinh(), e.0.y.asinh(), e.0.z.asinh()) }
#[inline] pub fn asinh_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.asinh(), e.0.y.asinh(), e.0.z.asinh(), e.0.w.asinh()) }

// ── atan ─────────────────────────────────────────────────────────────────────
#[inline] pub fn atan(e: f32) -> f32 { e.atan() }
#[inline] pub fn atan_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.atan(), e.0.y.atan()) }
#[inline] pub fn atan_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.atan(), e.0.y.atan(), e.0.z.atan()) }
#[inline] pub fn atan_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.atan(), e.0.y.atan(), e.0.z.atan(), e.0.w.atan()) }

// ── atanh ────────────────────────────────────────────────────────────────────
#[inline] pub fn atanh(e: f32) -> f32 { e.atanh() }
#[inline] pub fn atanh_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.atanh(), e.0.y.atanh()) }
#[inline] pub fn atanh_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.atanh(), e.0.y.atanh(), e.0.z.atanh()) }
#[inline] pub fn atanh_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.atanh(), e.0.y.atanh(), e.0.z.atanh(), e.0.w.atanh()) }

// ── atan2 ────────────────────────────────────────────────────────────────────
#[inline] pub fn atan2(y: f32, x: f32) -> f32 { y.atan2(x) }
#[inline] pub fn atan2_vec2(y: Vec2, x: Vec2) -> Vec2 { Vec2::new(y.0.x.atan2(x.0.x), y.0.y.atan2(x.0.y)) }
#[inline] pub fn atan2_vec3(y: Vec3, x: Vec3) -> Vec3 { Vec3::new(y.0.x.atan2(x.0.x), y.0.y.atan2(x.0.y), y.0.z.atan2(x.0.z)) }
#[inline] pub fn atan2_vec4(y: Vec4, x: Vec4) -> Vec4 { Vec4::new(y.0.x.atan2(x.0.x), y.0.y.atan2(x.0.y), y.0.z.atan2(x.0.z), y.0.w.atan2(x.0.w)) }

// ── ceil ─────────────────────────────────────────────────────────────────────
#[inline] pub fn ceil(e: f32) -> f32 { e.ceil() }
#[inline] pub fn ceil_vec2(e: Vec2) -> Vec2 { Vec2(e.0.ceil()) }
#[inline] pub fn ceil_vec3(e: Vec3) -> Vec3 { Vec3(e.0.ceil()) }
#[inline] pub fn ceil_vec4(e: Vec4) -> Vec4 { Vec4(e.0.ceil()) }

// ── clamp ────────────────────────────────────────────────────────────────────
#[inline] pub fn clamp_f32(e: f32, low: f32, high: f32) -> f32 { e.clamp(low, high) }
#[inline] pub fn clamp_i32(e: i32, low: i32, high: i32) -> i32 { e.clamp(low, high) }
#[inline] pub fn clamp_u32(e: u32, low: u32, high: u32) -> u32 { e.clamp(low, high) }
#[inline] pub fn clamp_vec2(e: Vec2, low: Vec2, high: Vec2) -> Vec2 { Vec2(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_vec3(e: Vec3, low: Vec3, high: Vec3) -> Vec3 { Vec3(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_vec4(e: Vec4, low: Vec4, high: Vec4) -> Vec4 { Vec4(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_ivec2(e: IVec2, low: IVec2, high: IVec2) -> IVec2 { IVec2(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_ivec3(e: IVec3, low: IVec3, high: IVec3) -> IVec3 { IVec3(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_ivec4(e: IVec4, low: IVec4, high: IVec4) -> IVec4 { IVec4(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_uvec2(e: UVec2, low: UVec2, high: UVec2) -> UVec2 { UVec2(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_uvec3(e: UVec3, low: UVec3, high: UVec3) -> UVec3 { UVec3(e.0.clamp(low.0, high.0)) }
#[inline] pub fn clamp_uvec4(e: UVec4, low: UVec4, high: UVec4) -> UVec4 { UVec4(e.0.clamp(low.0, high.0)) }

// ── cos ──────────────────────────────────────────────────────────────────────
#[inline] pub fn cos(e: f32) -> f32 { e.cos() }
#[inline] pub fn cos_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.cos(), e.0.y.cos()) }
#[inline] pub fn cos_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.cos(), e.0.y.cos(), e.0.z.cos()) }
#[inline] pub fn cos_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.cos(), e.0.y.cos(), e.0.z.cos(), e.0.w.cos()) }

// ── cosh ─────────────────────────────────────────────────────────────────────
#[inline] pub fn cosh(e: f32) -> f32 { e.cosh() }
#[inline] pub fn cosh_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.cosh(), e.0.y.cosh()) }
#[inline] pub fn cosh_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.cosh(), e.0.y.cosh(), e.0.z.cosh()) }
#[inline] pub fn cosh_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.cosh(), e.0.y.cosh(), e.0.z.cosh(), e.0.w.cosh()) }

// ── countLeadingZeros ────────────────────────────────────────────────────────
#[inline] pub fn count_leading_zeros_u32(e: u32) -> u32 { e.leading_zeros() }
#[inline] pub fn count_leading_zeros_i32(e: i32) -> i32 { e.leading_zeros() as i32 }

// ── countOneBits ─────────────────────────────────────────────────────────────
#[inline] pub fn count_one_bits_u32(e: u32) -> u32 { e.count_ones() }
#[inline] pub fn count_one_bits_i32(e: i32) -> i32 { e.count_ones() as i32 }

// ── countTrailingZeros ───────────────────────────────────────────────────────
#[inline] pub fn count_trailing_zeros_u32(e: u32) -> u32 { e.trailing_zeros() }
#[inline] pub fn count_trailing_zeros_i32(e: i32) -> i32 { e.trailing_zeros() as i32 }

// ── cross ────────────────────────────────────────────────────────────────────
/// `cross(a, b)` – 3D cross product.
#[inline] pub fn cross(a: Vec3, b: Vec3) -> Vec3 { Vec3(a.0.cross(b.0)) }
#[inline] pub fn cross_dvec3(a: DVec3, b: DVec3) -> DVec3 { DVec3(a.0.cross(b.0)) }

// ── degrees ──────────────────────────────────────────────────────────────────
#[inline] pub fn degrees(e: f32) -> f32 { e.to_degrees() }
#[inline] pub fn degrees_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.to_degrees(), e.0.y.to_degrees()) }
#[inline] pub fn degrees_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.to_degrees(), e.0.y.to_degrees(), e.0.z.to_degrees()) }
#[inline] pub fn degrees_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.to_degrees(), e.0.y.to_degrees(), e.0.z.to_degrees(), e.0.w.to_degrees()) }

// ── determinant ──────────────────────────────────────────────────────────────
#[inline] pub fn determinant_mat2(m: Mat2) -> f32 { m.0.determinant() }
#[inline] pub fn determinant_mat3(m: Mat3) -> f32 { m.0.determinant() }
#[inline] pub fn determinant_mat4(m: Mat4) -> f32 { m.0.determinant() }
#[inline] pub fn determinant_dmat2(m: DMat2) -> f64 { m.0.determinant() }
#[inline] pub fn determinant_dmat3(m: DMat3) -> f64 { m.0.determinant() }
#[inline] pub fn determinant_dmat4(m: DMat4) -> f64 { m.0.determinant() }

// ── distance ─────────────────────────────────────────────────────────────────
#[inline] pub fn distance_f32(e1: f32, e2: f32) -> f32 { (e1 - e2).abs() }
#[inline] pub fn distance_vec2(e1: Vec2, e2: Vec2) -> f32 { e1.0.distance(e2.0) }
#[inline] pub fn distance_vec3(e1: Vec3, e2: Vec3) -> f32 { e1.0.distance(e2.0) }
#[inline] pub fn distance_vec4(e1: Vec4, e2: Vec4) -> f32 { e1.0.distance(e2.0) }

// ── dot ──────────────────────────────────────────────────────────────────────
#[inline] pub fn dot_vec2(e1: Vec2, e2: Vec2) -> f32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_vec3(e1: Vec3, e2: Vec3) -> f32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_vec4(e1: Vec4, e2: Vec4) -> f32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_ivec2(e1: IVec2, e2: IVec2) -> i32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_ivec3(e1: IVec3, e2: IVec3) -> i32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_ivec4(e1: IVec4, e2: IVec4) -> i32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_uvec2(e1: UVec2, e2: UVec2) -> u32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_uvec3(e1: UVec3, e2: UVec3) -> u32 { e1.0.dot(e2.0) }
#[inline] pub fn dot_uvec4(e1: UVec4, e2: UVec4) -> u32 { e1.0.dot(e2.0) }

// ── exp / exp2 ───────────────────────────────────────────────────────────────
#[inline] pub fn exp(e: f32) -> f32 { e.exp() }
#[inline] pub fn exp_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.exp(), e.0.y.exp()) }
#[inline] pub fn exp_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.exp(), e.0.y.exp(), e.0.z.exp()) }
#[inline] pub fn exp_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.exp(), e.0.y.exp(), e.0.z.exp(), e.0.w.exp()) }

#[inline] pub fn exp2(e: f32) -> f32 { e.exp2() }
#[inline] pub fn exp2_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.exp2(), e.0.y.exp2()) }
#[inline] pub fn exp2_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.exp2(), e.0.y.exp2(), e.0.z.exp2()) }
#[inline] pub fn exp2_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.exp2(), e.0.y.exp2(), e.0.z.exp2(), e.0.w.exp2()) }

// ── faceForward ──────────────────────────────────────────────────────────────
/// `faceForward(e1, e2, e3)` – returns `e1` if `dot(e2,e3) < 0`, else `-e1`.
#[inline] pub fn face_forward_vec2(e1: Vec2, e2: Vec2, e3: Vec2) -> Vec2 {
    if dot_vec2(e2, e3) < 0.0 { e1 } else { -e1 }
}
#[inline] pub fn face_forward_vec3(e1: Vec3, e2: Vec3, e3: Vec3) -> Vec3 {
    if dot_vec3(e2, e3) < 0.0 { e1 } else { -e1 }
}
#[inline] pub fn face_forward_vec4(e1: Vec4, e2: Vec4, e3: Vec4) -> Vec4 {
    if dot_vec4(e2, e3) < 0.0 { e1 } else { -e1 }
}

// ── firstLeadingBit / firstTrailingBit ───────────────────────────────────────
#[inline] pub fn first_leading_bit_u32(e: u32) -> u32 { if e == 0 { u32::MAX } else { 31 - e.leading_zeros() } }
#[inline] pub fn first_leading_bit_i32(e: i32) -> i32 {
    if e == 0 || e == -1 { -1 } else {
        let bit = if e < 0 { 31 - (!e as u32).leading_zeros() } else { 31 - (e as u32).leading_zeros() };
        bit as i32
    }
}
#[inline] pub fn first_trailing_bit_u32(e: u32) -> u32 { if e == 0 { u32::MAX } else { e.trailing_zeros() } }
#[inline] pub fn first_trailing_bit_i32(e: i32) -> i32 { if e == 0 { -1 } else { e.trailing_zeros() as i32 } }

// ── floor ────────────────────────────────────────────────────────────────────
#[inline] pub fn floor(e: f32) -> f32 { e.floor() }
#[inline] pub fn floor_vec2(e: Vec2) -> Vec2 { Vec2(e.0.floor()) }
#[inline] pub fn floor_vec3(e: Vec3) -> Vec3 { Vec3(e.0.floor()) }
#[inline] pub fn floor_vec4(e: Vec4) -> Vec4 { Vec4(e.0.floor()) }

// ── fma ──────────────────────────────────────────────────────────────────────
#[inline] pub fn fma(e1: f32, e2: f32, e3: f32) -> f32 { e1.mul_add(e2, e3) }
#[inline] pub fn fma_vec2(e1: Vec2, e2: Vec2, e3: Vec2) -> Vec2 {
    Vec2::new(fma(e1.0.x, e2.0.x, e3.0.x), fma(e1.0.y, e2.0.y, e3.0.y))
}
#[inline] pub fn fma_vec3(e1: Vec3, e2: Vec3, e3: Vec3) -> Vec3 {
    Vec3::new(fma(e1.0.x, e2.0.x, e3.0.x), fma(e1.0.y, e2.0.y, e3.0.y), fma(e1.0.z, e2.0.z, e3.0.z))
}
#[inline] pub fn fma_vec4(e1: Vec4, e2: Vec4, e3: Vec4) -> Vec4 {
    Vec4::new(fma(e1.0.x, e2.0.x, e3.0.x), fma(e1.0.y, e2.0.y, e3.0.y), fma(e1.0.z, e2.0.z, e3.0.z), fma(e1.0.w, e2.0.w, e3.0.w))
}

// ── fract ────────────────────────────────────────────────────────────────────
#[inline] pub fn fract(e: f32) -> f32 { e.fract() }
#[inline] pub fn fract_vec2(e: Vec2) -> Vec2 { Vec2(e.0.fract()) }
#[inline] pub fn fract_vec3(e: Vec3) -> Vec3 { Vec3(e.0.fract()) }
#[inline] pub fn fract_vec4(e: Vec4) -> Vec4 { Vec4(e.0.fract()) }

// ── frexp ────────────────────────────────────────────────────────────────────
/// Returns `(fract, exp)` such that `e = fract * 2^exp` with `fract` in `[0.5, 1.0)`.
#[inline] pub fn frexp(e: f32) -> (f32, i32) {
    if e == 0.0 { return (0.0, 0); }
    let bits = e.to_bits();
    let exp = ((bits >> 23) & 0xFF) as i32 - 126;
    let fract = f32::from_bits((bits & 0x807FFFFF) | 0x3F000000);
    (fract, exp)
}

// ── insertBits / extractBits ─────────────────────────────────────────────────
#[inline] pub fn insert_bits_u32(e: u32, newbits: u32, offset: u32, count: u32) -> u32 {
    let w = 32u32;
    let o = offset.min(w);
    let c = count.min(w - o);
    if c == 0 { return e; }
    let mask = ((1u64 << c) - 1) as u32;
    (e & !(mask << o)) | ((newbits & mask) << o)
}
#[inline] pub fn extract_bits_u32(e: u32, offset: u32, count: u32) -> u32 {
    let o = offset.min(32);
    let c = count.min(32 - o);
    if c == 0 { 0 } else { (e >> o) & ((1u64.wrapping_shl(c) - 1) as u32) }
}
#[inline] pub fn extract_bits_i32(e: i32, offset: u32, count: u32) -> i32 {
    let o = offset.min(32);
    let c = count.min(32 - o);
    if c == 0 { return 0; }
    let shifted = (e >> o) as u32;
    let mask = (1u64.wrapping_shl(c) - 1) as u32;
    let raw = shifted & mask;
    // sign-extend
    if c < 32 && (raw >> (c - 1)) & 1 != 0 {
        (raw | (u32::MAX << c)) as i32
    } else {
        raw as i32
    }
}

// ── inverseSqrt ──────────────────────────────────────────────────────────────
#[inline] pub fn inverse_sqrt(e: f32) -> f32 { 1.0 / e.sqrt() }
#[inline] pub fn inverse_sqrt_vec2(e: Vec2) -> Vec2 { Vec2(e.0.powf(-0.5)) }
#[inline] pub fn inverse_sqrt_vec3(e: Vec3) -> Vec3 { Vec3(e.0.powf(-0.5)) }
#[inline] pub fn inverse_sqrt_vec4(e: Vec4) -> Vec4 { Vec4(e.0.powf(-0.5)) }

// ── ldexp ────────────────────────────────────────────────────────────────────
#[inline] pub fn ldexp(e1: f32, e2: i32) -> f32 { e1 * (2.0f32).powi(e2) }

// ── length ───────────────────────────────────────────────────────────────────
#[inline] pub fn length_f32(e: f32) -> f32 { e.abs() }
#[inline] pub fn length_vec2(e: Vec2) -> f32 { e.0.length() }
#[inline] pub fn length_vec3(e: Vec3) -> f32 { e.0.length() }
#[inline] pub fn length_vec4(e: Vec4) -> f32 { e.0.length() }

// ── log / log2 ───────────────────────────────────────────────────────────────
#[inline] pub fn log(e: f32) -> f32 { e.ln() }
#[inline] pub fn log_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.ln(), e.0.y.ln()) }
#[inline] pub fn log_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.ln(), e.0.y.ln(), e.0.z.ln()) }
#[inline] pub fn log_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.ln(), e.0.y.ln(), e.0.z.ln(), e.0.w.ln()) }

#[inline] pub fn log2(e: f32) -> f32 { e.log2() }
#[inline] pub fn log2_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.log2(), e.0.y.log2()) }
#[inline] pub fn log2_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.log2(), e.0.y.log2(), e.0.z.log2()) }
#[inline] pub fn log2_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.log2(), e.0.y.log2(), e.0.z.log2(), e.0.w.log2()) }

// ── max / min ────────────────────────────────────────────────────────────────
#[inline] pub fn max_f32(e1: f32, e2: f32) -> f32 { e1.max(e2) }
#[inline] pub fn max_i32(e1: i32, e2: i32) -> i32 { e1.max(e2) }
#[inline] pub fn max_u32(e1: u32, e2: u32) -> u32 { e1.max(e2) }
#[inline] pub fn max_vec2(e1: Vec2, e2: Vec2) -> Vec2 { Vec2(e1.0.max(e2.0)) }
#[inline] pub fn max_vec3(e1: Vec3, e2: Vec3) -> Vec3 { Vec3(e1.0.max(e2.0)) }
#[inline] pub fn max_vec4(e1: Vec4, e2: Vec4) -> Vec4 { Vec4(e1.0.max(e2.0)) }
#[inline] pub fn max_ivec2(e1: IVec2, e2: IVec2) -> IVec2 { IVec2(e1.0.max(e2.0)) }
#[inline] pub fn max_ivec3(e1: IVec3, e2: IVec3) -> IVec3 { IVec3(e1.0.max(e2.0)) }
#[inline] pub fn max_ivec4(e1: IVec4, e2: IVec4) -> IVec4 { IVec4(e1.0.max(e2.0)) }
#[inline] pub fn max_uvec2(e1: UVec2, e2: UVec2) -> UVec2 { UVec2(e1.0.max(e2.0)) }
#[inline] pub fn max_uvec3(e1: UVec3, e2: UVec3) -> UVec3 { UVec3(e1.0.max(e2.0)) }
#[inline] pub fn max_uvec4(e1: UVec4, e2: UVec4) -> UVec4 { UVec4(e1.0.max(e2.0)) }

#[inline] pub fn min_f32(e1: f32, e2: f32) -> f32 { e1.min(e2) }
#[inline] pub fn min_i32(e1: i32, e2: i32) -> i32 { e1.min(e2) }
#[inline] pub fn min_u32(e1: u32, e2: u32) -> u32 { e1.min(e2) }
#[inline] pub fn min_vec2(e1: Vec2, e2: Vec2) -> Vec2 { Vec2(e1.0.min(e2.0)) }
#[inline] pub fn min_vec3(e1: Vec3, e2: Vec3) -> Vec3 { Vec3(e1.0.min(e2.0)) }
#[inline] pub fn min_vec4(e1: Vec4, e2: Vec4) -> Vec4 { Vec4(e1.0.min(e2.0)) }
#[inline] pub fn min_ivec2(e1: IVec2, e2: IVec2) -> IVec2 { IVec2(e1.0.min(e2.0)) }
#[inline] pub fn min_ivec3(e1: IVec3, e2: IVec3) -> IVec3 { IVec3(e1.0.min(e2.0)) }
#[inline] pub fn min_ivec4(e1: IVec4, e2: IVec4) -> IVec4 { IVec4(e1.0.min(e2.0)) }
#[inline] pub fn min_uvec2(e1: UVec2, e2: UVec2) -> UVec2 { UVec2(e1.0.min(e2.0)) }
#[inline] pub fn min_uvec3(e1: UVec3, e2: UVec3) -> UVec3 { UVec3(e1.0.min(e2.0)) }
#[inline] pub fn min_uvec4(e1: UVec4, e2: UVec4) -> UVec4 { UVec4(e1.0.min(e2.0)) }

// ── mix ──────────────────────────────────────────────────────────────────────
/// `mix(e1, e2, e3)` – linear interpolation: `e1*(1-e3) + e2*e3`.
#[inline] pub fn mix(e1: f32, e2: f32, t: f32) -> f32 { e1 + (e2 - e1) * t }
#[inline] pub fn mix_vec2(e1: Vec2, e2: Vec2, t: f32) -> Vec2 { Vec2(e1.0.lerp(e2.0, t)) }
#[inline] pub fn mix_vec3(e1: Vec3, e2: Vec3, t: f32) -> Vec3 { Vec3(e1.0.lerp(e2.0, t)) }
#[inline] pub fn mix_vec4(e1: Vec4, e2: Vec4, t: f32) -> Vec4 { Vec4(e1.0.lerp(e2.0, t)) }
/// Component-wise mix with vector `t`.
#[inline] pub fn mix_vec2_t(e1: Vec2, e2: Vec2, t: Vec2) -> Vec2 {
    Vec2::new(mix(e1.0.x, e2.0.x, t.0.x), mix(e1.0.y, e2.0.y, t.0.y))
}
#[inline] pub fn mix_vec3_t(e1: Vec3, e2: Vec3, t: Vec3) -> Vec3 {
    Vec3::new(mix(e1.0.x, e2.0.x, t.0.x), mix(e1.0.y, e2.0.y, t.0.y), mix(e1.0.z, e2.0.z, t.0.z))
}
#[inline] pub fn mix_vec4_t(e1: Vec4, e2: Vec4, t: Vec4) -> Vec4 {
    Vec4::new(mix(e1.0.x, e2.0.x, t.0.x), mix(e1.0.y, e2.0.y, t.0.y), mix(e1.0.z, e2.0.z, t.0.z), mix(e1.0.w, e2.0.w, t.0.w))
}

// ── modf ─────────────────────────────────────────────────────────────────────
/// `modf(e)` – returns `(fract, whole)`.
#[inline] pub fn modf(e: f32) -> (f32, f32) { let w = e.trunc(); (e - w, w) }

// ── normalize ────────────────────────────────────────────────────────────────
#[inline] pub fn normalize_vec2(e: Vec2) -> Vec2 { Vec2(e.0.normalize()) }
#[inline] pub fn normalize_vec3(e: Vec3) -> Vec3 { Vec3(e.0.normalize()) }
#[inline] pub fn normalize_vec4(e: Vec4) -> Vec4 { Vec4(e.0.normalize()) }

// ── pow ──────────────────────────────────────────────────────────────────────
#[inline] pub fn pow(e1: f32, e2: f32) -> f32 { e1.powf(e2) }
#[inline] pub fn pow_vec2(e1: Vec2, e2: Vec2) -> Vec2 { Vec2(e1.0.powf(e2.0.x)) } // note: vec powf is per-component
// Proper component-wise:
#[inline] pub fn pow_vec2_cw(e1: Vec2, e2: Vec2) -> Vec2 { Vec2::new(e1.0.x.powf(e2.0.x), e1.0.y.powf(e2.0.y)) }
#[inline] pub fn pow_vec3_cw(e1: Vec3, e2: Vec3) -> Vec3 { Vec3::new(e1.0.x.powf(e2.0.x), e1.0.y.powf(e2.0.y), e1.0.z.powf(e2.0.z)) }
#[inline] pub fn pow_vec4_cw(e1: Vec4, e2: Vec4) -> Vec4 { Vec4::new(e1.0.x.powf(e2.0.x), e1.0.y.powf(e2.0.y), e1.0.z.powf(e2.0.z), e1.0.w.powf(e2.0.w)) }

// ── quantizeToF16 ────────────────────────────────────────────────────────────
/// Quantize a f32 to the nearest f16 representable value (round-to-nearest-even).
#[inline] pub fn quantize_to_f16(e: f32) -> f32 {
    // Encode to f16 bits then decode back.
    let h = half::f16::from_f32(e);
    h.to_f32()
}

// ── radians ──────────────────────────────────────────────────────────────────
#[inline] pub fn radians(e: f32) -> f32 { e.to_radians() }
#[inline] pub fn radians_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.to_radians(), e.0.y.to_radians()) }
#[inline] pub fn radians_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.to_radians(), e.0.y.to_radians(), e.0.z.to_radians()) }
#[inline] pub fn radians_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.to_radians(), e.0.y.to_radians(), e.0.z.to_radians(), e.0.w.to_radians()) }

// ── reflect ──────────────────────────────────────────────────────────────────
/// `reflect(e1, e2)` – reflect incident vector `e1` around normal `e2`.
#[inline] pub fn reflect_vec2(e1: Vec2, e2: Vec2) -> Vec2 { Vec2(e1.0 - 2.0 * dot_vec2(e1, e2) * e2.0) }
#[inline] pub fn reflect_vec3(e1: Vec3, e2: Vec3) -> Vec3 { Vec3(e1.0 - 2.0 * dot_vec3(e1, e2) * e2.0) }
#[inline] pub fn reflect_vec4(e1: Vec4, e2: Vec4) -> Vec4 { Vec4(e1.0 - 2.0 * dot_vec4(e1, e2) * e2.0) }

// ── refract ──────────────────────────────────────────────────────────────────
/// `refract(e1, e2, e3)` – refract incident `e1` through normal `e2` with ratio `e3`.
#[inline] pub fn refract_vec2(e1: Vec2, e2: Vec2, e3: f32) -> Vec2 {
    let k = 1.0 - e3 * e3 * (1.0 - dot_vec2(e2, e1).powi(2));
    if k < 0.0 { Vec2::splat(0.0) } else { Vec2(e3 * e1.0 - (e3 * dot_vec2(e2, e1) + k.sqrt()) * e2.0) }
}
#[inline] pub fn refract_vec3(e1: Vec3, e2: Vec3, e3: f32) -> Vec3 {
    let k = 1.0 - e3 * e3 * (1.0 - dot_vec3(e2, e1).powi(2));
    if k < 0.0 { Vec3::splat(0.0) } else { Vec3(e3 * e1.0 - (e3 * dot_vec3(e2, e1) + k.sqrt()) * e2.0) }
}
#[inline] pub fn refract_vec4(e1: Vec4, e2: Vec4, e3: f32) -> Vec4 {
    let k = 1.0 - e3 * e3 * (1.0 - dot_vec4(e2, e1).powi(2));
    if k < 0.0 { Vec4::splat(0.0) } else { Vec4(e3 * e1.0 - (e3 * dot_vec4(e2, e1) + k.sqrt()) * e2.0) }
}

// ── reverseBits ──────────────────────────────────────────────────────────────
#[inline] pub fn reverse_bits_u32(e: u32) -> u32 { e.reverse_bits() }
#[inline] pub fn reverse_bits_i32(e: i32) -> i32 { e.reverse_bits() }

// ── round ────────────────────────────────────────────────────────────────────
/// WGSL `round` uses round-half-to-even (banker's rounding).
#[inline] pub fn round(e: f32) -> f32 { (e * 0.5).round() * 2.0 - (e * 0.5).floor() * 2.0 + e - e.round() + e.round() }
// Simpler, correct implementation:
#[inline] pub fn round_wgsl(e: f32) -> f32 {
    let fl = e.floor();
    let diff = e - fl;
    if diff < 0.5 { fl } else if diff > 0.5 { fl + 1.0 } else if (fl as i64) % 2 == 0 { fl } else { fl + 1.0 }
}
#[inline] pub fn round_vec2(e: Vec2) -> Vec2 { Vec2::new(round_wgsl(e.0.x), round_wgsl(e.0.y)) }
#[inline] pub fn round_vec3(e: Vec3) -> Vec3 { Vec3::new(round_wgsl(e.0.x), round_wgsl(e.0.y), round_wgsl(e.0.z)) }
#[inline] pub fn round_vec4(e: Vec4) -> Vec4 { Vec4::new(round_wgsl(e.0.x), round_wgsl(e.0.y), round_wgsl(e.0.z), round_wgsl(e.0.w)) }

// ── saturate ─────────────────────────────────────────────────────────────────
/// `saturate(e)` – clamp to `[0, 1]`.
#[inline] pub fn saturate(e: f32) -> f32 { e.clamp(0.0, 1.0) }
#[inline] pub fn saturate_vec2(e: Vec2) -> Vec2 { Vec2(e.0.clamp(glam::Vec2::ZERO, glam::Vec2::ONE)) }
#[inline] pub fn saturate_vec3(e: Vec3) -> Vec3 { Vec3(e.0.clamp(glam::Vec3::ZERO, glam::Vec3::ONE)) }
#[inline] pub fn saturate_vec4(e: Vec4) -> Vec4 { Vec4(e.0.clamp(glam::Vec4::ZERO, glam::Vec4::ONE)) }

// ── sign ─────────────────────────────────────────────────────────────────────
#[inline] pub fn sign_f32(e: f32) -> f32 { e.signum() }
#[inline] pub fn sign_i32(e: i32) -> i32 { e.signum() }
#[inline] pub fn sign_vec2(e: Vec2) -> Vec2 { Vec2(e.0.signum()) }
#[inline] pub fn sign_vec3(e: Vec3) -> Vec3 { Vec3(e.0.signum()) }
#[inline] pub fn sign_vec4(e: Vec4) -> Vec4 { Vec4(e.0.signum()) }

// ── sin ──────────────────────────────────────────────────────────────────────
#[inline] pub fn sin(e: f32) -> f32 { e.sin() }
#[inline] pub fn sin_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.sin(), e.0.y.sin()) }
#[inline] pub fn sin_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.sin(), e.0.y.sin(), e.0.z.sin()) }
#[inline] pub fn sin_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.sin(), e.0.y.sin(), e.0.z.sin(), e.0.w.sin()) }

// ── sinh ─────────────────────────────────────────────────────────────────────
#[inline] pub fn sinh(e: f32) -> f32 { e.sinh() }
#[inline] pub fn sinh_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.sinh(), e.0.y.sinh()) }
#[inline] pub fn sinh_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.sinh(), e.0.y.sinh(), e.0.z.sinh()) }
#[inline] pub fn sinh_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.sinh(), e.0.y.sinh(), e.0.z.sinh(), e.0.w.sinh()) }

// ── smoothstep ───────────────────────────────────────────────────────────────
/// `smoothstep(low, high, x)` – Hermite interpolation.
#[inline] pub fn smoothstep(low: f32, high: f32, x: f32) -> f32 {
    let t = saturate((x - low) / (high - low));
    t * t * (3.0 - 2.0 * t)
}
#[inline] pub fn smoothstep_vec2(low: Vec2, high: Vec2, x: Vec2) -> Vec2 {
    Vec2::new(smoothstep(low.0.x, high.0.x, x.0.x), smoothstep(low.0.y, high.0.y, x.0.y))
}
#[inline] pub fn smoothstep_vec3(low: Vec3, high: Vec3, x: Vec3) -> Vec3 {
    Vec3::new(smoothstep(low.0.x, high.0.x, x.0.x), smoothstep(low.0.y, high.0.y, x.0.y), smoothstep(low.0.z, high.0.z, x.0.z))
}
#[inline] pub fn smoothstep_vec4(low: Vec4, high: Vec4, x: Vec4) -> Vec4 {
    Vec4::new(smoothstep(low.0.x, high.0.x, x.0.x), smoothstep(low.0.y, high.0.y, x.0.y), smoothstep(low.0.z, high.0.z, x.0.z), smoothstep(low.0.w, high.0.w, x.0.w))
}

// ── sqrt ─────────────────────────────────────────────────────────────────────
#[inline] pub fn sqrt(e: f32) -> f32 { e.sqrt() }
#[inline] pub fn sqrt_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.sqrt(), e.0.y.sqrt()) }
#[inline] pub fn sqrt_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.sqrt(), e.0.y.sqrt(), e.0.z.sqrt()) }
#[inline] pub fn sqrt_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.sqrt(), e.0.y.sqrt(), e.0.z.sqrt(), e.0.w.sqrt()) }

// ── step ─────────────────────────────────────────────────────────────────────
/// `step(edge, x)` – returns 0.0 if `x < edge`, else 1.0.
#[inline] pub fn step(edge: f32, x: f32) -> f32 { if x < edge { 0.0 } else { 1.0 } }
#[inline] pub fn step_vec2(edge: Vec2, x: Vec2) -> Vec2 { Vec2::new(step(edge.0.x, x.0.x), step(edge.0.y, x.0.y)) }
#[inline] pub fn step_vec3(edge: Vec3, x: Vec3) -> Vec3 { Vec3::new(step(edge.0.x, x.0.x), step(edge.0.y, x.0.y), step(edge.0.z, x.0.z)) }
#[inline] pub fn step_vec4(edge: Vec4, x: Vec4) -> Vec4 { Vec4::new(step(edge.0.x, x.0.x), step(edge.0.y, x.0.y), step(edge.0.z, x.0.z), step(edge.0.w, x.0.w)) }

// ── tan ──────────────────────────────────────────────────────────────────────
#[inline] pub fn tan(e: f32) -> f32 { e.tan() }
#[inline] pub fn tan_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.tan(), e.0.y.tan()) }
#[inline] pub fn tan_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.tan(), e.0.y.tan(), e.0.z.tan()) }
#[inline] pub fn tan_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.tan(), e.0.y.tan(), e.0.z.tan(), e.0.w.tan()) }

// ── tanh ─────────────────────────────────────────────────────────────────────
#[inline] pub fn tanh(e: f32) -> f32 { e.tanh() }
#[inline] pub fn tanh_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.tanh(), e.0.y.tanh()) }
#[inline] pub fn tanh_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.tanh(), e.0.y.tanh(), e.0.z.tanh()) }
#[inline] pub fn tanh_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.tanh(), e.0.y.tanh(), e.0.z.tanh(), e.0.w.tanh()) }

// ── transpose ────────────────────────────────────────────────────────────────
#[inline] pub fn transpose_mat2(m: Mat2) -> Mat2 { Mat2(m.0.transpose()) }
#[inline] pub fn transpose_mat3(m: Mat3) -> Mat3 { Mat3(m.0.transpose()) }
#[inline] pub fn transpose_mat4(m: Mat4) -> Mat4 { Mat4(m.0.transpose()) }
#[inline] pub fn transpose_dmat2(m: DMat2) -> DMat2 { DMat2(m.0.transpose()) }
#[inline] pub fn transpose_dmat3(m: DMat3) -> DMat3 { DMat3(m.0.transpose()) }
#[inline] pub fn transpose_dmat4(m: DMat4) -> DMat4 { DMat4(m.0.transpose()) }

// ── trunc ────────────────────────────────────────────────────────────────────
#[inline] pub fn trunc(e: f32) -> f32 { e.trunc() }
#[inline] pub fn trunc_vec2(e: Vec2) -> Vec2 { Vec2::new(e.0.x.trunc(), e.0.y.trunc()) }
#[inline] pub fn trunc_vec3(e: Vec3) -> Vec3 { Vec3::new(e.0.x.trunc(), e.0.y.trunc(), e.0.z.trunc()) }
#[inline] pub fn trunc_vec4(e: Vec4) -> Vec4 { Vec4::new(e.0.x.trunc(), e.0.y.trunc(), e.0.z.trunc(), e.0.w.trunc()) }

// ─────────────────────────────────────────────────────────────────────────────
// Bit-packing built-ins
// ─────────────────────────────────────────────────────────────────────────────

/// `pack4x8snorm` – pack 4 f32 in [-1,1] into a u32 as 4 signed 8-bit values.
pub fn pack4x8snorm(e: Vec4) -> u32 {
    let pack = |f: f32| -> u32 { (f.clamp(-1.0, 1.0) * 127.0).round() as i8 as u8 as u32 };
    pack(e.0.x) | (pack(e.0.y) << 8) | (pack(e.0.z) << 16) | (pack(e.0.w) << 24)
}

/// `pack4x8unorm` – pack 4 f32 in [0,1] into a u32 as 4 unsigned 8-bit values.
pub fn pack4x8unorm(e: Vec4) -> u32 {
    let pack = |f: f32| -> u32 { (f.clamp(0.0, 1.0) * 255.0).round() as u8 as u32 };
    pack(e.0.x) | (pack(e.0.y) << 8) | (pack(e.0.z) << 16) | (pack(e.0.w) << 24)
}

/// `pack2x16snorm` – pack 2 f32 in [-1,1] into a u32 as 2 signed 16-bit values.
pub fn pack2x16snorm(e: Vec2) -> u32 {
    let pack = |f: f32| -> u32 { (f.clamp(-1.0, 1.0) * 32767.0).round() as i16 as u16 as u32 };
    pack(e.0.x) | (pack(e.0.y) << 16)
}

/// `pack2x16unorm` – pack 2 f32 in [0,1] into a u32 as 2 unsigned 16-bit values.
pub fn pack2x16unorm(e: Vec2) -> u32 {
    let pack = |f: f32| -> u32 { (f.clamp(0.0, 1.0) * 65535.0).round() as u16 as u32 };
    pack(e.0.x) | (pack(e.0.y) << 16)
}

/// `pack2x16float` – pack 2 f32 into a u32 as 2 f16 values.
pub fn pack2x16float(e: Vec2) -> u32 {
    let lo = half::f16::from_f32(e.0.x).to_bits() as u32;
    let hi = half::f16::from_f32(e.0.y).to_bits() as u32;
    lo | (hi << 16)
}

/// `unpack4x8snorm`
pub fn unpack4x8snorm(e: u32) -> Vec4 {
    let unpack = |b: u32| -> f32 { ((b as u8) as i8 as f32 / 127.0).clamp(-1.0, 1.0) };
    Vec4::new(unpack(e), unpack(e >> 8), unpack(e >> 16), unpack(e >> 24))
}

/// `unpack4x8unorm`
pub fn unpack4x8unorm(e: u32) -> Vec4 {
    let unpack = |b: u32| -> f32 { (b as u8) as f32 / 255.0 };
    Vec4::new(unpack(e), unpack(e >> 8), unpack(e >> 16), unpack(e >> 24))
}

/// `unpack2x16snorm`
pub fn unpack2x16snorm(e: u32) -> Vec2 {
    let unpack = |b: u32| -> f32 { ((b as u16) as i16 as f32 / 32767.0).clamp(-1.0, 1.0) };
    Vec2::new(unpack(e), unpack(e >> 16))
}

/// `unpack2x16unorm`
pub fn unpack2x16unorm(e: u32) -> Vec2 {
    let unpack = |b: u32| -> f32 { (b as u16) as f32 / 65535.0 };
    Vec2::new(unpack(e), unpack(e >> 16))
}

/// `unpack2x16float`
pub fn unpack2x16float(e: u32) -> Vec2 {
    let lo = half::f16::from_bits(e as u16).to_f32();
    let hi = half::f16::from_bits((e >> 16) as u16).to_f32();
    Vec2::new(lo, hi)
}

// ─────────────────────────────────────────────────────────────────────────────
// Comparison helpers (WGSL component-wise comparisons return BVec)
// ─────────────────────────────────────────────────────────────────────────────

pub fn equal_vec2(a: Vec2, b: Vec2) -> BVec2 { BVec2::new(a.0.x == b.0.x, a.0.y == b.0.y) }
pub fn equal_vec3(a: Vec3, b: Vec3) -> BVec3 { BVec3::new(a.0.x == b.0.x, a.0.y == b.0.y, a.0.z == b.0.z) }
pub fn equal_vec4(a: Vec4, b: Vec4) -> BVec4 { BVec4::new(a.0.x == b.0.x, a.0.y == b.0.y, a.0.z == b.0.z, a.0.w == b.0.w) }

pub fn not_equal_vec2(a: Vec2, b: Vec2) -> BVec2 { BVec2::new(a.0.x != b.0.x, a.0.y != b.0.y) }
pub fn not_equal_vec3(a: Vec3, b: Vec3) -> BVec3 { BVec3::new(a.0.x != b.0.x, a.0.y != b.0.y, a.0.z != b.0.z) }
pub fn not_equal_vec4(a: Vec4, b: Vec4) -> BVec4 { BVec4::new(a.0.x != b.0.x, a.0.y != b.0.y, a.0.z != b.0.z, a.0.w != b.0.w) }

pub fn less_than_vec2(a: Vec2, b: Vec2) -> BVec2 { BVec2::new(a.0.x < b.0.x, a.0.y < b.0.y) }
pub fn less_than_vec3(a: Vec3, b: Vec3) -> BVec3 { BVec3::new(a.0.x < b.0.x, a.0.y < b.0.y, a.0.z < b.0.z) }
pub fn less_than_vec4(a: Vec4, b: Vec4) -> BVec4 { BVec4::new(a.0.x < b.0.x, a.0.y < b.0.y, a.0.z < b.0.z, a.0.w < b.0.w) }

pub fn less_than_equal_vec2(a: Vec2, b: Vec2) -> BVec2 { BVec2::new(a.0.x <= b.0.x, a.0.y <= b.0.y) }
pub fn less_than_equal_vec3(a: Vec3, b: Vec3) -> BVec3 { BVec3::new(a.0.x <= b.0.x, a.0.y <= b.0.y, a.0.z <= b.0.z) }
pub fn less_than_equal_vec4(a: Vec4, b: Vec4) -> BVec4 { BVec4::new(a.0.x <= b.0.x, a.0.y <= b.0.y, a.0.z <= b.0.z, a.0.w <= b.0.w) }

pub fn greater_than_vec2(a: Vec2, b: Vec2) -> BVec2 { BVec2::new(a.0.x > b.0.x, a.0.y > b.0.y) }
pub fn greater_than_vec3(a: Vec3, b: Vec3) -> BVec3 { BVec3::new(a.0.x > b.0.x, a.0.y > b.0.y, a.0.z > b.0.z) }
pub fn greater_than_vec4(a: Vec4, b: Vec4) -> BVec4 { BVec4::new(a.0.x > b.0.x, a.0.y > b.0.y, a.0.z > b.0.z, a.0.w > b.0.w) }

pub fn greater_than_equal_vec2(a: Vec2, b: Vec2) -> BVec2 { BVec2::new(a.0.x >= b.0.x, a.0.y >= b.0.y) }
pub fn greater_than_equal_vec3(a: Vec3, b: Vec3) -> BVec3 { BVec3::new(a.0.x >= b.0.x, a.0.y >= b.0.y, a.0.z >= b.0.z) }
pub fn greater_than_equal_vec4(a: Vec4, b: Vec4) -> BVec4 { BVec4::new(a.0.x >= b.0.x, a.0.y >= b.0.y, a.0.z >= b.0.z, a.0.w >= b.0.w) }

// ─────────────────────────────────────────────────────────────────────────────
// Convenience From/Into conversions between wrapper and glam types
// ─────────────────────────────────────────────────────────────────────────────
macro_rules! impl_from_inner {
    ($Outer:ty, $Inner:ty) => {
        impl From<$Inner> for $Outer { fn from(v: $Inner) -> Self { Self(v) } }
        impl From<$Outer> for $Inner { fn from(v: $Outer) -> Self { v.0 } }
    };
}

impl_from_inner!(Vec2, glam::Vec2);
impl_from_inner!(Vec3, glam::Vec3);
impl_from_inner!(Vec4, glam::Vec4);
impl_from_inner!(DVec2, glam::DVec2);
impl_from_inner!(DVec3, glam::DVec3);
impl_from_inner!(DVec4, glam::DVec4);
impl_from_inner!(IVec2, glam::IVec2);
impl_from_inner!(IVec3, glam::IVec3);
impl_from_inner!(IVec4, glam::IVec4);
impl_from_inner!(UVec2, glam::UVec2);
impl_from_inner!(UVec3, glam::UVec3);
impl_from_inner!(UVec4, glam::UVec4);
impl_from_inner!(Mat2, glam::Mat2);
impl_from_inner!(Mat3, glam::Mat3);
impl_from_inner!(Mat4, glam::Mat4);
impl_from_inner!(DMat2, glam::DMat2);
impl_from_inner!(DMat3, glam::DMat3);
impl_from_inner!(DMat4, glam::DMat4);

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_arithmetic() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        let c = a + b;
        assert_eq!(c, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_dot() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(dot_vec3(a, b), 0.0);
    }

    #[test]
    fn test_cross() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = cross(x, y);
        assert!((z.0.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let v = Vec3::new(3.0, 0.0, 0.0);
        let n = normalize_vec3(v);
        assert!((n.0.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_scalar() {
        assert_eq!(clamp_f32(2.5, 0.0, 1.0), 1.0);
        assert_eq!(clamp_f32(-1.0, 0.0, 1.0), 0.0);
        assert_eq!(clamp_i32(5, -3, 3), 3);
    }

    #[test]
    fn test_mix() {
        assert!((mix(0.0, 10.0, 0.5) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_smoothstep() {
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn test_mat_mul_vec() {
        let m = Mat4::identity();
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m * v, v);
    }

    #[test]
    fn test_transpose() {
        let m = Mat2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let t = transpose_mat2(m);
        // column 0 of transpose should equal row 0 of original
        assert_eq!(t.col(0), Vec2::new(1.0, 3.0));
    }

    #[test]
    fn test_pack_unpack_unorm() {
        let v = Vec4::new(0.0, 0.5, 1.0, 0.25);
        let packed = pack4x8unorm(v);
        let unpacked = unpack4x8unorm(packed);
        assert!((unpacked.0.z - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_select() {
        assert_eq!(select(0.0f32, 1.0f32, true), 1.0);
        assert_eq!(select(0.0f32, 1.0f32, false), 0.0);
    }

    #[test]
    fn test_bit_ops() {
        assert_eq!(count_one_bits_u32(0b1011), 3);
        assert_eq!(reverse_bits_u32(1u32), 1u32 << 31);
        assert_eq!(extract_bits_u32(0b1101, 1, 2), 0b10);
        assert_eq!(insert_bits_u32(0b0000, 0b11, 2, 2), 0b1100);
    }
}