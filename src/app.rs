use std::sync::Arc;
use pollster::FutureExt;

use instant::Instant;
#[cfg(not(web_platform))]
use std::time::Duration;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::Key;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::renderer::Renderer;

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = MedicalApp::default();
    let _ = event_loop.run_app(&mut app);
}

#[derive(Default)]
struct MedicalApp {
    close_requested: bool,
    last_update: Option<Instant>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>
}

impl ApplicationHandler for MedicalApp {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = WindowAttributes::default().with_title(
            "Medical App (FPS: ?)",
        ).with_inner_size(PhysicalSize::new(1000, 1000));

        #[cfg(web_platform)]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            window_attributes = window_attributes.with_append(true);
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.window = Some(window.clone());
        self.renderer = Some(Renderer::new(window.clone()).block_on());
        self.last_update = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key: key, state: ElementState::Pressed, .. },
                ..
            } => match key.as_ref() {
                Key::Character("Q") | Key::Character("q") => {
                    self.close_requested = true;
                },
                _ => (),
            },
        WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                window.pre_present_notify();
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // pollster::block_on(async {
            let renderer = self.renderer.as_mut().unwrap();
            let window = self.window.as_ref().unwrap();

            renderer.update();
            let _ = renderer.render();

            #[cfg(not(web_platform))]
            {
                if let Some(last_update) = self.last_update {
                    if last_update.elapsed() > Duration::from_secs(1) {
                        let fps = self.renderer.as_ref().unwrap().get_fps();
                        window.set_title(&format!("Medical App (FPS: {})", fps));
                        self.last_update = Some(Instant::now());
                    }
                }
            }
            window.request_redraw();
        // });

        if self.close_requested {
            event_loop.exit();
        }
    }
}
