use std::time::{Duration};

use instant::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder};

use crate::renderer::Renderer;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("wgpu test")
        .build(&event_loop).unwrap();

    App::new(&window).await.run(event_loop);
}

struct App<'a> {
    window: &'a Window,
    renderer: Renderer<'a>,
    last_update: Instant
}

impl<'a> App<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let renderer = Renderer::new(window).await;
        let last_update = Instant::now();
        Self {
            window,
            renderer,
            last_update
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, elwt| {

            // while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
            //     self.input_manager.gilrs_update(&event);
            // }

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    self.handle_window_event(event, elwt);
                },
                _ => {}
            }
        }).unwrap();
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        // let event_response = self.ui.egui_state_mut().on_window_event(self.window, event);
        //
        // if event_response.repaint {
        //     self.window.request_redraw();
        // }

        match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::KeyboardInput { .. } |
            WindowEvent::MouseWheel { .. } |
            WindowEvent::MouseInput { .. } |
            WindowEvent::CursorMoved { .. } => {}, 
                // self.input_manager.window_update(event, event_response.consumed),
            _ => {}
        }
    }

    fn render(&mut self) {
        pollster::block_on(async move {
            self.renderer.update();
            let _ = self.renderer.render().await;
            if self.last_update.elapsed() > Duration::from_secs(1) {
                let fps = self.renderer.get_fps();
                self.window.set_title(&format!("Fps: {}", fps));
                self.last_update = Instant::now();
            }
            self.window.request_redraw();
        });
    }

    fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.renderer.resize(size.width, size.height);
    }
}

