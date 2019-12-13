use std::sync::{Arc, Mutex};
use std::thread;

// TODO: VecDeque is **NOT** suitable for real-time audio.
use std::collections::VecDeque;

// TODO: crossbeam_channel::bounded (MPMC channel) is **NOT** suitable for real-time audio.
use crossbeam_channel::{bounded, Receiver};

use druid::{
    piet::Color,
    theme,
    widget::{Button, EnvScope, Flex, Label, Slider, WidgetExt},
    AppLauncher, Data, Lens, Selector, Widget, WindowDesc,
};
use hound::WavReader;

mod oscilloscope;
use oscilloscope::Oscilloscope;

mod volume_meter;
use volume_meter::VolumeMeter;

mod audio_player;
use audio_player::AudioPlayer;

#[derive(Clone, Data, Lens)]
struct State {
    #[druid(ignore)]
    audio_buffer: VecDeque<f64>,
    volume: f64,
    left_level: f64,
    right_level: f64,
    is_playing: bool,
}

fn main() {
    let (p, c) = bounded(4800);
    let (p2, c2) = bounded(4800);

    let path = std::env::args()
        .skip(1)
        .next()
        .expect("Expected a path to a 48khz WAV file");

    thread::spawn(move || {
        let mut reader = WavReader::open(path).unwrap();
        let samples: hound::WavSamples<'_, std::io::BufReader<std::fs::File>, i16> =
            reader.samples();
        // Samples are interleaved. Assuming two channels, we want to take every other sample.
        for s in samples.step_by(2) {
            let val = s.unwrap();
            let val_f32 = (val as f64) / (std::i16::MAX as f64);
            if let Err(e) = p.send(val_f32) {
                log::error!("p: {}", e)
            };
            if let Err(e) = p2.send(val_f32) {
                log::error!("p2: {}", e)
            };
        }
    });

    // 1. open a client
    let (client, _status) =
        jack::Client::new("druidaw", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let mut out_port = client
        .register_port("output", jack::AudioOut::default())
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
    if let Err(e) = active_client
        .as_client()
        .connect_ports_by_name("druidaw:output", "system:playback_1")
    {
        log::error!("system:playback_1: {}", e);
    }
    if let Err(e) = active_client
        .as_client()
        .connect_ports_by_name("druidaw:output", "system:playback_2")
    {
        log::error!("system:playback_2: {}", e);
    }

    // processing starts here

    let state = State {
        audio_buffer: VecDeque::new(),
        volume: 0.5,
        left_level: 0.7,
        right_level: 0.99,
        is_playing: false,
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

    let button = Button::new("Play / Stop", |ctx, data: &mut State, _env| {
        // Toggle is_playing
        // This is how AudioPlayer knows when to stop.
        data.is_playing = !data.is_playing;

        // Submit a command with a custom Selector.
        // This is how AudioPlayer knows when to start.
        if data.is_playing {
            ctx.submit_command(Selector::new("PLAY"), None);
        }
    })
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

    col.add_child(AudioPlayer::new(consumer), 0.0);
    col.add_child(row.fix_height(100.0), 0.0);
    col.add_child(Oscilloscope::new(), 1.0);

    col
}
