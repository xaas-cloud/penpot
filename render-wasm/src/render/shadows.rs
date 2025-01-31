use skia_safe::{self as skia, BlendMode, MaskFilter, Rect, TileMode};

use super::RenderState;
use crate::shapes::Shadow;

pub fn render_drop_shadow(render_state: &mut RenderState, shadow: &Shadow, scale: f32) {
    let shadow_paint = shadow.to_paint(true, scale);
    render_state.drawing_surface.draw(
        &mut render_state.shadow_surface.canvas(),
        (0.0, 0.0),
        skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
        Some(&shadow_paint),
    );

    render_state.shadow_surface.draw(
        &mut render_state.final_surface.canvas(),
        (0.0, 0.0),
        skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
        Some(&skia::Paint::default()),
    );

    render_state
        .shadow_surface
        .canvas()
        .clear(skia::Color::TRANSPARENT);
}

pub fn render_inner_shadow(render_state: &mut RenderState, shadow: &Shadow, scale: f32) {
    let sigma = shadow.blur;
    let mut filter = skia::image_filters::drop_shadow_only(
        (shadow.offset.0 * scale, shadow.offset.1 * scale), // DPR?
        (sigma * scale, sigma * scale),
        skia::Color::RED,
        None,
        None,
        None,
    );

    // filter = skia::image_filters::color_filter(
    //     skia::color_filters::blend(shadow.color, BlendMode::SrcIn).unwrap(),
    //     filter,
    //     None,
    // );

    let mut shadow_paint = skia::Paint::default();
    // shadow_paint.set_color(shadow.color);
    // shadow_paint.set_style(skia::PaintStyle::Fill);
    shadow_paint.set_image_filter(filter);

    render_state.drawing_surface.draw(
        &mut render_state.shadow_surface.canvas(),
        (0.0, 0.0),
        skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
        Some(&shadow_paint),
    );

    render_state.shadow_surface.draw(
        &mut render_state.overlay_surface.canvas(),
        (0.0, 0.0),
        skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
        None,
    );

    render_state
        .shadow_surface
        .canvas()
        .clear(skia::Color::TRANSPARENT);
    // render_state
    //     .overlay_surface
    //     .canvas()
    //     .clear(skia::Color::TRANSPARENT);
}

// pub fn render_inner_shadow(render_state: &mut RenderState, shadow: &Shadow, scale: f32) {
//     let shadow_paint = shadow.to_paint(true, scale);

//     // let mut shadow_paint = skia::Paint::default();
//     // shadow_paint.set_anti_alias(true);
//     // shadow_paint.set_color(shadow.color())

//     render_state.drawing_surface.draw(
//         &mut render_state.shadow_surface.canvas(),
//         (0.0, 0.0),
//         skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
//         None,
//     );

//     render_state
//         .shadow_surface
//         .canvas()
//         .draw_color(shadow.color, skia::BlendMode::SrcATop);

//     let mut blur_paint = skia::Paint::default();
//     // blur_paint.set_mask_filter(MaskFilter::blur(
//     //     skia_safe::BlurStyle::Inner,
//     //     shadow.blur,
//     //     true,
//     // ));
//     blur_paint.set_image_filter(skia::image_filters::blur(
//         (shadow.blur, shadow.blur),
//         TileMode::Clamp,
//         None,
//         None,
//     ));

//     let shadow_snapshot = render_state.shadow_surface.image_snapshot();
//     render_state
//         .shadow_surface
//         .canvas()
//         .clear(skia::Color::TRANSPARENT);

//     render_state
//         .shadow_surface
//         .canvas()
//         .draw_image(shadow_snapshot, (0., 0.), Some(&blur_paint));

//     // let original_snapshot = render_state.drawing_surface.image_snapshot();

//     // render_state.drawing_surface.draw(
//     //     &mut render_state.shadow_surface.canvas(),
//     //     (0.0, 0.0),
//     //     skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
//     //     // Some(&shadow_paint),
//     //     Some(&skia::Paint::default()),
//     // );

//     // let shadow_snapshot = render_state.shadow_surface.image_snapshot();
//     // render_state
//     //     .shadow_surface
//     //     .canvas()
//     //     .draw_image(&shadow_snapshot, (0, 0), Some(&shadow_paint));

//     // let mut mask_paint = skia::Paint::default();
//     // mask_paint.set_blend_mode(skia::BlendMode::DstOut);
//     // render_state
//     //     .shadow_surface
//     //     .canvas()
//     //     .draw_image(&original_snapshot, (0, 0), Some(&mask_paint));

//     // let rect = skia::Rect::from_xywh(0., 0., 100., 100.);
//     // let mut paint = skia::Paint::default();
//     // paint.set_color(skia::Color::RED);
//     // paint.set_style(skia::PaintStyle::Fill);

//     // render_state
//     //     .shadow_surface
//     //     .canvas()
//     //     .draw_rect(&rect, &paint);

//     render_state.shadow_surface.draw(
//         &mut render_state.final_surface.canvas(),
//         (0.0, 0.0),
//         skia::SamplingOptions::new(skia::FilterMode::Linear, skia::MipmapMode::Nearest),
//         Some(&skia::Paint::default()),
//     );

//     render_state
//         .shadow_surface
//         .canvas()
//         .clear(skia::Color::TRANSPARENT);
// }
