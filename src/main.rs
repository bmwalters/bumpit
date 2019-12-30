extern crate sdl2;
extern crate ears;

use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;

use sdl2::gfx::primitives::DrawRenderer;

use ears::{Music, AudioController};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

#[derive(Copy, Clone)]
enum Fret {
    G = 0, R, Y, B, O
}

enum InputAction {
    Quit,
    FretDown(Fret),
    FretUp(Fret),
    Strum
}

struct Note {
    ticks: u32,
    lane: Fret,
    duration: u32,
}

struct Chart {
    ticks_per_beat: u32, // aka Resolution
    beats_per_minute: u32,
    notes: std::vec::Vec<Note>,
}

impl Chart {
    fn ticks_to_ms(self: &Self, ticks: u32) -> f32 {
        ((ticks as f32) / (self.ticks_per_beat as f32)) / (self.beats_per_minute as f32) * 60f32 * 1000f32
    }
}

struct Playthrough {
    chart: Chart,
    hit: u32,
    overstrums: u32,
}

fn draw_fret<T: sdl2::render::RenderTarget>(canvas: &sdl2::render::Canvas<T>, enabled: bool, x: i16, y: i16, radius: i16, color: pixels::Color) -> Result<(), String> {
    if enabled {
        canvas.filled_circle(x, y, radius, color)
    } else {
        canvas.circle(x, y, radius, color)
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys.window("bumpit", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;
    let mut timer_subsys = sdl_context.timer()?;

    let chart: Chart = Chart {
        ticks_per_beat: 192,
        beats_per_minute: 135,
        notes: vec![
        Note { ticks: 960, lane: Fret::R, duration: 144 },
        Note { ticks: 960, lane: Fret::Y, duration: 144 },
        Note { ticks: 960, lane: Fret::B, duration: 144 },
        Note { ticks: 1344, lane: Fret::O, duration: 0 },
        Note { ticks: 1536, lane: Fret::Y, duration: 0 },
        Note { ticks: 1728, lane: Fret::G, duration: 0 },
        Note { ticks: 1920, lane: Fret::R, duration: 0 },
        Note { ticks: 2112, lane: Fret::Y, duration: 0 },
        Note { ticks: 2880, lane: Fret::O, duration: 0 },
        Note { ticks: 3072, lane: Fret::Y, duration: 0 },
        Note { ticks: 3264, lane: Fret::G, duration: 0 },
        Note { ticks: 3456, lane: Fret::R, duration: 0 },
        Note { ticks: 3648, lane: Fret::Y, duration: 0 },
        Note { ticks: 4032, lane: Fret::R, duration: 144 },
        Note { ticks: 4032, lane: Fret::Y, duration: 144 },
        Note { ticks: 4032, lane: Fret::B, duration: 144 },
        Note { ticks: 4416, lane: Fret::O, duration: 0 },
        Note { ticks: 4608, lane: Fret::R, duration: 0 },
        Note { ticks: 4608, lane: Fret::Y, duration: 0 },
        Note { ticks: 4608, lane: Fret::B, duration: 0 },
        Note { ticks: 4800, lane: Fret::G, duration: 0 },
        Note { ticks: 4992, lane: Fret::R, duration: 0 },
        Note { ticks: 5184, lane: Fret::Y, duration: 0 },
        Note { ticks: 5568, lane: Fret::R, duration: 0 },
        Note { ticks: 5568, lane: Fret::Y, duration: 0 },
        Note { ticks: 5568, lane: Fret::B, duration: 0 },
        Note { ticks: 5568, lane: Fret::Y, duration: 1368 },
        Note { ticks: 5952, lane: Fret::O, duration: 0 },
        Note { ticks: 6144, lane: Fret::R, duration: 0 },
        Note { ticks: 6144, lane: Fret::Y, duration: 0 },
        Note { ticks: 6144, lane: Fret::B, duration: 0 },
        Note { ticks: 6336, lane: Fret::G, duration: 0 },
        Note { ticks: 6528, lane: Fret::R, duration: 0 },
        Note { ticks: 6720, lane: Fret::Y, duration: 0 },
        //Note { ticks: 6912, lane: 7, duration: 0 },
        Note { ticks: 7488, lane: Fret::Y, duration: 0 },
        Note { ticks: 7488, lane: Fret::B, duration: 0 },
        Note { ticks: 7584, lane: Fret::R, duration: 0 },
        Note { ticks: 7584, lane: Fret::Y, duration: 0 },
        Note { ticks: 7680, lane: Fret::Y, duration: 0 },
        Note { ticks: 7680, lane: Fret::B, duration: 0 },
        Note { ticks: 7776, lane: Fret::R, duration: 0 },
        Note { ticks: 7872, lane: Fret::Y, duration: 96 },
        Note { ticks: 8064, lane: Fret::Y, duration: 0 },
        Note { ticks: 8160, lane: Fret::R, duration: 0 },
        Note { ticks: 8256, lane: Fret::B, duration: 0 },
        Note { ticks: 8352, lane: Fret::Y, duration: 0 },
        Note { ticks: 8448, lane: Fret::R, duration: 0 },
        Note { ticks: 8448, lane: Fret::Y, duration: 0 },
        Note { ticks: 8448, lane: Fret::B, duration: 0 },
        Note { ticks: 8544, lane: Fret::Y, duration: 0 },
        Note { ticks: 8568, lane: Fret::B, duration: 0 },
        Note { ticks: 8640, lane: Fret::R, duration: 192 },
        Note { ticks: 9024, lane: Fret::Y, duration: 0 },
        Note { ticks: 9072, lane: Fret::R, duration: 0 },
        Note { ticks: 9120, lane: Fret::B, duration: 0 },
        Note { ticks: 9216, lane: Fret::Y, duration: 0 },
        Note { ticks: 9312, lane: Fret::R, duration: 0 },
        Note { ticks: 9360, lane: Fret::G, duration: 0 },
        Note { ticks: 9408, lane: Fret::R, duration: 0 },
        Note { ticks: 9432, lane: Fret::Y, duration: 0 },
        Note { ticks: 9504, lane: Fret::B, duration: 0 },
        //Note { ticks: 9504, lane: 5, duration: 0 },
        Note { ticks: 9888, lane: Fret::R, duration: 0 },
        Note { ticks: 9888, lane: Fret::Y, duration: 0 },
        Note { ticks: 9888, lane: Fret::B, duration: 0 },
        Note { ticks: 9984, lane: Fret::G, duration: 0 },
        Note { ticks: 9984, lane: Fret::R, duration: 0 },
        Note { ticks: 9984, lane: Fret::Y, duration: 0 },
        Note { ticks: 10560, lane: Fret::B, duration: 72 },]
    };

    let mut playthrough = Playthrough {
        chart: chart,
        hit: 0,
        overstrums: 0,
    };

    let mut frets: [bool; 5] = [false, false, false, false, false];
    frets[Fret::G as usize] = false;
    frets[Fret::R as usize] = false;
    frets[Fret::Y as usize] = false;
    frets[Fret::B as usize] = false;
    frets[Fret::O as usize] = false;

    fn draw<T: sdl2::render::RenderTarget>(canvas: &mut sdl2::render::Canvas<T>, playthrough: &Playthrough, frets: &[bool; 5], time: f32) {
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        for i in 0..playthrough.hit {
            let _ = draw_fret(&canvas, true, (i as i16) * 10, 10, 5, pixels::Color::RGB(255, 255, 255));
        }

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
            let _ = draw_fret(&canvas, true, 50 + (note.lane as i16) * 100, y, 17, pixels::Color::RGB(60, 80, 100));
        }

        canvas.present();
    };

    fn input<'a>(events: &'a mut sdl2::EventPump) -> impl Iterator<Item = Option<InputAction>> + 'a {
        events.poll_iter()
            .map(|event| match event {
                Event::Quit {..} => Some(InputAction::Quit),
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => Some(InputAction::Quit),

                Event::KeyDown { keycode : Some(Keycode::Z), .. } => Some(InputAction::FretDown(Fret::G)),
                Event::KeyDown { keycode : Some(Keycode::X), .. } => Some(InputAction::FretDown(Fret::R)),
                Event::KeyDown { keycode : Some(Keycode::C), .. } => Some(InputAction::FretDown(Fret::Y)),
                Event::KeyDown { keycode : Some(Keycode::V), .. } => Some(InputAction::FretDown(Fret::B)),
                Event::KeyDown { keycode : Some(Keycode::B), .. } => Some(InputAction::FretDown(Fret::O)),

                Event::KeyUp { keycode : Some(Keycode::Z), .. } => Some(InputAction::FretUp(Fret::G)),
                Event::KeyUp { keycode : Some(Keycode::X), .. } => Some(InputAction::FretUp(Fret::R)),
                Event::KeyUp { keycode : Some(Keycode::C), .. } => Some(InputAction::FretUp(Fret::Y)),
                Event::KeyUp { keycode : Some(Keycode::V), .. } => Some(InputAction::FretUp(Fret::B)),
                Event::KeyUp { keycode : Some(Keycode::B), .. } => Some(InputAction::FretUp(Fret::O)),

                Event::KeyDown { keycode : Some(Keycode::Space), .. } => Some(InputAction::Strum),

                _ => None
            })
    }

    let mut music = ears::Sound::new("test.ogg")?;
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

        draw(&mut canvas, &playthrough, &frets, song_time_ms);

        input(&mut events)
            .for_each(|action| match action {
                Some(InputAction::Quit) => run = false,
                Some(InputAction::FretDown(fret)) => frets[fret as usize] = true,
                Some(InputAction::FretUp(fret)) => frets[fret as usize] = false,
                Some(InputAction::Strum) => {
                    let first_near = playthrough.chart.notes.iter().find(|note|
                        (song_time_ms - playthrough.chart.ticks_to_ms(note.ticks)).abs() < 60f32);
                    match first_near {
                        None => playthrough.overstrums += 1,
                        Some(first_note) => {
                            if frets[first_note.lane as usize] {
                                playthrough.hit += 1;
                            }
                        }
                    }
                },
                None => (),
            });

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
