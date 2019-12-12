use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::{bounded, Receiver};
use druid::{
    piet::Color,
    theme,
    widget::{Button, EnvScope, Flex, Label, Slider, WidgetExt},
    AppLauncher, Data, Lens, Widget, WindowDesc,
};
use hound::WavReader;

mod oscilloscope;
use oscilloscope::Oscilloscope;

mod volume_meter;
use volume_meter::VolumeMeter;

#[derive(Clone, Data, Lens)]
struct State {
    volume: f64,
    left_level: f64,
    right_level: f64,
}

fn main() {
    let (p, c) = bounded(4800);
    let (p2, c2) = bounded(4800);

    thread::spawn(move || {
        let mut reader = WavReader::open("/Users/futurepaul/Music/ahh_48.wav").unwrap();
        let samples: hound::WavSamples<'_, std::io::BufReader<std::fs::File>, i16> =
            reader.samples();
        for s in samples.step_by(2) {
            let val = s.unwrap();
            let val_f32 = (val as f64) / (std::i16::MAX as f64);
            //println!("{}", val_f32);
            if let Err(e) = p.send(val_f32) {
                log::error!("{}", e)
            };
            if let Err(e) = p2.send(val_f32) {
                log::error!("{}", e)
            };
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
    let _active_client = client.activate_async((), process).unwrap();
    // processing starts here

    let state = State {
        volume: 0.5,
        left_level: 0.7,
        right_level: 0.99,
    };

    let consumer = Arc::new(Mutex::new(Some(c)));

    let window = WindowDesc::new(move || ui_builder(consumer.clone())).window_size((800.0, 600.0));
    AppLauncher::with_window(window)
        .configure_env(|env| {
            env.set(theme::WINDOW_BACKGROUND_COLOR, Color::BLACK);
        })
        .use_simple_logger()
        .launch(state)
        .expect("launch failed");
}

fn ui_builder(consumer: Arc<Mutex<Option<Receiver<f64>>>>) -> impl Widget<State> {
    let mut col = Flex::column();
    let mut row = Flex::row();

    let button = Button::new("Play", |_ctx, _data, _env| {})
        .fix_height(50.0)
        .fix_width(100.0);

    let big_text_label = EnvScope::new(
        |env| {
            env.set(theme::TEXT_SIZE_NORMAL, 24.0);
        },
        Label::new("0:00 / 3:14")
            .padding(20.0)
            .border(Color::grey(0.5), 2.0),
    );

    let mut volume_column = Flex::column();

    let volume_l = VolumeMeter::new().lens(State::left_level);
    let volume_r = VolumeMeter::new().lens(State::right_level);
    let volume_slider = Slider::new().lens(State::volume);

    volume_column.add_child(volume_l, 0.5);
    volume_column.add_child(volume_r, 0.5);
    volume_column.add_child(volume_slider, 1.0);

    row.add_child(button.center(), 1.0);
    row.add_child(big_text_label.center(), 1.0);
    row.add_child(volume_column.center().padding(10.0), 1.0);

    col.add_child(row.fix_height(100.0), 0.0);
    col.add_child(Oscilloscope::new(consumer), 1.0);

    col
}
