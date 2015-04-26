#![deny(missing_docs)]
#![warn(dead_code)]

//! A user friendly game engine written in Rust.

#[cfg(feature = "include_gfx")]
extern crate libc;
#[cfg(feature = "include_gfx")]
extern crate gfx;
#[cfg(feature = "include_gfx")]
extern crate gfx_device_gl;
#[cfg(feature = "include_gfx")]
extern crate gfx_graphics;
#[cfg(feature = "include_opengl")]
extern crate opengl_graphics;
#[cfg(feature = "include_sdl2")]
extern crate sdl2;
#[cfg(feature = "include_sdl2")]
extern crate sdl2_window;
#[cfg(feature = "include_glfw")]
extern crate glfw;
#[cfg(feature = "include_glfw")]
extern crate glfw_window;
#[cfg(feature = "include_glutin")]
extern crate glutin;
#[cfg(feature = "include_glutin")]
extern crate glutin_window;
extern crate graphics;
extern crate fps_counter;
extern crate current;
extern crate shader_version;

extern crate piston;

#[cfg(feature = "include_sdl2")]
pub use sdl2_window::Sdl2Window as WindowBackEnd;
#[cfg(feature = "include_glfw")]
pub use glfw_window::GlfwWindow as WindowBackEnd;
#[cfg(feature = "include_glutin")]
pub use glutin_window::GlutinWindow as WindowBackEnd;

#[cfg(feature = "include_gfx")]
use gfx_graphics::Gfx2d;
#[cfg(feature = "include_gfx")]
use gfx_device_gl::{ Device, Resources, CommandBuffer, Output };

#[cfg(feature = "include_opengl")]
use opengl_graphics::GlGraphics;
use fps_counter::FPSCounter;

use std::rc::Rc;
use std::cell::RefCell;

use piston::window::WindowSettings;
use current::{ Current, CurrentGuard };

fn start_window<F>(
    opengl: shader_version::OpenGL,
    window_settings: WindowSettings,
    mut f: F
)
    where
        F: FnMut()
{
    let mut window = Rc::new(RefCell::new(WindowBackEnd::new(
        opengl,
        window_settings,
    )));
    let mut fps_counter = Rc::new(RefCell::new(FPSCounter::new()));

    #[cfg(feature = "include_sdl2")]
    fn disable_vsync() { sdl2::video::gl_set_swap_interval(0); }

    #[cfg(not(feature = "include_sdl2"))]
    fn disable_vsync() {}

    disable_vsync();

    let window_guard = CurrentGuard::new(&mut window);
    let fps_counter_guard = CurrentGuard::new(&mut fps_counter);

    f();

    drop(window_guard);
    drop(fps_counter_guard);
}

#[cfg(feature = "include_opengl")]
fn start_opengl<F>(opengl: shader_version::OpenGL, mut f: F)
    where
        F: FnMut()
{
    let mut gl = Rc::new(RefCell::new(GlGraphics::new(opengl)));
    
    let gl_guard = CurrentGuard::new(&mut gl);

    f();    

    drop(gl_guard);
}

#[cfg(not(feature = "include_opengl"))]
fn start_opengl<F>(_opengl: shader_version::OpenGL, mut f: F)
    where
        F: FnMut()
{
    f();
}


#[cfg(feature = "include_gfx")]
fn start_gfx<F>(mut f: F)
    where
        F: FnMut()
{
    use piston::window::Window;
    use gfx::render::RenderFactory;

    let window = current_window();

    let (device, mut factory) = gfx_device_gl::create(|s| {
        #[cfg(feature = "include_sdl2")]
        fn get_proc_address(_: &WindowBackEnd, s: &str) ->
            *const libc::types::common::c95::c_void {
            unsafe { std::mem::transmute(sdl2::video::gl_get_proc_address(s)) }
        }

        #[cfg(feature = "include_glfw")]
        fn get_proc_address(window: &mut WindowBackEnd, s: &str) ->
            *const libc::types::common::c95::c_void {
            window.window.get_proc_address(s)
        }

        #[cfg(feature = "include_glutin")]
        fn get_proc_address(window: &WindowBackEnd, s: &str) ->
            *const libc::types::common::c95::c_void {
            window.window.get_proc_address(s)
        }

        get_proc_address(&mut *window.borrow_mut(), s)
    });
    let mut renderer: Rc<RefCell<
        gfx::Renderer<gfx_device_gl::Resources,
        <gfx_device_gl::Device as gfx::Device>::CommandBuffer
    >>> =
        Rc::new(RefCell::new(factory.create_renderer()));
    let mut device = Rc::new(RefCell::new(device));
    let factory = Rc::new(RefCell::new(factory));
    let mut g2d = Rc::new(RefCell::new(Gfx2d::new(&mut *device.borrow_mut(), &mut *factory.borrow_mut())));
    let size = window.borrow().size(); 
    let mut output = Rc::new(RefCell::new(factory.borrow_mut().make_fake_output(
        size.width as u16, size.height as u16)));

    let device_guard = CurrentGuard::new(&mut device);
    let g2d_guard = CurrentGuard::new(&mut g2d);
    let renderer_guard = CurrentGuard::new(&mut renderer);
    let output_guard = CurrentGuard::new(&mut output);

    f();
    
    drop(g2d_guard);
    drop(renderer_guard);
    drop(output_guard);
    drop(device_guard);
}

#[cfg(not(feature = "include_gfx"))]
fn start_gfx<F>(mut f: F)
    where
        F: FnMut()
{
    f();
}

/// Initializes window and sets up current objects.
pub fn start<F>(
    opengl: shader_version::OpenGL,
    window_settings: WindowSettings,
    mut f: F
)
    where
        F: FnMut()
{
    start_window(opengl, window_settings, || {
        start_opengl(opengl, || {
            if cfg!(feature = "include_gfx") {
                start_gfx(|| f());
            } else {
                f();
            }
        })
    });
}

/// The current window
pub fn current_window() -> Rc<RefCell<WindowBackEnd>> {
    unsafe {
        Current::<Rc<RefCell<WindowBackEnd>>>::new().clone()
    }
}
/// The current Gfx device
#[cfg(feature = "include_gfx")]
pub fn current_gfx_device() -> Rc<RefCell<Device>> {
    unsafe {
        Current::<Rc<RefCell<Device>>>::new().clone()
    }
}
/// The current opengl_graphics back-end
#[cfg(feature = "include_opengl")]
pub fn current_gl() -> Rc<RefCell<GlGraphics>> {
    unsafe {
        Current::<Rc<RefCell<GlGraphics>>>::new().clone()
    }
}
/// The current gfx_graphics back-end
#[cfg(feature = "include_gfx")]
pub fn current_g2d() -> Rc<RefCell<Gfx2d<Resources>>> {
    unsafe {
        Current::<Rc<RefCell<Gfx2d<Resources>>>>::new().clone()
    }
}
/// The current Gfx renderer
#[cfg(feature = "include_gfx")]
pub fn current_renderer() -> Rc<RefCell<gfx::Renderer<Resources, CommandBuffer>>> {
    unsafe {
        Current::<Rc<RefCell<gfx::Renderer<Resources, CommandBuffer>>>>::new().clone()
    }
}
/// The current Gfx frame
#[cfg(feature = "include_gfx")]
pub fn current_output() -> Rc<RefCell<Output>> {
    unsafe {
        Current::<Rc<RefCell<Output>>>::new().clone()
    }
}
/// The current FPS counter
pub fn current_fps_counter() -> Rc<RefCell<FPSCounter>> {
    unsafe {
        Current::<Rc<RefCell<FPSCounter>>>::new().clone()
    }
}

/// Returns an event iterator for the event loop
pub fn events() -> piston::event::events::Events<WindowBackEnd, piston::event::Event> {
    use piston::event::Events;
    
    current_window().events()
}

/// Updates the FPS counter and gets the frames per second.
pub fn fps_tick() -> usize {
    let fps_counter = current_fps_counter();
    let mut fps_counter = fps_counter.borrow_mut();
    fps_counter.tick()
}

/// Sets title of the current window.
pub fn set_title(text: String) {
    use piston::window::AdvancedWindow;

    let window = current_window();
    let mut window = window.borrow_mut();
    window.set_title(text);
}

/// Returns true if the current window should be closed.
pub fn should_close() -> bool {
    use piston::window::Window;

    let window = current_window();
    let window = window.borrow();
    window.should_close()
}

/// Renders 2D graphics using Gfx.
#[cfg(feature = "include_gfx")]
pub fn render_2d_gfx<F>(
    bg_color: Option<[f32; 4]>, 
    mut f: F
)
    where
        F: FnMut(graphics::Context, 
            &mut gfx_graphics::GfxGraphics<Resources, CommandBuffer, Output>)
{
    use gfx::Device;
    use piston::window::Window;
    use graphics::Viewport;

    let window = current_window();
    let window = window.borrow();
    let size = window.size();
    let draw_size = window.draw_size();
    let renderer = current_renderer();
    let mut renderer = renderer.borrow_mut();
    let renderer = &mut *renderer;
    let gfx = current_g2d();
    let mut gfx = gfx.borrow_mut();
    let output = current_output();
    let output = output.borrow();
    gfx.draw(
        renderer,
        &*output,
        Viewport {
            rect: [0, 0, draw_size.width as i32, draw_size.height as i32],
            draw_size: [draw_size.width, draw_size.height],
            window_size: [size.width, size.height],
        },
        |c, g| {
            if let Some(bg_color) = bg_color {
                graphics::clear(bg_color, g);
            }
            f(c, g);
        });
    let gfx_device = current_gfx_device();
    let mut gfx_device = gfx_device.borrow_mut();
    gfx_device.submit(renderer.as_buffer());
    renderer.reset();
}

/// Renders 2D graphics using OpenGL.
///
/// Panics if called nested within the closure
/// to prevent mutable aliases to the graphics back-end.
#[cfg(feature = "include_opengl")]
pub fn render_2d_opengl<F>(
    bg_color: Option<[f32; 4]>,
    mut f: F
)
    where
        F: FnMut(graphics::Context, &mut opengl_graphics::GlGraphics)
{
    use piston::window::Window;
    use graphics::Viewport;

    let window = current_window();
    let window = window.borrow();
    let size = window.size();
    let draw_size = window.draw_size();
    let gl = current_gl();
    let mut gl = gl.borrow_mut();
    gl.draw(Viewport {
        rect: [0, 0, draw_size.width as i32, draw_size.height as i32],
        draw_size: [draw_size.width, draw_size.height],
        window_size: [size.width, size.height]
    }, |c, g| {
        use graphics::*;
        if let Some(bg_color) = bg_color {
            graphics::clear(bg_color, g);
        }
        f(c, g);
    });
}

