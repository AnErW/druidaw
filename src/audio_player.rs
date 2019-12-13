//! Special widget for handling audio samples coming from the DSP thread.
//! Doesn't draw anything.

use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;
use druid::{
    kurbo::Size,
    BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
};

use super::State;

const AUDIO_BUFFER_SIZE: usize = 8192;

pub struct AudioPlayer {
    consumer: Receiver<f64>,
}

impl AudioPlayer {
    pub fn new(consumer: Arc<Mutex<Option<Receiver<f64>>>>) -> impl Widget<State> {
        let mut c = consumer.lock().unwrap();
        let mut new_consumer = None;
        std::mem::swap(&mut *c, &mut new_consumer);
        Self {
            consumer: new_consumer.unwrap(),
        }
    }
}

impl Widget<State> for AudioPlayer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, _env: &Env) {
        // Fetch samples from the channel, and push them into the application state as needed.
        log::info!("AudioPlayer EVENT");
        match event {
            Event::AnimFrame(interval) => {
                let delta_t = (*interval as f64) * 1e-9;

                // Consume samples
                loop {
                    let sample = self.consumer.try_recv();
                    if sample.is_err() {
                        break;
                    }
                    data.audio_buffer.push_back(sample.unwrap());
                    if data.audio_buffer.len() > AUDIO_BUFFER_SIZE {
                        data.audio_buffer.pop_front();
                    }
                }

                ctx.request_anim_frame();
            }
            _ => (),
        }


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
        // Take up minimal layout space
        bc.min()
    }

    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx,
        base_state: &BaseState,
        _data: &State,
        _env: &Env,
    ) {
        // Don't paint anything
    }
}