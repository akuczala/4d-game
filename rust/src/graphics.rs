pub mod graphics2d;
pub mod graphics3d;
use glium::glutin;

//use glium::Surface;
use glium::glutin::dpi::LogicalSize;


fn init_glium() -> (winit::EventsLoop, glium::Display){
    let events_loop = glutin::EventsLoop::new();
    let size = LogicalSize{width : 1024.0,height : 768.0};
    let wb = glutin::WindowBuilder::new()
        .with_dimensions(size)
        .with_title("dim4");;
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    (events_loop,display)
}

// listing the events produced by application and waiting to be received
fn listen_events(events_loop :  &mut winit::EventsLoop, mut closed: &mut bool) {
    events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => *closed = true,
                    _ => (),
                },
                _ => (),
            }
        });
}
