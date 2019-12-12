use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crossbeam_channel::bounded;
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
    let (p, c) = bounded(4800);
    let (p2, c2) = bounded(4800);

    thread::spawn(move || {
        let mut reader = WavReader::open("/home/crs/Downloads/nanou2.wav").unwrap();
        let samples: hound::WavSamples<'_, std::io::BufReader<std::fs::File>, i16> = reader.samples();
        for s in samples.step_by(2) {
            let val = s.unwrap();
            let val_f32 = (val as f64) / (std::i16::MAX as f64);
            //println!("{}", val_f32);
            p.send(val_f32);
            p2.send(val_f32);
        }
    });

    // 1. open a client
    let (client, _status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let mut out_port = client
        .register_port("sine_out", jack::AudioOut::default())
        .unwrap();

    // 3. define process callback handler
    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let out = out_port.as_mut_slice(ps);

            // Write output
            for v in out.iter_mut() {
                *v = c2.recv().unwrap() as f32;
            }

            // Continue as normal
            jack::Control::Continue
        },
    );

    // 4. activate the client
    let active_client = client.activate_async((), process).unwrap();
    // processing starts here

    let state = State {
        some_value: 42,
    };

    let consumer = Arc::new(Mutex::new(Some(c)));

    let window = WindowDesc::new(move || { Oscilloscope::new(consumer.clone()) })
        .window_size((800.0, 600.0));
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(state)
        .expect("launch failed");
}
