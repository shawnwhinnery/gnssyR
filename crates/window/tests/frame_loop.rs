use gfx::driver::{GraphicsDriver, MeshHandle, Vertex};
use glam::Mat3;
use input::event::{Button, InputEvent};
use input::player::PlayerId;
use input::SimulatedBackend;
use window::App;

// ---------------------------------------------------------------------------
// SpyDriver — records driver call labels in order
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SpyDriver {
    log: Vec<&'static str>,
}

impl GraphicsDriver for SpyDriver {
    fn begin_frame(&mut self) {
        self.log.push("begin_frame");
    }
    fn end_frame(&mut self) {
        self.log.push("end_frame");
    }
    fn present(&mut self) {
        self.log.push("present");
    }

    fn clear(&mut self, _color: [f32; 4]) {}

    fn upload_mesh(&mut self, _v: &[Vertex], _i: &[u32]) -> MeshHandle {
        0
    }
    fn draw_mesh(&mut self, _m: MeshHandle, _t: Mat3, _c: [f32; 4]) {}

    fn resize(&mut self, _w: u32, _h: u32) {}
    fn backend_name(&self) -> &'static str {
        "spy"
    }
    fn surface_size(&self) -> (u32, u32) {
        (800, 600)
    }
}

// ---------------------------------------------------------------------------
// Helper state types
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CounterState {
    counter: u32,
}

// ---------------------------------------------------------------------------
// 1. frame_sequence_begin_render_end_present
// ---------------------------------------------------------------------------

#[test]
fn frame_sequence_begin_render_end_present() {
    let mut render_called = false;

    App::run_frames(
        (),
        SimulatedBackend::new(),
        SpyDriver::default(),
        |_state, _events| {},
        |_state, _driver| {
            render_called = true;
        },
        1,
    );

    // render_called proves render_fn ran; driver call order is verified
    // by a dedicated SpyDriver test below.
    assert!(render_called);
}

#[test]
fn driver_call_order_single_frame() {
    // We need to capture the log out of the driver after run_frames.
    // Wrap it in a way the closures can observe it via shared mutable state.
    use std::cell::RefCell;
    use std::rc::Rc;

    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    struct LoggingDriver(Rc<RefCell<Vec<&'static str>>>);

    impl GraphicsDriver for LoggingDriver {
        fn begin_frame(&mut self) {
            self.0.borrow_mut().push("begin_frame");
        }
        fn end_frame(&mut self) {
            self.0.borrow_mut().push("end_frame");
        }
        fn present(&mut self) {
            self.0.borrow_mut().push("present");
        }
        fn clear(&mut self, _: [f32; 4]) {}
        fn upload_mesh(&mut self, _: &[Vertex], _: &[u32]) -> MeshHandle {
            0
        }
        fn draw_mesh(&mut self, _: MeshHandle, _: Mat3, _: [f32; 4]) {}
        fn resize(&mut self, _: u32, _: u32) {}
        fn backend_name(&self) -> &'static str {
            "spy"
        }
        fn surface_size(&self) -> (u32, u32) {
            (800, 600)
        }
    }

    let render_order = log.clone();
    App::run_frames(
        (),
        SimulatedBackend::new(),
        LoggingDriver(log.clone()),
        |_, _| {},
        move |_, _| render_order.borrow_mut().push("render"),
        1,
    );

    assert_eq!(
        *log.borrow(),
        vec!["begin_frame", "render", "end_frame", "present"],
    );
}

// ---------------------------------------------------------------------------
// 2. input_events_delivered_to_tick
// ---------------------------------------------------------------------------

#[test]
fn input_events_delivered_to_tick() {
    let mut backend = SimulatedBackend::new();
    let p = PlayerId(0);
    backend.push(InputEvent::Button {
        player: p,
        button: Button::South,
        pressed: true,
    });
    backend.push(InputEvent::MouseMove { dx: 1.0, dy: 2.0 });

    let mut received: Vec<InputEvent> = Vec::new();

    App::run_frames(
        (),
        backend,
        SpyDriver::default(),
        |_state, events| received.extend(events),
        |_, _| {},
        1,
    );

    assert_eq!(received.len(), 2);
    assert!(matches!(
        &received[0],
        InputEvent::Button {
            button: Button::South,
            pressed: true,
            ..
        }
    ));
    assert!(matches!(&received[1], InputEvent::MouseMove { dx, dy } if *dx == 1.0 && *dy == 2.0));
}

// ---------------------------------------------------------------------------
// 3. empty_poll_delivers_empty_events
// ---------------------------------------------------------------------------

#[test]
fn empty_poll_delivers_empty_events() {
    let mut received_count = 0usize;

    App::run_frames(
        (),
        SimulatedBackend::new(),
        SpyDriver::default(),
        |_state, events| received_count = events.len(),
        |_, _| {},
        1,
    );

    assert_eq!(received_count, 0);
}

// ---------------------------------------------------------------------------
// 4. tick_state_mutation_visible_in_render
// ---------------------------------------------------------------------------

#[test]
fn tick_state_mutation_visible_in_render() {
    let mut render_saw: u32 = 0;

    App::run_frames(
        CounterState::default(),
        SimulatedBackend::new(),
        SpyDriver::default(),
        |state, _events| state.counter += 1,
        |state, _driver| render_saw = state.counter,
        1,
    );

    assert_eq!(render_saw, 1);
}

// ---------------------------------------------------------------------------
// 5. multiple_frames_call_counts
// ---------------------------------------------------------------------------

#[test]
fn multiple_frames_call_counts() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    struct LoggingDriver(Rc<RefCell<Vec<&'static str>>>);
    impl GraphicsDriver for LoggingDriver {
        fn begin_frame(&mut self) {
            self.0.borrow_mut().push("begin_frame");
        }
        fn end_frame(&mut self) {
            self.0.borrow_mut().push("end_frame");
        }
        fn present(&mut self) {
            self.0.borrow_mut().push("present");
        }
        fn clear(&mut self, _: [f32; 4]) {}
        fn upload_mesh(&mut self, _: &[Vertex], _: &[u32]) -> MeshHandle {
            0
        }
        fn draw_mesh(&mut self, _: MeshHandle, _: Mat3, _: [f32; 4]) {}
        fn resize(&mut self, _: u32, _: u32) {}
        fn backend_name(&self) -> &'static str {
            "spy"
        }
        fn surface_size(&self) -> (u32, u32) {
            (800, 600)
        }
    }

    App::run_frames(
        (),
        SimulatedBackend::new(),
        LoggingDriver(log.clone()),
        |_, _| {},
        |_, _| {},
        3,
    );

    let log = log.borrow();
    let begins = log.iter().filter(|&&s| s == "begin_frame").count();
    let ends = log.iter().filter(|&&s| s == "end_frame").count();
    let presents = log.iter().filter(|&&s| s == "present").count();
    assert_eq!(begins, 3, "begin_frame count");
    assert_eq!(ends, 3, "end_frame count");
    assert_eq!(presents, 3, "present count");
}

// ---------------------------------------------------------------------------
// 6. events_cleared_between_frames
// ---------------------------------------------------------------------------

#[test]
fn events_cleared_between_frames() {
    let mut backend = SimulatedBackend::new();
    let p = PlayerId(0);
    backend.push(InputEvent::Button {
        player: p,
        button: Button::Start,
        pressed: true,
    });

    let mut counts: Vec<usize> = Vec::new();

    App::run_frames(
        (),
        backend,
        SpyDriver::default(),
        |_state, events| counts.push(events.len()),
        |_, _| {},
        2,
    );

    assert_eq!(counts, vec![1, 0]);
}

// ---------------------------------------------------------------------------
// 7. render_sees_cumulative_state
// ---------------------------------------------------------------------------

#[test]
fn render_sees_cumulative_state() {
    let mut observed: Vec<u32> = Vec::new();

    App::run_frames(
        CounterState::default(),
        SimulatedBackend::new(),
        SpyDriver::default(),
        |state, _events| state.counter += 1,
        |state, _driver| observed.push(state.counter),
        3,
    );

    assert_eq!(observed, vec![1, 2, 3]);
}

// ---------------------------------------------------------------------------
// 8. factory_is_called_to_create_driver
//
// Verifies that run_frames_with_factory actually invokes the factory.
// The original bug was that App::run took a pre-built driver, making it
// impossible for GPU drivers to receive the window handle. Had this test
// existed from the start, the deferred-construction contract would have been
// encoded in the API from day one.
// ---------------------------------------------------------------------------

#[test]
fn factory_is_called_to_create_driver() {
    use std::cell::Cell;
    use std::rc::Rc;

    let called = Rc::new(Cell::new(false));
    let called2 = called.clone();

    App::run_frames_with_factory(
        (),
        SimulatedBackend::new(),
        move || {
            called2.set(true);
            SpyDriver::default()
        },
        |_, _| {},
        |_, _| {},
        1,
    );

    assert!(called.get(), "make_driver factory was never called");
}

// ---------------------------------------------------------------------------
// W-M1. cursor_moved_delivered_to_tick
// ---------------------------------------------------------------------------

#[test]
fn cursor_moved_delivered_to_tick() {
    let mut backend = SimulatedBackend::new();
    backend.push(InputEvent::CursorMoved { x: 0.4, y: -0.6 });

    let mut received: Vec<InputEvent> = Vec::new();

    App::run_frames(
        (),
        backend,
        SpyDriver::default(),
        |_state, events| received.extend(events),
        |_, _| {},
        1,
    );

    assert_eq!(received.len(), 1);
    assert!(matches!(&received[0], InputEvent::CursorMoved { x, y } if *x == 0.4 && *y == -0.6));
}

// ---------------------------------------------------------------------------
// W-M2. two_cursor_moved_both_reach_tick
// ---------------------------------------------------------------------------

#[test]
fn two_cursor_moved_both_reach_tick() {
    let mut backend = SimulatedBackend::new();
    backend.push(InputEvent::CursorMoved { x: -0.5, y: 0.5 });
    backend.push(InputEvent::CursorMoved { x: 0.5, y: -0.5 });

    let mut received: Vec<InputEvent> = Vec::new();

    App::run_frames(
        (),
        backend,
        SpyDriver::default(),
        |_state, events| received.extend(events),
        |_, _| {},
        1,
    );

    assert_eq!(received.len(), 2);
    assert!(matches!(&received[0], InputEvent::CursorMoved { x, y } if *x == -0.5 && *y ==  0.5));
    assert!(matches!(&received[1], InputEvent::CursorMoved { x, y } if *x ==  0.5 && *y == -0.5));
}

// ---------------------------------------------------------------------------
// W-M3. cursor_moved_cleared_between_frames
// ---------------------------------------------------------------------------

#[test]
fn cursor_moved_cleared_between_frames() {
    let mut backend = SimulatedBackend::new();
    backend.push(InputEvent::CursorMoved { x: 0.1, y: 0.2 });

    let mut counts: Vec<usize> = Vec::new();

    App::run_frames(
        (),
        backend,
        SpyDriver::default(),
        |_state, events| counts.push(events.len()),
        |_, _| {},
        2,
    );

    assert_eq!(counts, vec![1, 0]);
}

// ---------------------------------------------------------------------------
// 9. factory_driver_is_used_for_frames
//
// Verifies that the driver returned by the factory is the one that receives
// begin_frame / end_frame / present — not some default or discarded driver.
// ---------------------------------------------------------------------------

#[test]
fn factory_driver_is_used_for_frames() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    struct LoggingDriver(Rc<RefCell<Vec<&'static str>>>);
    impl GraphicsDriver for LoggingDriver {
        fn begin_frame(&mut self) {
            self.0.borrow_mut().push("begin_frame");
        }
        fn end_frame(&mut self) {
            self.0.borrow_mut().push("end_frame");
        }
        fn present(&mut self) {
            self.0.borrow_mut().push("present");
        }
        fn clear(&mut self, _: [f32; 4]) {}
        fn upload_mesh(&mut self, _: &[Vertex], _: &[u32]) -> MeshHandle {
            0
        }
        fn draw_mesh(&mut self, _: MeshHandle, _: Mat3, _: [f32; 4]) {}
        fn resize(&mut self, _: u32, _: u32) {}
        fn backend_name(&self) -> &'static str {
            "spy"
        }
        fn surface_size(&self) -> (u32, u32) {
            (800, 600)
        }
    }

    let log2 = log.clone();
    App::run_frames_with_factory(
        (),
        SimulatedBackend::new(),
        move || LoggingDriver(log2),
        |_, _| {},
        |_, _| {},
        2,
    );

    let log = log.borrow();
    assert_eq!(
        *log,
        vec![
            "begin_frame",
            "end_frame",
            "present",
            "begin_frame",
            "end_frame",
            "present"
        ],
        "factory-created driver was not used for all frames",
    );
}
