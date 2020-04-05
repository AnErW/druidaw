use druid::kurbo::{Line, Point, Size};
use druid::piet::{Color, RenderContext};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, UpdateCtx, Widget,
};

pub struct Oscilloscope {}

use super::State;

impl Oscilloscope {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget<State> for Oscilloscope {
    fn event(&mut self, ctx: &mut EventCtx, _event: &Event, _data: &mut State, _env: &Env) {
        ctx.request_paint();
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &State,
        _env: &Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &State,
        _data: &State,
        _env: &Env,
    ) {
        // Don't do anything special on update
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &State,
        _env: &Env,
    ) -> Size {
        // Take up the entire layout
        bc.constrain((800.0, 500.0))
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx,
        data: &State,
        _env: &Env,
    ) {
        // Draw all of the samples we have so far
        let Size { width, height } = ctx.size();

        let white = Color::from_rgba32_u32(0xffffffff);

        let mut prev = 0.0;
        for (index, d) in data.audio_buffer.iter().enumerate() {
            if index == 0 {
                prev = *d;
                continue;
            }

            let p0 = Point::new(
                index as f64 * width / (data.audio_buffer.len() as f64),
                prev * height / 2.0 + (height / 2.0),
            );

            let p1 = Point::new(
                index as f64 * width / (data.audio_buffer.len() as f64),
                *d * height / 2.0 + (height / 2.0),
            );

            let line = Line::new(p0, p1);
            ctx.stroke(line, &white, 1.0);


            prev = *d;
        }
    }
}
