extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;

use sdl2::gfx::primitives::DrawRenderer;

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
    lane: Fret,
    time: u32,
}

struct Chart<T> {
    notes: T
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

    const chart: Chart<[Note; 1]> = Chart {
        notes: [Note { lane: Fret::G, time: 2 }]
    };

    let mut frets: [bool; 5] = [false, false, false, false, false];
    frets[Fret::G as usize] = false;
    frets[Fret::R as usize] = false;
    frets[Fret::Y as usize] = false;
    frets[Fret::B as usize] = false;
    frets[Fret::O as usize] = false;

    fn draw<T: sdl2::render::RenderTarget>(canvas: &mut sdl2::render::Canvas<T>, frets: &[bool; 5]) {
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        let _ = draw_fret(&canvas, frets[Fret::G as usize], 50, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(0, 128, 0));
        let _ = draw_fret(&canvas, frets[Fret::R as usize], 150, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(128, 0, 0));
        let _ = draw_fret(&canvas, frets[Fret::Y as usize], 250, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(128, 128, 0));
        let _ = draw_fret(&canvas, frets[Fret::B as usize], 350, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(0, 0, 128));
        let _ = draw_fret(&canvas, frets[Fret::O as usize], 450, (SCREEN_HEIGHT as i16) - 75, 25, pixels::Color::RGB(192, 128, 00));

        for note in &chart.notes {
            let _ = draw_fret(&canvas, true, 50 + (note.lane as i16) * 100, 50, 17, pixels::Color::RGB(60, 80, 100));
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

                _ => None
            })
    }

    let mut run = true;
    while run {
        draw(&mut canvas, &frets);

        input(&mut events)
            .for_each(|action| match action {
                Some(InputAction::Quit) => run = false,
                Some(InputAction::FretDown(fret)) => frets[fret as usize] = true,
                Some(InputAction::FretUp(fret)) => frets[fret as usize] = false,
                Some(InputAction::Strum) => std::todo!(),
                None => (),
            });

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
