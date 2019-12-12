use std::sync::{Arc, Mutex};

use druid::{Widget, EventCtx, PaintCtx, BoxConstraints, BaseState, LayoutCtx, Event, Env, UpdateCtx};
use druid::piet::{Color, RenderContext};
use druid::kurbo::{Line, Point, Size};

use bounded_spsc_queue::{Producer, Consumer};

pub struct Oscilloscope {
    consumer: Consumer<f64>,
    buffer: VecDeque<f64>,
}

use super::State;
use std::collections::VecDeque;

impl Oscilloscope {
    pub fn new(consumer: Arc<Mutex<Option<Consumer<f64>>>>) -> Self {
        let mut c = consumer.lock().unwrap();
        let mut new_consumer = None;
        std::mem::swap(&mut *c, &mut new_consumer);
        Self {
            consumer: new_consumer.unwrap(),
            buffer: VecDeque::new(),
        }
    }
}

impl Widget<State> for Oscilloscope {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut State, _env: &Env) {
        // Don't handle events
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&State>, _data: &State, _env: &Env) {
        // Don't do anything special on update
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &State, _env: &Env) -> Size {
        // Take up the entire layout
        bc.constrain((800.0, 600.0))
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &State, env: &Env) {
        // Consume some samples
        for i in 0..100 {
            let x = self.consumer.pop();
            self.buffer.push_back(x);
        }

        // Limit buffer to 800 samples
        while self.buffer.len() > 800 {
            self.buffer.pop_front();
        }

        // Redraw
        paint_ctx.clear(Color::from_rgba32_u32(0x000000ff));

        // Draw all of the samples we have so far
        let red = Color::from_rgba32_u32(0xff0000ff);
        for x in 0..(self.buffer.len()-1) {
            let p0 = Point::new(x as f64, self.buffer[x] * 300.0 + 300.0);
            let p1 = Point::new((x+1) as f64, self.buffer[x+1] * 300.0 + 300.0);
            let line = Line::new(p0, p1);
            paint_ctx.stroke(line, &red, 1.0);
        }
    }
}