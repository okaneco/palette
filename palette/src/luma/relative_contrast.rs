pub mod wcag {
    use Component;

    /// A trait for calculating relative contrast between two colors
    ///
    /// W3C's Web Content Accessibility Guidelines (WCAG) 2.1 suggest a method
    /// to calculate accessible contrast ratios of text and background colors
    /// for those with low vision or color deficiencies. The possible range of
    /// ratios is from 1:1 to 21:1. There is a Success Criterion for Contrast
    /// (Minimum) and a Success Criterion for Contrast (Enhanced), SC 1.4.3 and
    /// SC 1.4.6 respectively. The relative contrast is calculated by `(L1 +
    /// 0.05) / (L2 + 0.05)`, where `L1` is the luminance of the brighter color
    /// and `L2` is the luminance of the darker color both in sRGB linear space.
    /// Higher contrast ratio is generally desireable.
    pub trait RelativeContrast {
        /// Type of return value for contrast ratio
        type Scalar: Component;

        /// Calculate contrast ratio between two colors
        fn get_contrast_ratio(&self, other: &Self) -> Self::Scalar;
        /// Verify the contrast between two colors satisfies SC 1.4.3. Contrast
        /// is at least 4.5:1 (Level AA).
        fn is_min_contrast(&self, other: &Self) -> bool;
        /// Verify the contrast between two colors satisfies SC 1.4.3 for large
        /// text. Contrast is at least 3:1 (Level AA).
        fn is_min_contrast_large(&self, other: &Self) -> bool;
        /// Verify the contrast between two colors satisfies SC 1.4.6. Contrast
        /// is at least 7:1 (Level AAA).
        fn is_enhanced_contrast(&self, other: &Self) -> bool;
        /// Verify the contrast between two colors satisfies SC 1.4.6 for large
        /// text. Contrast is at least 4.5:1 (Level AAA).
        fn is_enhanced_contrast_large(&self, other: &Self) -> bool;
    }
}
