use crate::WDW_HEIGHT;
use crate::WDW_SIZE_SCALAR;
use crate::WDW_WIDTH;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use sdl2::{pixels::Color, rect::Rect};

pub struct Renderer {
    canvas: WindowCanvas,
    display_backing: [bool; WDW_WIDTH as usize * WDW_HEIGHT as usize],
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        // initialize to all pixels off at first
        let display_backing = [false; WDW_WIDTH as usize * WDW_HEIGHT as usize];
        Ok(Renderer {
            canvas,
            display_backing,
        })
    }
    pub fn draw(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        // i = WDW_WIDTH * y + x
        for x in 0usize..WDW_WIDTH as usize {
            for y in 0usize..WDW_HEIGHT as usize {
                let i = (WDW_WIDTH as usize * y) + x;
                if self.display_backing[i] {
                    // print!("1");
                    self.draw_spot(x as i32, y as i32)?;
                } else {
                    // print!("0");
                }
            }
            // print!("\n");
        }
        // print!("\n");
        self.canvas.present();
        Ok(())
    }
    pub fn draw_spot(&mut self, x: i32, y: i32) -> Result<(), String> {
        self.canvas.fill_rect(Rect::new(
            x * WDW_SIZE_SCALAR as i32,
            y * WDW_SIZE_SCALAR as i32,
            WDW_SIZE_SCALAR,
            WDW_SIZE_SCALAR,
        ))?;
        Ok(())
    }
    pub fn set_display_at_location(
        &mut self,
        x: usize,
        y: usize,
        value: bool,
    ) -> Result<(), String> {
        self.display_backing[WDW_WIDTH as usize * y + x] = value;
        Ok(())
    }
    pub fn print_debug(&mut self) {
        println!("{:?}", self.display_backing);
        for y in 0..WDW_HEIGHT as usize {
            for x in 0..WDW_WIDTH as usize {
                print!(
                    "{}",
                    if self.display_backing[WDW_WIDTH as usize * y + x] {
                        1
                    } else {
                        0
                    }
                );
            }
            println!();
        }
    }
    pub fn get_display_at_location(&self, x: usize, y: usize) -> Result<bool, String> {
        Ok(self.display_backing
            [(WDW_WIDTH as usize * y + x) % (WDW_WIDTH as usize * WDW_HEIGHT as usize)])
    }
    pub fn replace_display(
        &mut self,
        replacement: [bool; WDW_WIDTH as usize * WDW_HEIGHT as usize],
    ) {
        self.display_backing = replacement;
    }
    pub fn clear_display(&mut self) {
        self.replace_display([false; WDW_WIDTH as usize * WDW_HEIGHT as usize])
    }
}
