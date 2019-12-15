//! Special widget for handling audio samples coming from the DSP thread.
//! Doesn't draw anything.

use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;
use druid::{
    kurbo::Size, BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, Selector,
    UpdateCtx, Widget,
};
use log::info;

use super::State;

// TODO: This seemingly has to be tweaked per-song...?
const AUDIO_DB_RANGE: f64 = 70.0;

pub struct AudioPlayer {
    consumer: Receiver<f64>,
    audio_buffer_decimation: usize,
    volume_filter_delay_one: f64,
    volume_filter_coefficient: f64,
}

impl AudioPlayer {
    pub fn new(consumer: Arc<Mutex<Option<Receiver<f64>>>>) -> impl Widget<State> {
        let mut c = consumer.lock().unwrap();
        let mut new_consumer = None;
        std::mem::swap(&mut *c, &mut new_consumer);
        Self {
            consumer: new_consumer.unwrap(),
            audio_buffer_decimation: 0,
            volume_filter_delay_one: 0.0,
            volume_filter_coefficient: 0.00005,
        }
    }
}

impl Widget<State> for AudioPlayer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, _env: &Env) {
        // Fetch samples from the channel, and push them into the application state as needed.
        log::info!("AudioPlayer EVENT");

        match event {
            Event::Command(command) => {
                let play_selector = Selector::new("PLAY");
                if command.selector == play_selector {
                    log::info!("Play command received.");
                    ctx.request_anim_frame();
                }
            }
            Event::AnimFrame(_interval) => {
                if data.is_playing {
                    // Consume samples
                    loop {
                        let sample = self.consumer.try_recv();
                        if sample.is_err() {
                            break;
                        }
                        let current_sample = sample.unwrap();

                        // Set the level for the volume meters (dB scale)
                        // Firstly, get the current audio amplitude, squared:
                        let amplitude = current_sample.powi(2);
                        // Send that through an IIR filter:
                        let filtered = self.volume_filter_delay_one * (1.0 - self.volume_filter_coefficient) +
                            amplitude * self.volume_filter_coefficient;
                        self.volume_filter_delay_one = filtered;
                        // Find the filtered value in decibels (0.0 is loudest)
                        let mut db = 20.0 * filtered.log10();
                        // Add to it so that (most) values are in the range [0.0, AUDIO_DB_RANGE]:
                        db += AUDIO_DB_RANGE;
                        // Truncate it so that it's *definitely* in that range:
                        db = db.min(AUDIO_DB_RANGE).max(0.0);
                        // Now divide by AUDIO_DB_RANGE so that it's in range [0.0, 1.0] for our volume meters:
                        let volume = db / AUDIO_DB_RANGE;
                        data.left_level = volume;
                        data.right_level = volume;

                        // Fill the audio buffer for the oscilloscope
                        if self.audio_buffer_decimation == 128 {
                            data.audio_buffer.push_back(current_sample);
                            self.audio_buffer_decimation = 0;
                        }
                        self.audio_buffer_decimation += 1;
                    }
                    ctx.request_anim_frame();
                }
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
        _paint_ctx: &mut PaintCtx,
        _base_state: &BaseState,
        _data: &State,
        _env: &Env,
    ) {
        // Don't paint anything
    }
}
