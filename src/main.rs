extern crate sdl2;
extern crate ears;

mod chart;
mod guitarplaythrough;

use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;

use sdl2::gfx::primitives::DrawRenderer;

use ears::{AudioController};

use guitarplaythrough::*;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

enum GameButton {
    Green,
    Red,
    Yellow,
    Blue,
    Orange,
}

enum GameInputAction {
    Quit,
    ButtonDown(GameButton),
    ButtonUp(GameButton),
    Strum,
}

impl GameButton {
    fn to_guitar(self: &Self) -> Fret {
        match self {
            GameButton::Green => Fret::G,
            GameButton::Red => Fret::R,
            GameButton::Yellow => Fret::Y,
            GameButton::Blue => Fret::B,
            GameButton::Orange => Fret::O,
        }
    }
}

impl GameInputAction {
    fn to_guitar_action(self: &Self) -> Option<GuitarInputAction> {
        match self {
            GameInputAction::Quit => None,
            GameInputAction::ButtonDown(button) => Some(GuitarInputAction::FretDown(button.to_guitar())),
            GameInputAction::ButtonUp(button) => Some(GuitarInputAction::FretUp(button.to_guitar())),
            GameInputAction::Strum => Some(GuitarInputAction::Strum),
        }
    }
}

enum GameInputEffect {
    Quit,
    GuitarEffect(GuitarGameEffect),
}

fn draw_fret<T: sdl2::render::RenderTarget>(canvas: &sdl2::render::Canvas<T>, enabled: bool, x: i16, y: i16, radius: i16, color: pixels::Color) -> Result<(), String> {
    if enabled {
        canvas.filled_circle(x, y, radius, color)
    } else {
        canvas.circle(x, y, radius, color)
    }
}

enum FrameLimit {
    Vsync,
    Cap(u32),
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    /* joystick initialization */

    let joystick_subsystem = sdl_context.joystick()?;

    let available = joystick_subsystem.num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))?;

    println!("{} joysticks available", available);

    // Iterate over all available joysticks and stop once we manage to open one.
    let mut joystick = (0..available).find_map(|id| match joystick_subsystem.open(id) {
        Ok(c) => {
            println!("Success: opened \"{}\"", c.name());
            Some(c)
        },
        Err(e) => {
            println!("failed: {:?}", e);
            None
        },
    }).expect("Couldn't open any joystick");

    // Print the joystick's power level
    println!("\"{}\" power level: {:?}", joystick.name(), joystick.power_level()
        .map_err(|e| e.to_string())?);

    /* window initialization */

    let video_subsys = sdl_context.video()?;
    let window = video_subsys.window("bumpit", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;

    let mut playthrough: GuitarPlaythrough = std::fs::read_to_string("Songs/notes.chart")
        .map_err(|e| e.to_string())
        .and_then(|file| chart::read(file.as_ref())
            .map_err(|e| { println!("Error: {:?}", e); return String::from("couldn't parse chart") })) // TODO: error to string
        .and_then(|chart| GuitarPlaythrough::new(chart)
            .map_err(|s| String::from(s)))?;

    fn draw<T: sdl2::render::RenderTarget>(canvas: &mut sdl2::render::Canvas<T>, playthrough: &GuitarPlaythrough, time: f32) {
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        for i in 0..playthrough.notes_hit {
            let _ = draw_fret(&canvas, true, (i as i16) * 10, 10, 5, pixels::Color::RGB(255, 255, 255));
        }

        let frets = playthrough.frets;
        let _ = draw_fret(&canvas, frets[Fret::G as usize], 50, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(0, 128, 0));
        let _ = draw_fret(&canvas, frets[Fret::R as usize], 150, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(128, 0, 0));
        let _ = draw_fret(&canvas, frets[Fret::Y as usize], 250, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(128, 128, 0));
        let _ = draw_fret(&canvas, frets[Fret::B as usize], 350, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(0, 0, 128));
        let _ = draw_fret(&canvas, frets[Fret::O as usize], 450, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(192, 128, 00));

        for note in &playthrough.chart.notes {
            let position_past_time = playthrough.chart.ticks_to_ms(note.ticks) - time;
            let progress_on_screen = position_past_time / 1000f32;
            if progress_on_screen > 1f32 || progress_on_screen < 0f32 {
                continue;
            }
            let y = ((1f32 - progress_on_screen) * (SCREEN_HEIGHT as f32)) as i16 - 75;

            if note.is_open() {
                let _ = canvas.rectangle(50, y - 2, 462, y + 2, pixels::Color::RGB(200, 60, 200));
            } else {
                note.chord.iter()
                    .enumerate()
                    .filter(|(_i, chord_note)| **chord_note)
                    .for_each(|(note_index, _chord_note)| {
                        let _ = draw_fret(&canvas, true, 50 + (note_index as i16) * 100, y, 17, pixels::Color::RGB(60, 80, 100));
                    });
            }
        }

        canvas.present();
    };

    fn input<'a>(events: &'a mut sdl2::EventPump) -> impl Iterator<Item = Option<GameInputAction>> + 'a {
        events.poll_iter()
            .map(|event| match event {
                Event::Quit {..} => Some(GameInputAction::Quit),
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => Some(GameInputAction::Quit),

                Event::KeyDown { keycode : Some(Keycode::Z), .. } => Some(GameInputAction::ButtonDown(GameButton::Green)),
                Event::KeyDown { keycode : Some(Keycode::X), .. } => Some(GameInputAction::ButtonDown(GameButton::Red)),
                Event::KeyDown { keycode : Some(Keycode::C), .. } => Some(GameInputAction::ButtonDown(GameButton::Yellow)),
                Event::KeyDown { keycode : Some(Keycode::V), .. } => Some(GameInputAction::ButtonDown(GameButton::Blue)),
                Event::KeyDown { keycode : Some(Keycode::B), .. } => Some(GameInputAction::ButtonDown(GameButton::Orange)),

                Event::KeyUp { keycode : Some(Keycode::Z), .. } => Some(GameInputAction::ButtonUp(GameButton::Green)),
                Event::KeyUp { keycode : Some(Keycode::X), .. } => Some(GameInputAction::ButtonUp(GameButton::Red)),
                Event::KeyUp { keycode : Some(Keycode::C), .. } => Some(GameInputAction::ButtonUp(GameButton::Yellow)),
                Event::KeyUp { keycode : Some(Keycode::V), .. } => Some(GameInputAction::ButtonUp(GameButton::Blue)),
                Event::KeyUp { keycode : Some(Keycode::B), .. } => Some(GameInputAction::ButtonUp(GameButton::Orange)),

                Event::KeyDown { keycode : Some(Keycode::Space), .. } => Some(GameInputAction::Strum),

                Event::JoyButtonDown { button_idx : 0, .. } => Some(GameInputAction::ButtonDown(GameButton::Green)),
                Event::JoyButtonDown { button_idx : 1, .. } => Some(GameInputAction::ButtonDown(GameButton::Red)),
                Event::JoyButtonDown { button_idx : 3, .. } => Some(GameInputAction::ButtonDown(GameButton::Yellow)),
                Event::JoyButtonDown { button_idx : 2, .. } => Some(GameInputAction::ButtonDown(GameButton::Blue)),
                Event::JoyButtonDown { button_idx : 4, .. } => Some(GameInputAction::ButtonDown(GameButton::Orange)),

                Event::JoyButtonUp { button_idx : 0, .. } => Some(GameInputAction::ButtonUp(GameButton::Green)),
                Event::JoyButtonUp { button_idx : 1, .. } => Some(GameInputAction::ButtonUp(GameButton::Red)),
                Event::JoyButtonUp { button_idx : 3, .. } => Some(GameInputAction::ButtonUp(GameButton::Yellow)),
                Event::JoyButtonUp { button_idx : 2, .. } => Some(GameInputAction::ButtonUp(GameButton::Blue)),
                Event::JoyButtonUp { button_idx : 4, .. } => Some(GameInputAction::ButtonUp(GameButton::Orange)),

                Event::JoyHatMotion { hat_idx : 0, state : sdl2::joystick::HatState::Up, .. } => Some(GameInputAction::Strum),
                Event::JoyHatMotion { hat_idx : 0, state : sdl2::joystick::HatState::Down, .. } => Some(GameInputAction::Strum),

                _ => None
            })
    }

    // for power-saving. if Some, the game will sleep for
    const FRAME_LIMIT: Option<FrameLimit> = Option::Some(FrameLimit::Cap(120));

    // TODO: enable vsync based on frame_limit
    // https://wiki.libsdl.org/SDL_GL_SetSwapInterval

    // TODO: process inputs more frequently than once per frame?
    // avoidable if we have accurate input event timestamps? (+ assumption our processing is short)

    // TODO: when frame_limit is FPS cap, do measurements for sleep interval
    // that results in that frequency (at runtime)
    // and ensure game loop handles huge outliers in sleep wakeup time

    let mut music = ears::Sound::new("Songs/song.ogg")?;
    music.play();

    let mut previous_frame_time = Instant::now();
    let mut last_playhead_pos_ms = 0f32;
    let mut song_time_ms = 0f32;

    let mut run = true;
    while run {
        // https://www.reddit.com/r/gamedev/comments/13y26t/how_do_rhythm_games_stay_in_sync_with_the_music/c78aawd/
        let this_frame_time = Instant::now();
        song_time_ms += this_frame_time.duration_since(previous_frame_time).as_millis() as f32;
        previous_frame_time = this_frame_time;

        let playhead_pos_ms = music.get_offset() * 1000f32;
        if playhead_pos_ms != last_playhead_pos_ms {
            song_time_ms = (song_time_ms + playhead_pos_ms) / 2f32;
            last_playhead_pos_ms = playhead_pos_ms;
        }

        let effects = input(&mut events)
            .filter_map(|action| match action {
                Some(GameInputAction::Quit) => Some(GameInputEffect::Quit),
                Some(action) => match action.to_guitar_action() {
                    Some(guitar_action) => {
                        // sdl's event timestamps are always later than the OS timestamp
                        // so just assume that events are happening at this instant
                        // TODO: can we do better?
                        // TODO: track inputs for replays?
                        playthrough.apply(&guitar_action, song_time_ms).map(|e| GameInputEffect::GuitarEffect(e))
                    },
                    None => None,
                },
                None => None,
            });

        effects.for_each(|effect: GameInputEffect| {
            match effect {
                GameInputEffect::Quit => run = false,
                GameInputEffect::GuitarEffect(effect) => match effect {
                    Hit => (),
                    Overstrum => (),
                    MissStreak => (),
                    MissNoStreak => (),
                    ReleaseSustain => (),
                }
            }
        });

        playthrough.update_time(song_time_ms)
            .map(|e| GameInputEffect::GuitarEffect(e))
            .map(|effect: GameInputEffect| {
                match effect {
                    GameInputEffect::Quit => run = false,
                    GameInputEffect::GuitarEffect(effect) => match effect {
                        Hit => (),
                        Overstrum => (),
                        MissStreak => (),
                        MissNoStreak => (),
                        ReleaseSustain => (),
                    }
                }
            });

        draw(&mut canvas, &playthrough, song_time_ms);

        match FRAME_LIMIT {
            Some(FrameLimit::Vsync) => (), // present() waits for vsync if on
            Some(FrameLimit::Cap(cap)) => {
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / cap));
            },
            None => (),
        }
    }

    Ok(())
}
