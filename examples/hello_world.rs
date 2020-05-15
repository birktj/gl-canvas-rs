extern crate glium;
extern crate nalgebra as na; 
extern crate gl_canvas_rs;

use std::io::Write;

use glium::Surface;
use glium::glutin::{Event, self, WindowEvent};

fn main() {
    let mut event_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions((1024, 768).into());
    let context = glutin::ContextBuilder::new()
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    
    let mut ctx = gl_canvas_rs::RenderContext::new(display.clone());

    let mut time = std::time::Instant::now();
    let mut closed = false;

    print!("fps: ");
    while !closed {
        print!("\rfps: {}", 1000000.0 / time.elapsed().subsec_micros() as f32);
        std::io::stdout().flush().unwrap();
        time = std::time::Instant::now();
        
        ctx.clear(gl_canvas_rs::Color::new(0.5, 0.5, 0.5, 1.0));
        
        ctx.move_to(0.0, 0.0);
        ctx.line_to(100.0,100.0);
        ctx.stroke();

        ctx.move_to(200.0, 200.0);
        ctx.line_to(200.0, 500.0);
        ctx.line_to(500.0, 500.0);
        ctx.line_to(500.0, 100.0);
        //ctx.line_to(700.0, 1000.0);
        ctx.fill_color(gl_canvas_rs::Color::new(1.0,0.0,0.0,1.0));
        ctx.fill();

        ctx.render();

        event_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => closed = true,
                _ => (),
            }
        });
    }
}
