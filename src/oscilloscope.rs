use druid::kurbo::{Line, Point, Size};
use druid::piet::{Color, RenderContext};
use druid::{
    BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
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
        ctx.invalidate();
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: Option<&State>,
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
        paint_ctx: &mut PaintCtx,
        base_state: &BaseState,
        data: &State,
        _env: &Env,
    ) {
        // Draw all of the samples we have so far
        let width = base_state.size().width;
        let height = base_state.size().height;

        let white = Color::from_rgba32_u32(0xffffffff);
        if data.audio_buffer.len() > 0 {
            for x in 0..(data.audio_buffer.len() - 1) {
                let p0 = Point::new(
                    x as f64 * width / (data.audio_buffer.len() as f64),
                    (data.audio_buffer[x] * height / 2.0) + (height / 2.0),
                );
                let p1 = Point::new(
                    (x + 1) as f64 * width / (data.audio_buffer.len() as f64),
                    (data.audio_buffer[x + 1] * height / 2.0) + (height / 2.0),
                );
                let line = Line::new(p0, p1);
                paint_ctx.stroke(line, &white, 1.0);
            }
        }
    }
}
