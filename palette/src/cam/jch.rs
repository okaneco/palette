use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(feature = "random")]
use rand::distributions::uniform::{SampleBorrow, SampleUniform, Uniform, UniformSampler};
#[cfg(feature = "random")]
use rand::distributions::{Distribution, Standard};
#[cfg(feature = "random")]
use rand::Rng;

use crate::color_difference::ColorDifference;
use crate::color_difference::{get_ciede_difference, LabColorDiff};
use crate::convert::{FromColorUnclamped, IntoColorUnclamped};
use crate::encoding::pixel::RawPixel;
use crate::white_point::{WhitePoint, D65};
use crate::{
    clamp, contrast_ratio, from_f64, Alpha, CamHue, Component, FloatComponent, FromColor, GetHue,
    Hue, Limited, Mix, Pixel, RelativeContrast, Saturate, Shade, Xyz,
};

// TODO: Documentation, skip derives?, todo

/// CIE JCh with an alpha component. See the [`Jcha` implementation in
/// `Alpha`](struct.Alpha.html#Jcha).
pub type Jcha<Wp, T = f32> = Alpha<Jch<Wp, T>, T>;

/// CIECAM02 JCh color space.
///
/// JCh is a cylindrical representation of the CIE Color Appearance Model
/// proposed in 2002, a revision to CIECAM97. Color appearance models attempt to
/// better represent characteristics of human vision perception. CAMs attempt
/// to take into account appearance effects of colors due to brightness, chroma,
/// colorfulness, hue, lightness, and saturation.
#[derive(Debug, PartialEq, Pixel, FromColorUnclamped, WithAlpha)]
#[cfg_attr(feature = "serializing", derive(Serialize, Deserialize))]
#[palette(
    palette_internal,
    white_point = "Wp",
    component = "T",
    skip_derives(Xyz)
)]
#[repr(C)]
pub struct Jch<Wp = D65, T = f32>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    /// The correlate of lightness.
    pub j: T,

    /// The correlate of chroma.
    pub chroma: T,

    /// The correlate of hue.
    #[palette(unsafe_same_layout_as = "T")]
    pub hue: CamHue<T>,

    /// The white point associated with the color's illuminant and observer.
    /// D65 for 2 degree observer is used by default.
    #[cfg_attr(feature = "serializing", serde(skip))]
    #[palette(unsafe_zero_sized)]
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
    /// CIE JCh with white point D65.
    pub fn new<H: Into<CamHue<T>>>(j: T, chroma: T, hue: H) -> Jch<D65, T> {
        Jch {
            j,
            chroma,
            hue: hue.into(),
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    /// CIE JCh.
    pub fn with_wp<H: Into<CamHue<T>>>(j: T, chroma: T, hue: H) -> Jch<Wp, T> {
        Jch {
            j,
            chroma,
            hue: hue.into(),
            white_point: PhantomData,
        }
    }

    /// Convert to a `(J, C, h)` tuple.
    pub fn into_components(self) -> (T, T, CamHue<T>) {
        (self.j, self.chroma, self.hue)
    }

    /// Convert from a `(J, C, h)` tuple.
    pub fn from_components<H: Into<CamHue<T>>>((j, chroma, hue): (T, T, H)) -> Self {
        Self::with_wp(j, chroma, hue)
    }

    /// Return the `l` value minimum.
    pub fn min_j() -> T {
        T::zero()
    }

    /// Return the `l` value maximum.
    pub fn max_j() -> T {
        from_f64(100.0)
    }

    /// Return the `chroma` value minimum.
    pub fn min_chroma() -> T {
        T::zero()
    }

    /// Return the `chroma` value maximum. This value does not cover the entire
    /// color space, but covers enough to be practical for downsampling to
    /// smaller color spaces like sRGB.
    pub fn max_chroma() -> T {
        from_f64(128.0)
    }

    /// Return the `chroma` extended maximum value. This value covers the entire
    /// color space and is included for completeness, but the additional range
    /// should be unnecessary for most use cases.
    pub fn max_extended_chroma() -> T {
        from_f64(crate::float::Float::sqrt(128.0f64 * 128.0 + 128.0 * 128.0))
    }
}

///<span id="Jcha"></span>[`Jcha`](type.Jcha.html) implementations.
impl<T, A> Alpha<Jch<D65, T>, A>
where
    T: FloatComponent,
    A: Component,
{
    /// CIE JCh and transparency with white point D65.
    pub fn new<H: Into<CamHue<T>>>(j: T, chroma: T, hue: H, alpha: A) -> Self {
        Alpha {
            color: Jch::new(j, chroma, hue),
            alpha,
        }
    }
}

///<span id="Jcha"></span>[`Jcha`](type.Jcha.html) implementations.
impl<Wp, T, A> Alpha<Jch<Wp, T>, A>
where
    T: FloatComponent,
    A: Component,
    Wp: WhitePoint,
{
    /// CIE JCh and transparency.
    pub fn with_wp<H: Into<CamHue<T>>>(j: T, chroma: T, hue: H, alpha: A) -> Self {
        Alpha {
            color: Jch::with_wp(j, chroma, hue),
            alpha,
        }
    }

    /// Convert to a `(J, C, h, alpha)` tuple.
    pub fn into_components(self) -> (T, T, CamHue<T>, A) {
        (self.j, self.chroma, self.hue, self.alpha)
    }

    /// Convert from a `(J, C, h, alpha)` tuple.
    pub fn from_components<H: Into<CamHue<T>>>((j, chroma, hue, alpha): (T, T, H, A)) -> Self {
        Self::with_wp(j, chroma, hue, alpha)
    }
}

impl<Wp, T> FromColorUnclamped<Jch<Wp, T>> for Jch<Wp, T>
where
    Wp: WhitePoint,
    T: FloatComponent,
{
    fn from_color_unclamped(color: Jch<Wp, T>) -> Self {
        color
    }
}

impl<Wp, T> FromColorUnclamped<Xyz<Wp, T>> for Jch<Wp, T>
where
    Wp: WhitePoint,
    T: FloatComponent,
{
    fn from_color_unclamped(color: Xyz<Wp, T>) -> Self {
        todo!();
    }
}

impl<Wp: WhitePoint, T: FloatComponent, H: Into<CamHue<T>>> From<(T, T, H)> for Jch<Wp, T> {
    fn from(components: (T, T, H)) -> Self {
        Self::from_components(components)
    }
}

impl<Wp: WhitePoint, T: FloatComponent> Into<(T, T, CamHue<T>)> for Jch<Wp, T> {
    fn into(self) -> (T, T, CamHue<T>) {
        self.into_components()
    }
}

impl<Wp: WhitePoint, T: FloatComponent, H: Into<CamHue<T>>, A: Component> From<(T, T, H, A)>
    for Alpha<Jch<Wp, T>, A>
{
    fn from(components: (T, T, H, A)) -> Self {
        Self::from_components(components)
    }
}

impl<Wp: WhitePoint, T: FloatComponent, A: Component> Into<(T, T, CamHue<T>, A)>
    for Alpha<Jch<Wp, T>, A>
{
    fn into(self) -> (T, T, CamHue<T>, A) {
        self.into_components()
    }
}

impl<Wp, T> Limited for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    fn is_valid(&self) -> bool {
        self.j >= T::zero() && self.j <= from_f64(100.0) && self.chroma >= T::zero()
    }

    fn clamp(&self) -> Jch<Wp, T> {
        let mut c = *self;
        c.clamp_self();
        c
    }

    fn clamp_self(&mut self) {
        self.j = clamp(self.j, T::zero(), from_f64(100.0));
        self.chroma = self.chroma.max(T::zero())
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
        let hue_diff: T = (other.hue - self.hue).to_degrees();
        Jch {
            j: self.j + factor * (other.j - self.j),
            chroma: self.chroma + factor * (other.chroma - self.chroma),
            hue: self.hue + factor * hue_diff,
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
            j: self.j + amount * from_f64(100.0),
            chroma: self.chroma,
            hue: self.hue,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> GetHue for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Hue = CamHue<T>;

    fn get_hue(&self) -> Option<CamHue<T>> {
        if self.chroma <= T::zero() {
            None
        } else {
            Some(self.hue)
        }
    }
}

impl<Wp, T> Hue for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    fn with_hue<H: Into<Self::Hue>>(&self, hue: H) -> Jch<Wp, T> {
        Jch {
            j: self.j,
            chroma: self.chroma,
            hue: hue.into(),
            white_point: PhantomData,
        }
    }

    fn shift_hue<H: Into<Self::Hue>>(&self, amount: H) -> Jch<Wp, T> {
        Jch {
            j: self.j,
            chroma: self.chroma,
            hue: self.hue + amount.into(),
            white_point: PhantomData,
        }
    }
}

/// CIEDE2000 distance metric for color difference.
impl<Wp, T> ColorDifference for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn get_color_difference(&self, other: &Jch<Wp, T>) -> Self::Scalar {
        todo!();
    }
}

impl<Wp, T> Saturate for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn saturate(&self, factor: T) -> Jch<Wp, T> {
        Jch {
            j: self.j,
            chroma: self.chroma * (T::one() + factor),
            hue: self.hue,
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
        Jch::with_wp(T::zero(), T::zero(), CamHue::from(T::zero()))
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
            j: self.j + other.j,
            chroma: self.chroma + other.chroma,
            hue: self.hue + other.hue,
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
            j: self.j + c,
            chroma: self.chroma + c,
            hue: self.hue + c,
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
        self.j += other.j;
        self.chroma += other.chroma;
        self.hue += other.hue;
    }
}

impl<Wp, T> AddAssign<T> for Jch<Wp, T>
where
    T: FloatComponent + AddAssign,
    Wp: WhitePoint,
{
    fn add_assign(&mut self, c: T) {
        self.j += c;
        self.chroma += c;
        self.hue += c;
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
            j: self.j - other.j,
            chroma: self.chroma - other.chroma,
            hue: self.hue - other.hue,
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
            j: self.j - c,
            chroma: self.chroma - c,
            hue: self.hue - c,
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
        self.j -= other.j;
        self.chroma -= other.chroma;
        self.hue -= other.hue;
    }
}

impl<Wp, T> SubAssign<T> for Jch<Wp, T>
where
    T: FloatComponent + SubAssign,
    Wp: WhitePoint,
{
    fn sub_assign(&mut self, c: T) {
        self.j -= c;
        self.chroma -= c;
        self.hue -= c;
    }
}

impl<Wp, T, P> AsRef<P> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
    P: RawPixel<T> + ?Sized,
{
    fn as_ref(&self) -> &P {
        self.as_raw()
    }
}

impl<Wp, T, P> AsMut<P> for Jch<Wp, T>
where
    T: FloatComponent,
    Wp: WhitePoint,
    P: RawPixel<T> + ?Sized,
{
    fn as_mut(&mut self) -> &mut P {
        self.as_raw_mut()
    }
}

impl<Wp, T> RelativeContrast for Jch<Wp, T>
where
    Wp: WhitePoint,
    T: FloatComponent,
{
    type Scalar = T;

    fn get_contrast_ratio(&self, other: &Self) -> T {
        todo!();
        // let xyz1 = Xyz::from_color(*self);
        // let xyz2 = Xyz::from_color(*other);

        // contrast_ratio(xyz1.y, xyz2.y)
    }
}

#[cfg(feature = "random")]
impl<Wp, T> Distribution<Jch<Wp, T>> for Standard
where
    T: FloatComponent,
    Wp: WhitePoint,
    Standard: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Jch<Wp, T> {
        Jch {
            j: rng.gen(),
            chroma: crate::Float::sqrt(rng.gen()) * from_f64(128.0),
            hue: rng.gen::<CamHue<T>>(),
            white_point: PhantomData,
        }
    }
}

#[cfg(feature = "random")]
pub struct UniformJch<Wp, T>
where
    T: FloatComponent + SampleUniform,
    Wp: WhitePoint + SampleUniform,
{
    j: Uniform<T>,
    chroma: Uniform<T>,
    hue: crate::hues::UniformCamHue<T>,
    white_point: PhantomData<Wp>,
}

#[cfg(feature = "random")]
impl<Wp, T> SampleUniform for Jch<Wp, T>
where
    T: FloatComponent + SampleUniform,
    Wp: WhitePoint + SampleUniform,
{
    type Sampler = UniformJch<Wp, T>;
}

#[cfg(feature = "random")]
impl<Wp, T> UniformSampler for UniformJch<Wp, T>
where
    T: FloatComponent + SampleUniform,
    Wp: WhitePoint + SampleUniform,
{
    type X = Jch<Wp, T>;

    fn new<B1, B2>(low_b: B1, high_b: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = *low_b.borrow();
        let high = *high_b.borrow();

        UniformJch {
            j: Uniform::new::<_, T>(low.j, high.j),
            chroma: Uniform::new::<_, T>(low.chroma * low.chroma, high.chroma * high.chroma),
            hue: crate::hues::UniformCamHue::new(low.hue, high.hue),
            white_point: PhantomData,
        }
    }

    fn new_inclusive<B1, B2>(low_b: B1, high_b: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = *low_b.borrow();
        let high = *high_b.borrow();

        UniformJch {
            j: Uniform::new_inclusive::<_, T>(low.j, high.j),
            chroma: Uniform::new_inclusive::<_, T>(
                low.chroma * low.chroma,
                high.chroma * high.chroma,
            ),
            hue: crate::hues::UniformCamHue::new_inclusive(low.hue, high.hue),
            white_point: PhantomData,
        }
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Jch<Wp, T> {
        Jch {
            j: self.j.sample(rng),
            chroma: crate::Float::sqrt(self.chroma.sample(rng)),
            hue: self.hue.sample(rng),
            white_point: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::white_point::D65;
    use crate::Jch;

    //     #[test]
    //     fn ranges() {
    //         assert_ranges! {
    //             Jch<D65, f64>;
    //             limited {
    //                 j: 0.0 => 100.0
    //             }
    //             limited_min {
    //                 chroma: 0.0 => 200.0
    //             }
    //             unlimited {
    //                 hue: -360.0 => 360.0
    //             }
    //         }
    //     }

    //     raw_pixel_conversion_tests!(Jch<D65>: l, chroma, hue);
    //     raw_pixel_conversion_fail_tests!(Jch<D65>: l, chroma, hue);

    #[test]
    fn check_min_max_components() {
        assert_relative_eq!(Jch::<D65, f32>::min_j(), 0.0);
        assert_relative_eq!(Jch::<D65, f32>::max_j(), 100.0);
        assert_relative_eq!(Jch::<D65, f32>::min_chroma(), 0.0);
        assert_relative_eq!(Jch::<D65, f32>::max_chroma(), 128.0);
        assert_relative_eq!(Jch::<D65, f32>::max_extended_chroma(), 181.01933598375618);
    }

    #[cfg(feature = "serializing")]
    #[test]
    fn serialize() {
        let serialized = ::serde_json::to_string(&Jch::new(0.3, 0.8, 0.1)).unwrap();

        assert_eq!(serialized, r#"{"l":0.3,"chroma":0.8,"hue":0.1}"#);
    }

    #[cfg(feature = "serializing")]
    #[test]
    fn deserialize() {
        let deserialized: Jch =
            ::serde_json::from_str(r#"{"l":0.3,"chroma":0.8,"hue":0.1}"#).unwrap();

        assert_eq!(deserialized, Jch::new(0.3, 0.8, 0.1));
    }
}
