use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use bounded_spsc_queue::{Producer, Consumer};
use hound::WavReader;
use druid::{AppLauncher, WindowDesc, Data, Lens, Widget};
use itertools::Itertools;

mod oscilloscope;
use oscilloscope::Oscilloscope;

#[derive(Clone, Data, Lens)]
struct State {
    some_value: usize,
}

fn main() {
    let (p, c) = bounded_spsc_queue::make(4800);
    thread::spawn(move|| {
        let mut reader = WavReader::open("/home/crs/Downloads/lifeformed.wav").unwrap();
        let samples: hound::WavSamples<'_, std::io::BufReader<std::fs::File>, i16> = reader.samples();
        // Samples are interleaved, so take every other one.
        for s in samples.step_by(2) {
            let val = s.unwrap();
            let val_f32 = (val as f64) / (std::i16::MAX as f64);
            //println!("{}", val_f32);
            p.push(val_f32);
        }
    });


    let state = State {
        some_value: 42,
    };

    let consumer = Arc::new(Mutex::new(Some(c)));

    let window = WindowDesc::new(move || { Oscilloscope::new(consumer.clone()) });
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(state)
        .expect("launch failed");
}
