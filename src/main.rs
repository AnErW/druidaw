use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use bounded_spsc_queue::{Producer, Consumer};
use hound::WavReader;
use druid::{AppLauncher, WindowDesc, Data, Lens, Widget};

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
        let samples = reader.samples();
        for s in samples {
            p.push(s.unwrap());
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
