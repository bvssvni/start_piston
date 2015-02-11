#![deny(missing_docs)]
#![warn(dead_code)]

//! A user friendly game engine written in Rust.

#[cfg(feature = "include_gfx")]
extern crate libc;
#[cfg(feature = "include_gfx")]
extern crate gfx;
#[cfg(feature = "include_gfx")]
extern crate gfx_graphics;
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
use gfx_graphics::G2D;
#[cfg(feature = "include_gfx")]
use gfx::{ DeviceHelper };
use opengl_graphics::Gl;
use fps_counter::FPSCounter;

use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;

use piston::window::WindowSettings;
use piston::quack::{ Get, Set };
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
    let mut gl = Rc::new(RefCell::new(Gl::new(opengl)));
    let mut fps_counter = Rc::new(RefCell::new(FPSCounter::new()));

    #[cfg(feature = "include_sdl2")]
    fn disable_vsync() { sdl2::video::gl_set_swap_interval(0); }

    #[cfg(not(feature = "include_sdl2"))]
    fn disable_vsync() {}

    disable_vsync();

    let window_guard = CurrentGuard::new(&mut window);
    let gl_guard = CurrentGuard::new(&mut gl);
    let fps_counter_guard = CurrentGuard::new(&mut fps_counter);

    f();

    drop(window_guard);
    drop(gl_guard);
    drop(fps_counter_guard);
}

#[cfg(feature = "include_gfx")]
fn start_gfx<F>(mut f: F)
    where
        F: FnMut()
{
    let window = current_window();

    let mut device = Rc::new(RefCell::new(gfx::GlDevice::new(|s| {
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
    })));
    let mut g2d = Rc::new(RefCell::new(G2D::new(&mut *device.borrow_mut())));
    let mut renderer = Rc::new(RefCell::new(device.borrow_mut().create_renderer()));
    let piston::window::Size([w, h]) = window.get(); 
    let mut frame = Rc::new(RefCell::new(gfx::Frame::new(w as u16, h as u16)));

    let device_guard = CurrentGuard::new(&mut device);
    let g2d_guard = CurrentGuard::new(&mut g2d);
    let renderer_guard = CurrentGuard::new(&mut renderer);
    let frame_guard = CurrentGuard::new(&mut frame);

    f();
    
    drop(g2d_guard);
    drop(renderer_guard);
    drop(frame_guard);
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
        if cfg!(feature = "include_gfx") {
            start_gfx(|| f());
        } else {
            f();
        }
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
pub fn current_gfx_device() -> Rc<RefCell<gfx::GlDevice>> {
    unsafe {
        Current::<Rc<RefCell<gfx::GlDevice>>>::new().clone()
    }
}
/// The current opengl_graphics back-end
pub fn current_gl() -> Rc<RefCell<Gl>> {
    unsafe {
        Current::<Rc<RefCell<Gl>>>::new().clone()
    }
}
/// The current gfx_graphics back-end
#[cfg(feature = "include_gfx")]
pub fn current_g2d() -> Rc<RefCell<G2D>> {
    unsafe {
        Current::<Rc<RefCell<G2D>>>::new().clone()
    }
}
/// The current Gfx renderer
#[cfg(feature = "include_gfx")]
pub fn current_renderer() -> Rc<RefCell<gfx::Renderer<gfx::GlCommandBuffer>>> {
    unsafe {
        Current::<Rc<RefCell<gfx::Renderer<gfx::GlCommandBuffer>>>>::new().clone()
    }
}
/// The current Gfx frame
#[cfg(feature = "include_gfx")]
pub fn current_frame() -> Rc<RefCell<gfx::Frame>> {
    unsafe {
        Current::<Rc<RefCell<gfx::Frame>>>::new().clone()
    }
}
/// The current FPS counter
pub fn current_fps_counter() -> Rc<RefCell<FPSCounter>> {
    unsafe {
        Current::<Rc<RefCell<FPSCounter>>>::new().clone()
    }
}

/// Returns an event iterator for the event loop
pub fn events() -> piston::event::Events<
    Rc<RefCell<WindowBackEnd>>, piston::input::Input, piston::event::Event
> {
    piston::event::events(current_window())
}

/// Updates the FPS counter and gets the frames per second.
pub fn fps_tick() -> usize {
    current_fps_counter().borrow_mut().tick()
}

/// Sets title of the current window.
pub fn set_title(text: String) {
    current_window().set_mut(piston::window::Title(text));
}

/// Returns true if the current window should be closed.
pub fn should_close() -> bool {
    use piston::window::ShouldClose;
    let ShouldClose(val) = current_window().get();
    val
}

/// Renders 2D graphics using Gfx.
#[cfg(feature = "include_gfx")]
pub fn render_2d_gfx<F>(
    bg_color: Option<[f32; 4]>, 
    mut f: F
)
    where
        F: FnMut(graphics::Context, 
            &mut gfx_graphics::GraphicsBackEnd<gfx::GlCommandBuffer>)
{
    use gfx::Device;    

    let renderer = current_renderer();
    let mut renderer = renderer.borrow_mut();
    let renderer = &mut *renderer;
    current_g2d().borrow_mut().draw(
        renderer,
        &mut *current_frame().borrow_mut(), 
        |c, g| {
            if let Some(bg_color) = bg_color {
                graphics::clear(bg_color, g);
            }
            f(c, g);
        });
    current_gfx_device().borrow_mut().submit(renderer.as_buffer());
    renderer.reset();
}

/// Renders 2D graphics using OpenGL.
///
/// Panics if called nested within the closure
/// to prevent mutable aliases to the graphics back-end.
pub fn render_2d_opengl<F>(
    bg_color: Option<[f32; 4]>,
    mut f: F
)
    where
        F: FnMut(graphics::Context, &mut opengl_graphics::Gl)
{
    use std::ops::Deref;

    let piston::window::Size([w, h]) = current_window().borrow().deref().get();
    current_gl().borrow_mut().draw([0, 0, w as i32, h as i32], |c, g| {
        use graphics::*;
        if let Some(bg_color) = bg_color {
            graphics::clear(bg_color, g);
        }
        f(c, g);
    });
}

