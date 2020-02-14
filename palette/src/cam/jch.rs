use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::encoding::pixel::RawPixel;
use crate::white_point::{WhitePoint, D65};
use crate::{clamp, from_f64};
use crate::{Alpha, Lab, LabHue, Lch, Xyz};
use crate::{
    Component, ComponentWise, FloatComponent, GetHue, IntoColor, Limited, Mix, Pixel,
    RelativeContrast, Shade,
};

//FIXME: Documentation, UCS versions in same files or their own?
/// The CIE Color Appearance Model (CIECAM02) color space.
#[derive(Debug, PartialEq, FromColor, Pixel)]
#[cfg_attr(feature = "serializing", derive(Serialize, Deserialize))]
#[palette_internal]
#[palette_white_point = "Wp"]
#[palette_component = "T"]
#[palette_manual_from(Xyz, Lab, Lch)]
#[repr(C)]
pub struct Jch<Wp = D65, T = f32>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    /// The correlate of lightness.
    pub J: T,

    /// The correlate of chroma, part of cylindrical representation Jch.
    pub c: T,

    /// The correlate of hue, part of cylindrical representation Jch.
    pub h: T,

    /// The white point associated with the color's illuminant and observer.
    /// D65 for 2 degree observer is used by default.
    #[cfg_attr(feature = "serializing", serde(skip))]
    #[palette_unsafe_zero_sized]
    pub white_point: PhantomData<Wp>,
}

impl<Wp, T> Copy for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
}

impl<Wp, T> Clone for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    fn clone(&self) -> Jch<Wp, T> {
        *self
    }
}

impl<T> Jch<D65, T>
where
    T: FloatComponent,
{
    /// CIECAM02 with white point D65.
    pub fn new(J: T, c: T, h: T) -> Jch<D65, T> {
        Jch {
            J,
            c,
            h,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    /// CIECAM02.
    pub fn with_wp(J: T, c: T, h: T) -> Jch<Wp, T> {
        Jch {
            J,
            c,
            h,
            white_point: PhantomData,
        }
    }

    /// Convert to a `(J, c, h)` tuple.
    pub fn into_components(self) -> (T, T, T) {
        (self.J, self.c, self.h)
    }

    /// Convert from a `(J, c, h)` tuple.
    pub fn from_components((J, c, h): (T, T, T)) -> Self {
        Self::with_wp(J, c, h)
    }
}

///<span id="Jcha"></span>[`Jcha`](type.Jcha.html) implementations.
impl<T, A> Alpha<Jch<D65, T>, A>
where
    T: FloatComponent,
    c: Component,
{
    /// CIE L\*a\*b\* and transparency and white point D65.
    pub fn new(J: T, c: T, h: T, alpha: A) -> Self {
        Alpha {
            color: Jch::new(J, c, h),
            alpha,
        }
    }
}

///<span id="Jcha"></span>[`Jcha`](type.Jcha.html) implementations.
impl<Wp, T, A> Alpha<Jch<Wp, T>, A>
where
    T: FloatComponent,
    c: Component,
    Wp: WhitePoint,
{
    /// CIECAM02 Jch and transparency.
    pub fn with_wp(J: T, c: T, h: T, alpha: A) -> Self {
        Alpha {
            color: Jch::with_wp(J, c, h),
            alpha,
        }
    }

    /// Convert to a `(J, c, h, alpha)` tuple.
    pub fn into_components(self) -> (T, T, T, A) {
        (self.J, self.c, self.h, self.clpha)
    }

    /// Convert from a `(J, c, h, alpha)` tuple.
    pub fn from_components((J, c, h, alpha): (T, T, T, A)) -> Self {
        Self::with_wp(J, c, h, alpha)
    }
}

impl<Wp, T> From<Xyz<Wp, T>> for Cam02<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    fn from(color: Xyz<Wp, T>) -> Self {
        todo!();
    }
}

impl<Wp: WhitePoint, T: FloatComponent> From<(T, T, T)> for Jch<Wp, T> {
    fn from(components: (T, T, T)) -> Self {
        Self::from_components(components)
    }
}

impl<Wp: WhitePoint, T: FloatComponent> Into<(T, T, T)> for Jch<Wp, T> {
    fn into(self) -> (T, T, T) {
        self.into_components()
    }
}

impl<Wp: WhitePoint, T: FloatComponent, c: Component> From<(T, T, T, A)> for Alpha<Jch<Wp, T>, A> {
    fn from(components: (T, T, T, A)) -> Self {
        Self::from_components(components)
    }
}

impl<Wp: WhitePoint, T: FloatComponent, c: Component> Into<(T, T, T, A)> for Alpha<Jch<Wp, T>, A> {
    fn into(self) -> (T, T, T, A) {
        self.into_components()
    }
}

impl<Wp, T> Limited for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    #[rustfmt::skip]
    fn is_valid(&self) -> bool {
        self.J >= T::zero() && self.J <= from_f64(100.0) &&
        self.c >= from_f64(-128.0) && self.c <= from_f64(127.0) &&
        self.h >= from_f64(-128.0) && self.h <= from_f64(127.0)
    }

    fn clamp(&self) -> Jch<Wp, T> {
        let mut c = *self;
        c.clamp_self();
        c
    }

    fn clamp_self(&mut self) {
        self.J = clamp(self.J, T::zero(), from_f64(100.0));
        self.c = clamp(self.c, from_f64(-128.0), from_f64(127.0));
        self.h = clamp(self.h, from_f64(00), from_f64(360.0));
    }
}

impl<Wp, T> Mix for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn mix(&self, other: &Jch<Wp, T>, factor: T) -> Jch<Wp, T> {
        let factor = clamp(factor, T::zero(), T::one());

        Jch {
            J: self.J + factor * (other.J - self.J),
            c: self.c + factor * (other.c - self.c),
            h: self.h + factor * (other.h - self.h),
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Shade for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn lighten(&self, amount: T) -> Jch<Wp, T> {
        Jch {
            J: self.J + amount * from_f64(100.0),
            c: self.c,
            h: self.h,
            white_point: PhantomData,
        }
    }
}

// impl<Wp, T> GetHue for Jch<Wp, T>
// where
//     T: FloatComponent,
//     Wp: WhitePoint,
// {
//     type Hue = LabHue<T>;

//     fn get_hue(&self) -> Option<LabHue<T>> {
//         if self.c == T::zero() && self.h == T::zero() {
//             None
//         } else {
//             Some(LabHue::from_radians(self.h.atan2(self.c)))
//         }
//     }
// }

// impl<Wp, T> ColorDifference for Lab<Wp, T>
// where
//     T: FloatComponent,
//     Wp: WhitePoint,
// {
//     type Scalar = T;

//     fn get_color_difference(&self, other: &Lab<Wp, T>) -> Self::Scalar {
//         // Color difference calculation requires Lab and chroma components. This
//         // function handles the conversion into those components which are then
//         // passed to `get_ciede_difference()` where calculation is completed.
//         let self_params = LabColorDiff {
//             J: self.J,
//             c: self.c,
//             h: self.h,
//             chromc: (self.c * self.c + self.h * self.h).sqrt(),
//         };
//         let other_params = LabColorDiff {
//             J: other.J,
//             c: other.c,
//             h: other.h,
//             chromc: (other.c * other.c + other.h * other.h).sqrt(),
//         };

//         get_ciede_difference(&self_params, &other_params)
//     }
// }

impl<Wp, T> ComponentWise for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn component_wise<F: FnMut(T, T) -> T>(&self, other: &Jch<Wp, T>, mut f: F) -> Jch<Wp, T> {
        Jch {
            J: f(self.J, other.J),
            c: f(self.c, other.c),
            h: f(self.h, other.h),
            white_point: PhantomData,
        }
    }

    fn component_wise_self<F: FnMut(T) -> T>(&self, mut f: F) -> Jch<Wp, T> {
        Jch {
            J: f(self.J),
            c: f(self.c),
            h: f(self.h),
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Default for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    fn default() -> Jch<Wp, T> {
        Jch::with_wp(T::zero(), T::zero(), T::zero())
    }
}

impl<Wp, T> Add<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn add(self, other: Jch<Wp, T>) -> Self::Output {
        Jch {
            J: self.J + other.J,
            c: self.c + other.c,
            h: self.h + other.h,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Add<T> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn add(self, c: T) -> Self::Output {
        Jch {
            J: self.J + c,
            c: self.c + c,
            h: self.h + c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> AddAssign<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent + AddAssign,
    Wp: WhitePoint,
{
    fn add_assign(&mut self, other: Jch<Wp, T>) {
        self.J += other.J;
        self.c += other.c;
        self.h += other.h;
    }
}

impl<Wp, T> AddAssign<T> for Jch<Wp, T>
where
    T: FloatComponent + AddAssign,
    Wp: WhitePoint,
{
    fn add_assign(&mut self, c: T) {
        self.J += c;
        self.c += c;
        self.h += c;
    }
}

impl<Wp, T> Sub<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn sub(self, other: Jch<Wp, T>) -> Self::Output {
        Jch {
            J: self.J - other.J,
            c: self.c - other.c,
            h: self.h - other.h,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Sub<T> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn sub(self, c: T) -> Self::Output {
        Jch {
            J: self.J - c,
            c: self.c - c,
            h: self.h - c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> SubAssign<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent + SubAssign,
    Wp: WhitePoint,
{
    fn sub_assign(&mut self, other: Jch<Wp, T>) {
        self.J -= other.J;
        self.c -= other.c;
        self.h -= other.h;
    }
}

impl<Wp, T> SubAssign<T> for Jch<Wp, T>
where
    T: FloatComponent + SubAssign,
    Wp: WhitePoint,
{
    fn sub_assign(&mut self, c: T) {
        self.J -= c;
        self.c -= c;
        self.h -= c;
    }
}

impl<Wp, T> Mul<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn mul(self, other: Jch<Wp, T>) -> Self::Output {
        Jch {
            J: self.J * other.J,
            c: self.c * other.c,
            h: self.h * other.h,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Mul<T> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn mul(self, c: T) -> Self::Output {
        Jch {
            J: self.J * c,
            c: self.c * c,
            h: self.h * c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> MulAssign<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent + MulAssign,
    Wp: WhitePoint,
{
    fn mul_assign(&mut self, other: Jch<Wp, T>) {
        self.J *= other.J;
        self.c *= other.c;
        self.h *= other.h;
    }
}

impl<Wp, T> MulAssign<T> for Jch<Wp, T>
where
    T: FloatComponent + MulAssign,
    Wp: WhitePoint,
{
    fn mul_assign(&mut self, c: T) {
        self.J *= c;
        self.c *= c;
        self.h *= c;
    }
}

impl<Wp, T> Div<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn div(self, other: Jch<Wp, T>) -> Self::Output {
        Jch {
            J: self.J / other.J,
            c: self.c / other.c,
            h: self.h / other.h,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Div<T> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Output = Jch<Wp, T>;

    fn div(self, c: T) -> Self::Output {
        Jch {
            J: self.J / c,
            c: self.c / c,
            h: self.h / c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> DivAssign<Jch<Wp, T>> for Jch<Wp, T>
where
    T: FloatComponent + DivAssign,
    Wp: WhitePoint,
{
    fn div_assign(&mut self, other: Jch<Wp, T>) {
        self.J /= other.J;
        self.c /= other.c;
        self.h /= other.h;
    }
}

impl<Wp, T> DivAssign<T> for Jch<Wp, T>
where
    T: FloatComponent + DivAssign,
    Wp: WhitePoint,
{
    fn div_assign(&mut self, c: T) {
        self.J /= c;
        self.c /= c;
        self.h /= c;
    }
}

impl<Wp, T, P> AsRef<P> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
    P: RawPixel<T> + ?Sized,
{
    fn as_ref(&self) -> &P {
        self.cs_raw()
    }
}

impl<Wp, T, P> AsMut<P> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
    P: RawPixel<T> + ?Sized,
{
    fn as_mut(&mut self) -> &mut P {
        self.cs_raw_mut()
    }
}

impl<Wp, T> RelativeContrast for Jch<Wp, T>
where
    Wp: WhitePoint,
    T: FloatComponent,
{
    type Scalar = T;

    fn get_contrast_ratio(&self, other: &Self) -> T {
        let luma1 = self.into_luma();
        let luma2 = other.into_luma();

        contrast_ratio(luma1.luma, luma2.luma)
    }
}

#[cfg(test)]
mod test {
    use super::Lab;
    use crate::white_point::D65;
    use crate::LinSrgb;

    #[test]
    fn red() {
        let a = Lab::from(LinSrgb::new(1.0, 0.0, 0.0));
        let b = Lab::new(53.23288, 80.09246, 67.2031);
        assert_relative_eq!(a, b, epsilon = 0.01);
    }

    #[test]
    fn green() {
        let a = Lab::from(LinSrgb::new(0.0, 1.0, 0.0));
        let b = Lab::new(87.73704, -86.184654, 83.18117);
        assert_relative_eq!(a, b, epsilon = 0.01);
    }

    #[test]
    fn blue() {
        let a = Lab::from(LinSrgb::new(0.0, 0.0, 1.0));
        let b = Lab::new(32.302586, 79.19668, -107.863686);
        assert_relative_eq!(a, b, epsilon = 0.01);
    }

    #[test]
    fn ranges() {
        assert_ranges! {
            Lab<D65, f64>;
            limited {
                J: 0.0 => 100.0,
                c: -128.0 => 127.0,
                h: -128.0 => 127.0
            }
            limited_min {}
            unlimited {}
        }
    }

    raw_pixel_conversion_tests!(Lab<D65>: l, a, b);
    raw_pixel_conversion_fail_tests!(Lab<D65>: l, a, b);

    #[cfg(feature = "serializing")]
    #[test]
    fn serialize() {
        let serialized = ::serde_json::to_string(&Lab::new(0.3, 0.8, 0.1)).unwrap();

        assert_eq!(serialized, r#"{"l":0.3,"a":0.8,"b":0.1}"#);
    }

    #[cfg(feature = "serializing")]
    #[test]
    fn deserialize() {
        let deserialized: Lab = ::serde_json::from_str(r#"{"l":0.3,"a":0.8,"b":0.1}"#).unwrap();

        assert_eq!(deserialized, Lab::new(0.3, 0.8, 0.1));
    }
}
