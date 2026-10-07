//! In-process protocol fixture: verify real destructor requests without a GPU.
use super::*;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use wayland_protocols_wlr::layer_shell::v1::server::{
    zwlr_layer_shell_v1 as shell, zwlr_layer_surface_v1 as layer,
};
use wayland_server::{
    protocol::{wl_callback as scb, wl_compositor as sc, wl_output as so, wl_surface as ss},
    Client, DataInit, DisplayHandle, GlobalDispatch, New,
};

type Log = Arc<Mutex<Vec<&'static str>>>;
struct Server(Log, Vec<scb::WlCallback>);

macro_rules! global {
    ($ty:ty) => {
        impl GlobalDispatch<$ty, ()> for Server {
            fn bind(
                _: &mut Self,
                _: &DisplayHandle,
                _: &Client,
                resource: New<$ty>,
                _: &(),
                init: &mut DataInit<'_, Self>,
            ) {
                init.init(resource, ());
            }
        }
    };
}
global!(sc::WlCompositor);
global!(shell::ZwlrLayerShellV1);
impl GlobalDispatch<so::WlOutput, bool> for Server {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<so::WlOutput>,
        named: &bool,
        init: &mut DataInit<'_, Self>,
    ) {
        let output = init.init(resource, ());
        output.mode(so::Mode::Current, 640, 360, 60000);
        if *named {
            output.name("test-output".into());
        }
        output.done();
    }
}
macro_rules! requests {
    ($ty:ty, $state:ident, $request:ident, $init:ident, $body:block) => {
        impl wayland_server::Dispatch<$ty, ()> for Server {
            fn request($state: &mut Self, _: &Client, _: &$ty, $request: <$ty as wayland_server::Resource>::Request, _: &(), _: &DisplayHandle, $init: &mut DataInit<'_, Self>) $body
        }
    };
}
requests!(sc::WlCompositor, state, request, init, {
    if let sc::Request::CreateSurface { id } = request {
        init.init(id, ());
        state.0.lock().unwrap().push("surface-created");
    }
});
requests!(shell::ZwlrLayerShellV1, state, request, init, {
    match request {
        shell::Request::GetLayerSurface { id, .. } => {
            init.init(id, ());
            state.0.lock().unwrap().push("layer-created");
        }
        shell::Request::Destroy => state.0.lock().unwrap().push("shell-destroyed"),
        _ => {}
    }
});
requests!(ss::WlSurface, state, request, init, {
    match request {
        ss::Request::Destroy => state.0.lock().unwrap().push("surface-destroyed"),
        ss::Request::Frame { callback } => {
            state.1.push(init.init(callback, ()));
        }
        _ => {}
    }
});
requests!(layer::ZwlrLayerSurfaceV1, state, request, _init, {
    if let layer::Request::Destroy = request {
        state.0.lock().unwrap().push("layer-destroyed");
    }
});
requests!(so::WlOutput, state, request, _init, {
    if let so::Request::Release = request {
        state.0.lock().unwrap().push("output-released");
    }
});
requests!(scb::WlCallback, _state, _request, _init, {});

struct Fixture {
    queue: wayland_client::EventQueue<EngineState>,
    state: EngineState,
    log: Log,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    events: mpsc::Receiver<EngineEvent>,
    globals: mpsc::Sender<(bool, mpsc::Sender<()>)>,
}
impl Fixture {
    fn new(named: bool) -> Self {
        let (client, server) = UnixStream::pair().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (globals, commands) = mpsc::channel::<(bool, mpsc::Sender<()>)>();
        let server_log = log.clone();
        let server_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            let mut display = wayland_server::Display::<Server>::new().unwrap();
            let mut handle = display.handle();
            handle.create_global::<Server, sc::WlCompositor, _>(4, ());
            handle.create_global::<Server, shell::ZwlrLayerShellV1, _>(4, ());
            let mut output_global = Some(
                handle.create_global::<Server, so::WlOutput, _>(if named { 4 } else { 3 }, named),
            );
            handle.insert_client(server, Arc::new(())).unwrap();
            let mut state = Server(server_log, Vec::new());
            while !server_stop.load(Ordering::Relaxed) {
                for (connected, ack) in commands.try_iter() {
                    if connected {
                        output_global = Some(handle.create_global::<Server, so::WlOutput, _>(
                            if named { 4 } else { 3 },
                            named,
                        ));
                    } else if let Some(global) = output_global.take() {
                        handle.remove_global::<Server>(global);
                    }
                    ack.send(()).unwrap();
                }
                display.dispatch_clients(&mut state).unwrap();
                for callback in state.1.drain(..) {
                    callback.done(0);
                }
                display.flush_clients().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        let connection = Connection::from_socket(client).unwrap();
        let (events_tx, events) = mpsc::channel();
        let mut queue = connection.new_event_queue::<EngineState>();
        let qh = queue.handle();
        let mut state = EngineState {
            outputs: OutputManager::new(),
            sessions: HashMap::new(),
            events_tx,
            config: EngineConfig::default(),
            running: true,
            pending_outputs: HashMap::new(),
            compositor: None,
            layer_shell: None,
            egl_context: None,
            layer_surfaces: HashMap::new(),
            queue_handle: Some(qh.clone()),
            on_battery: false,
            power_paused: false,
            last_battery_check: std::time::Instant::now(),
            next_surface_generation: 0,
            connection: connection.clone(),
        };
        connection.display().get_registry(&qh, ());
        queue.roundtrip(&mut state).unwrap();
        queue.roundtrip(&mut state).unwrap();
        Self {
            queue,
            state,
            log,
            stop,
            thread: Some(thread),
            events,
            globals,
        }
    }
    fn hotplug(&mut self, connected: bool) {
        let (tx, rx) = mpsc::channel();
        self.globals.send((connected, tx)).unwrap();
        rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        self.sync();
        self.sync();
    }
    fn sync(&mut self) {
        self.queue.roundtrip(&mut self.state).unwrap();
    }
    fn name(&self) -> String {
        self.state
            .outputs
            .ready_outputs()
            .next()
            .unwrap()
            .1
            .name
            .clone()
    }
    fn apply(&mut self, name: &str) {
        apply_wallpaper_to_output(
            &mut self.state,
            std::path::Path::new("/wallpaper.png"),
            name,
            &self.queue.handle(),
        )
        .unwrap();
    }
    fn count(&self, event: &str) -> usize {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|&&x| x == event)
            .count()
    }
    fn closed(&mut self, identity: &SurfaceIdentity, proxy: &ZwlrLayerSurfaceV1) {
        let connection = self.state.connection.clone();
        <EngineState as Dispatch<ZwlrLayerSurfaceV1, SurfaceIdentity>>::event(
            &mut self.state,
            proxy,
            zwlr_layer_surface_v1::Event::Closed,
            identity,
            &connection,
            &self.queue.handle(),
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.state.cleanup();
        if let Some(shell) = self.state.layer_shell.take() {
            shell.destroy();
        }
        let synced = self.queue.roundtrip(&mut self.state);
        self.stop.store(true, Ordering::Relaxed);
        let joined = self.thread.take().unwrap().join();
        if !std::thread::panicking() {
            synced.unwrap();
            joined.unwrap();
        }
    }
}

#[test]
fn protocol_clear_replace_and_stale_events() {
    let mut fixture = Fixture::new(true);
    let name = fixture.name();
    fixture.apply(&name);
    let original = fixture.state.layer_surfaces[&name].wl_surface.id();
    let proxy = fixture.state.layer_surfaces[&name].layer_surface.clone();
    let identity = SurfaceIdentity {
        output: name.clone(),
        generation: fixture.state.layer_surfaces[&name].generation,
    };
    let callback = fixture.state.layer_surfaces[&name]
        .wl_surface
        .frame(&fixture.queue.handle(), identity.clone());
    fixture.apply(&name); // Replacement before configure must reuse the window.
    fixture.sync();
    assert_eq!(
        fixture.state.layer_surfaces[&name].wl_surface.id(),
        original
    );
    assert_eq!(fixture.count("surface-created"), 1);
    assert_eq!(fixture.count("surface-destroyed"), 0);
    handle_command(
        EngineCommand::ClearWallpaper {
            output: Some(name.clone()),
        },
        &mut fixture.state,
    );
    assert!(fixture.state.sessions.is_empty());
    fixture.apply(&name);
    assert_ne!(
        fixture.state.layer_surfaces[&name].wl_surface.id(),
        original
    );
    fixture.closed(&identity, &proxy);
    assert_eq!(fixture.state.sessions.len(), 1);
    assert!(fixture.state.current_surface(&identity).is_none());
    let connection = fixture.state.connection.clone();
    <EngineState as Dispatch<ZwlrLayerSurfaceV1, SurfaceIdentity>>::event(
        &mut fixture.state,
        &proxy,
        zwlr_layer_surface_v1::Event::Configure {
            serial: 99,
            width: 1,
            height: 1,
        },
        &identity,
        &connection,
        &fixture.queue.handle(),
    );
    <EngineState as Dispatch<WlCallback, SurfaceIdentity>>::event(
        &mut fixture.state,
        &callback,
        wl_callback::Event::Done { callback_data: 1 },
        &identity,
        &connection,
        &fixture.queue.handle(),
    );
    assert!(!fixture.state.layer_surfaces[&name].configured);
    assert!(!fixture.state.layer_surfaces[&name].frame_pending);
    let current = fixture.state.layer_surfaces[&name].layer_surface.clone();
    let current_identity = SurfaceIdentity {
        output: name.clone(),
        generation: fixture.state.layer_surfaces[&name].generation,
    };
    let connection = fixture.state.connection.clone();
    <EngineState as Dispatch<ZwlrLayerSurfaceV1, SurfaceIdentity>>::event(
        &mut fixture.state,
        &current,
        zwlr_layer_surface_v1::Event::Configure {
            serial: 1,
            width: 640,
            height: 360,
        },
        &current_identity,
        &connection,
        &fixture.queue.handle(),
    );
    assert!(fixture.state.layer_surfaces[&name].configured);
    fixture
        .state
        .layer_surfaces
        .get_mut(&name)
        .unwrap()
        .frame_pending = false;
    <EngineState as Dispatch<WlCallback, SurfaceIdentity>>::event(
        &mut fixture.state,
        &callback,
        wl_callback::Event::Done { callback_data: 2 },
        &current_identity,
        &connection,
        &fixture.queue.handle(),
    );
    assert!(fixture.state.layer_surfaces[&name].frame_pending);
    fixture.closed(&current_identity, &current);
    fixture.closed(&current_identity, &current);
    fixture.sync();
    assert!(fixture.state.sessions.is_empty());
    assert!(fixture.state.layer_surfaces.is_empty());
    assert_eq!(fixture.count("surface-destroyed"), 2);
    assert_eq!(fixture.count("layer-destroyed"), 2);
}

#[test]
fn protocol_output_removal_and_shutdown_release_owned_bindings() {
    for named in [true, false] {
        let mut fixture = Fixture::new(named);
        let name = fixture.name();
        fixture.apply(&name);
        let global = *fixture.state.pending_outputs.keys().next().unwrap();
        fixture.hotplug(false);
        fixture.state.remove_output(global);
        fixture.sync();
        assert!(fixture.state.sessions.is_empty());
        assert!(fixture.state.layer_surfaces.is_empty());
        assert!(fixture.state.outputs.ready_outputs().next().is_none());
        assert_eq!(fixture.count("output-released"), 1);
        assert_eq!(fixture.count("surface-destroyed"), 1);
        assert_eq!(fixture.count("layer-destroyed"), 1);
        assert!(fixture
            .events
            .try_iter()
            .any(|event| matches!(event, EngineEvent::OutputRemoved(output) if output == name)));
        fixture.hotplug(true);
        let reconnected = fixture.name();
        if named {
            assert_eq!(reconnected, name);
        } else {
            assert!(reconnected.starts_with("output-"));
        }
        fixture.apply(&reconnected);
        fixture.state.cleanup();
        fixture.sync();
        assert_eq!(fixture.count("output-released"), 2);
        assert_eq!(fixture.count("surface-destroyed"), 2);
        let log = fixture.log.lock().unwrap();
        assert!(
            log.iter().position(|&x| x == "layer-destroyed")
                < log.iter().position(|&x| x == "surface-destroyed")
        );
        assert!(
            log.iter().position(|&x| x == "surface-destroyed")
                < log.iter().position(|&x| x == "output-released")
        );
    }
    let mut fixture = Fixture::new(true);
    let name = fixture.name();
    fixture.apply(&name);
    fixture.state.cleanup();
    fixture.state.cleanup();
    fixture.sync();
    assert_eq!(fixture.count("surface-destroyed"), 1);
    assert_eq!(fixture.count("output-released"), 1);
}

#[test]
fn protocol_repeated_lifetimes_return_to_baseline() {
    for _ in 0..20 {
        let mut fixture = Fixture::new(true);
        let name = fixture.name();
        for _ in 0..5 {
            fixture.apply(&name);
            fixture.apply(&name);
            fixture.state.remove_wallpaper(&name);
            fixture.state.remove_wallpaper(&name);
            fixture.sync();
            assert!(fixture.state.sessions.is_empty());
            assert!(fixture.state.layer_surfaces.is_empty());
            assert_eq!(
                fixture.count("surface-created"),
                fixture.count("surface-destroyed")
            );
            assert_eq!(
                fixture.count("layer-created"),
                fixture.count("layer-destroyed")
            );
            fixture.events.try_iter().for_each(drop);
        }
        fixture.state.cleanup();
        fixture.sync();
        assert_eq!(fixture.count("output-released"), 1);
    }
}
