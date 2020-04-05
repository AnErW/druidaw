use druid::{
    kurbo::{Point, RoundedRect, Size},
    piet::{Color, LinearGradient, RenderContext, UnitPoint},
    theme,
    widget::Align,
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, UpdateCtx, Widget,
};

#[derive(Default)]
pub struct VolumeMeter {}

impl VolumeMeter {
    pub fn new() -> impl Widget<f64> {
        Align::vertical(UnitPoint::CENTER, Self::default())
    }
}

impl Widget<f64> for VolumeMeter {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut f64, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &f64,
        _env: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &f64, _data: &f64, _env: &Env) {
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        env: &Env,
    ) -> Size {
        bc.debug_check("Volume Meter");

        let default_width = 100.0;

        if bc.is_width_bounded() {
            bc.constrain(Size::new(
                bc.max().width,
                env.get(theme::BASIC_WIDGET_HEIGHT) / 2.0,
            ))
        } else {
            bc.constrain(Size::new(
                default_width,
                env.get(theme::BASIC_WIDGET_HEIGHT) / 2.0,
            ))
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, env: &Env) {
        let clamped = data.max(0.0).min(1.0);

        let rounded_rect = RoundedRect::from_origin_size(
            Point::ORIGIN,
            (Size {
                width: ctx.size().width,
                height: env.get(theme::BASIC_WIDGET_HEIGHT) / 2.0,
            })
            .to_vec2(),
            4.,
        );

        //Paint the border
        ctx.stroke(rounded_rect, &env.get(theme::BORDER_DARK), 2.0);

        //Paint the level
        let calculated_level_width = clamped * rounded_rect.width();
        let rounded_rect = RoundedRect::from_origin_size(
            Point::ORIGIN,
            (Size {
                width: calculated_level_width,
                height: env.get(theme::BASIC_WIDGET_HEIGHT) / 2.0,
            })
            .to_vec2(),
            4.,
        );
        let meter_gradient = LinearGradient::new(
            UnitPoint::LEFT,
            UnitPoint::RIGHT,
            (
                Color::WHITE,
                Color::WHITE,
                Color::WHITE,
                Color::rgb8(0xff, 0xff, 0x00),
                Color::rgb8(0xff, 0x00, 0x00),
            ),
        );
        ctx.fill(rounded_rect, &meter_gradient);
    }
}
