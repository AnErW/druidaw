use std::sync::{Arc, Mutex};

use druid::kurbo::{Line, Point, Size};
use druid::piet::{Color, RenderContext};
use druid::{
    BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
};

use crossbeam_channel::Receiver;

pub struct Oscilloscope {
    consumer: Receiver<f64>,
    buffer: VecDeque<f64>,
    t: f64,
    sum_t: f64,
}

use super::State;
use std::collections::VecDeque;

impl Oscilloscope {
    pub fn new(consumer: Arc<Mutex<Option<Receiver<f64>>>>) -> Self {
        let mut c = consumer.lock().unwrap();
        let mut new_consumer = None;
        std::mem::swap(&mut *c, &mut new_consumer);
        Self {
            consumer: new_consumer.unwrap(),
            buffer: VecDeque::new(),
            t: 0.0,
            sum_t: 0.0,
        }
    }
}

impl Widget<State> for Oscilloscope {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut State, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                self.t = 0.0;
                self.sum_t = 0.0;
                ctx.request_anim_frame();
            }
            Event::AnimFrame(interval) => {
                let delta_t = (*interval as f64) * 1e-9;
                self.t = delta_t;
                self.sum_t += delta_t;
                ctx.request_anim_frame();
                // When we do fine-grained invalidation,
                // no doubt this will be required:
                //ctx.invalidate();
            }
            _ => (),
        }
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
        _data: &State,
        _env: &Env,
    ) {
        let buffer_size = 8192;
        // Consume some samples
        for _i in 0..(48000.0 * self.t) as usize {
            let x = self.consumer.recv().unwrap();
            self.buffer.push_back(x);
            if self.buffer.len() > buffer_size {
                self.buffer.pop_front();
            }
        }

        // Draw all of the samples we have so far
        let width = base_state.size().width;
        let height = base_state.size().height;

        let red = Color::from_rgba32_u32(0xff0000ff);
        if self.buffer.len() > 0 {
            for x in 0..(self.buffer.len() - 1) {
                let p0 = Point::new(
                    x as f64 * width / (buffer_size as f64),
                    (self.buffer[x] * height / 2.0) + (height / 2.0),
                );
                let p1 = Point::new(
                    (x + 1) as f64 * width / (buffer_size as f64),
                    (self.buffer[x + 1] * height / 2.0) + (height / 2.0),
                );
                let line = Line::new(p0, p1);
                paint_ctx.stroke(line, &red, 1.0);
            }
        }
    }
}
