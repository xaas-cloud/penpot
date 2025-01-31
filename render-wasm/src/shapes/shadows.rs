use skia_safe::{self as skia, image_filters, BlurStyle, ImageFilter, MaskFilter, PaintStyle};

use super::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowStyle {
    Drop,
    Inner,
}

impl From<u8> for ShadowStyle {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Drop,
            1 => Self::Inner,
            _ => Self::default(),
        }
    }
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self::Drop
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub color: Color,
    pub blur: f32,
    spread: f32,
    pub offset: (f32, f32),
    style: ShadowStyle,
    hidden: bool,
}

// TODO: create shadows out of a chunk of bytes
impl Shadow {
    pub fn new(
        color: Color,
        blur: f32,
        spread: f32,
        offset: (f32, f32),
        style: ShadowStyle,
        hidden: bool,
    ) -> Self {
        Self {
            color,
            blur,
            spread,
            offset,
            style,
            hidden,
        }
    }

    pub fn style(&self) -> ShadowStyle {
        self.style
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn to_paint(&self, dilate: bool, scale: f32) -> skia::Paint {
        let mut paint = skia::Paint::default();

        let (image_filter, mask_filter) = self.filters(dilate, scale);

        if mask_filter.is_some() {
            paint.set_mask_filter(mask_filter);
            paint.set_color(self.color);
        }

        paint.set_image_filter(image_filter);
        paint.set_anti_alias(true);

        paint
    }

    fn filters(&self, dilate: bool, scale: f32) -> (Option<ImageFilter>, Option<MaskFilter>) {
        let mut filter = match self.style {
            ShadowStyle::Drop => image_filters::drop_shadow_only(
                (self.offset.0 * scale, self.offset.1 * scale),
                (self.blur * scale, self.blur * scale),
                self.color,
                None,
                None,
                None,
            ),
            ShadowStyle::Inner => None,
        };

        let mask_filter = match self.style {
            ShadowStyle::Drop => None,
            ShadowStyle::Inner => MaskFilter::blur(BlurStyle::Normal, self.blur * scale, true),
        };

        if dilate {
            filter =
                image_filters::dilate((self.spread * scale, self.spread * scale), filter, None);
        }

        (filter, mask_filter)
    }
}
