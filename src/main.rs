extern crate sdl2;

mod chip8;

use chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::pixels;
use std::env;
use std::time::Duration;

const SCALE_FACTOR: u32 = 20;
const SCREEN_WIDTH: u32 = 64 * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = 32 * SCALE_FACTOR;

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(format!("Usage: ./chipulator8 chip8application"))
    }
    let mut chip = Chip8::new();
    chip.load_application(&args[1]);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("chipulator8", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => chip.key[0x1] = 1,
                    Keycode::Num2 => chip.key[0x2] = 1,
                    Keycode::Num3 => chip.key[0x3] = 1,
                    Keycode::Num4 => chip.key[0xC] = 1,

                    Keycode::Q => chip.key[0x4] = 1,
                    Keycode::W => chip.key[0x5] = 1,
                    Keycode::E => chip.key[0x6] = 1,
                    Keycode::R => chip.key[0xD] = 1,

                    Keycode::A => chip.key[0x7] = 1,
                    Keycode::S => chip.key[0x8] = 1,
                    Keycode::D => chip.key[0x9] = 1,
                    Keycode::F => chip.key[0xE] = 1,

                    Keycode::Y => chip.key[0xA] = 1,
                    Keycode::X => chip.key[0x0] = 1,
                    Keycode::C => chip.key[0xB] = 1,
                    Keycode::V => chip.key[0xF] = 1,
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => chip.key[0x1] = 0,
                    Keycode::Num2 => chip.key[0x2] = 0,
                    Keycode::Num3 => chip.key[0x3] = 0,
                    Keycode::Num4 => chip.key[0xC] = 0,

                    Keycode::Q => chip.key[0x4] = 0,
                    Keycode::W => chip.key[0x5] = 0,
                    Keycode::E => chip.key[0x6] = 0,
                    Keycode::R => chip.key[0xD] = 0,

                    Keycode::A => chip.key[0x7] = 0,
                    Keycode::S => chip.key[0x8] = 0,
                    Keycode::D => chip.key[0x9] = 0,
                    Keycode::F => chip.key[0xE] = 0,

                    Keycode::Y => chip.key[0xA] = 0,
                    Keycode::X => chip.key[0x0] = 0,
                    Keycode::C => chip.key[0xB] = 0,
                    Keycode::V => chip.key[0xF] = 0,
                    _ => {}
                },
                _ => {}
            }
        }
        chip.emulate_cycle();
        if chip.draw_flag {
            canvas.clear();
            for y in 0..32 {
                for x in 0..64 {
                    let mut color = pixels::Color::RGB(0, 0, 0);
                    if chip.gfx[((y * 64) + x) as usize] != 0 {
                        color = pixels::Color::RGB(255, 255, 255);
                    }
                    canvas.set_draw_color(color);
                    let x = x * SCALE_FACTOR;
                    let y = y * SCALE_FACTOR;
                    let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR))?;
                }
            }
            chip.draw_flag = false;
        }
        canvas.present();
        ::std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}
