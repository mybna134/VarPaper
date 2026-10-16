//! Minimal dynamically loaded PulseAudio client. All pointers and callbacks
//! belong to the monitor worker; no libpulse ABI objects cross threads.
use super::{Capture, Input, SAMPLE_RATE};
use anyhow::{bail, ensure, Context, Result};
use libloading::Library;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::time::{Duration, Instant};

type Handle = *mut c_void;
#[repr(C)]
struct SampleSpec {
    format: c_int,
    rate: u32,
    channels: u8,
}
#[repr(C)]
struct ChannelMap {
    channels: u8,
    map: [c_int; 32],
}
#[repr(C)]
struct Volume {
    channels: u8,
    values: [u32; 32],
}
// Only the documented, stable prefix is accessed. Later ABI fields can grow.
#[repr(C)]
struct ServerInfo {
    user: *const c_char,
    host: *const c_char,
    version: *const c_char,
    name: *const c_char,
    sample: SampleSpec,
    sink: *const c_char,
}
#[repr(C)]
struct SinkInfo {
    name: *const c_char,
    index: u32,
    description: *const c_char,
    sample: SampleSpec,
    map: ChannelMap,
    owner: u32,
    volume: Volume,
    mute: c_int,
    monitor_index: u32,
    monitor_name: *const c_char,
}
#[repr(C)]
struct BufferAttr {
    maxlength: u32,
    tlength: u32,
    prebuf: u32,
    minreq: u32,
    fragsize: u32,
}
type ServerCallback = unsafe extern "C" fn(Handle, *const ServerInfo, Handle);
type SinkCallback = unsafe extern "C" fn(Handle, *const SinkInfo, c_int, Handle);
type SubscribeCallback = unsafe extern "C" fn(Handle, c_int, u32, Handle);
macro_rules! api {
    ($($field:ident: $ty:ty = $name:literal),* $(,)?) => {
        struct Api { _library: Library, $($field: $ty,)* }
        impl Api {
            fn load() -> Result<Self> {
                unsafe {
                    let library=Library::new("libpulse.so.0").context("Cannot load libpulse.so.0")?;
                    Ok(Self { $($field: *library.get(concat!($name,"\0").as_bytes())?,)* _library: library })
                }
            }
        }
    }
}
api! {
    mainloop_new: unsafe extern "C" fn()->Handle = "pa_mainloop_new",
    mainloop_free: unsafe extern "C" fn(Handle) = "pa_mainloop_free",
    mainloop_api: unsafe extern "C" fn(Handle)->Handle = "pa_mainloop_get_api",
    iterate: unsafe extern "C" fn(Handle,c_int,*mut c_int)->c_int = "pa_mainloop_iterate",
    context_new: unsafe extern "C" fn(Handle,*const c_char)->Handle = "pa_context_new",
    context_connect: unsafe extern "C" fn(Handle,*const c_char,c_int,Handle)->c_int = "pa_context_connect",
    context_state: unsafe extern "C" fn(Handle)->c_int = "pa_context_get_state",
    context_disconnect: unsafe extern "C" fn(Handle) = "pa_context_disconnect",
    context_unref: unsafe extern "C" fn(Handle) = "pa_context_unref",
    server_info: unsafe extern "C" fn(Handle,ServerCallback,Handle)->Handle = "pa_context_get_server_info",
    sink_info: unsafe extern "C" fn(Handle,*const c_char,SinkCallback,Handle)->Handle = "pa_context_get_sink_info_by_name",
    subscribe_callback: unsafe extern "C" fn(Handle,Option<SubscribeCallback>,Handle) = "pa_context_set_subscribe_callback",
    subscribe: unsafe extern "C" fn(Handle,c_int,Handle,Handle)->Handle = "pa_context_subscribe",
    operation_unref: unsafe extern "C" fn(Handle) = "pa_operation_unref",
    stream_new: unsafe extern "C" fn(Handle,*const c_char,*const SampleSpec,Handle)->Handle = "pa_stream_new",
    record: unsafe extern "C" fn(Handle,*const c_char,*const BufferAttr,c_int)->c_int = "pa_stream_connect_record",
    stream_state: unsafe extern "C" fn(Handle)->c_int = "pa_stream_get_state",
    stream_disconnect: unsafe extern "C" fn(Handle)->c_int = "pa_stream_disconnect",
    stream_unref: unsafe extern "C" fn(Handle) = "pa_stream_unref",
    peek: unsafe extern "C" fn(Handle,*mut *const c_void,*mut usize)->c_int = "pa_stream_peek",
    drop_buffer: unsafe extern "C" fn(Handle)->c_int = "pa_stream_drop",
}
#[derive(Default)]
struct Discovery {
    changed: bool,
    server_pending: bool,
    sink_pending: bool,
    sink: Option<CString>,
    monitor: Option<CString>,
    error: Option<&'static str>,
}
unsafe extern "C" fn server_info(_: Handle, info: *const ServerInfo, data: Handle) {
    let discovery = &mut *data.cast::<Discovery>();
    discovery.server_pending = false;
    if info.is_null() || (*info).sink.is_null() {
        discovery.error = Some("Default playback sink is unavailable");
        return;
    }
    discovery.sink = Some(CStr::from_ptr((*info).sink).to_owned());
}
unsafe extern "C" fn sink_info(_: Handle, info: *const SinkInfo, end: c_int, data: Handle) {
    let discovery = &mut *data.cast::<Discovery>();
    if end != 0 {
        discovery.sink_pending = false;
        if end < 0 {
            discovery.error = Some("Playback sink lookup failed");
        }
        return;
    }
    if info.is_null() || (*info).monitor_index == u32::MAX || (*info).monitor_name.is_null() {
        discovery.error = Some("Playback sink has no monitor source");
        return;
    }
    // Use the sink's declared monitor, never default_source_name (microphone).
    discovery.monitor = Some(CStr::from_ptr((*info).monitor_name).to_owned());
}
unsafe extern "C" fn subscription(_: Handle, _: c_int, _: u32, data: Handle) {
    (*data.cast::<Discovery>()).changed = true;
}
pub(super) struct Monitor {
    api: Api,
    mainloop: Handle,
    context: Handle,
    stream: Handle,
    discovery: Box<Discovery>,
    subscribed: bool,
    active: Option<CString>,
    requested: Instant,
    stream_started: Instant,
    last_refresh: Instant,
}
impl Monitor {
    pub(super) fn new() -> Result<Self> {
        let api = Api::load()?;
        let mainloop = unsafe { (api.mainloop_new)() };
        ensure!(
            !mainloop.is_null(),
            "Cannot create playback monitor mainloop"
        );
        let mut monitor = Self {
            api,
            mainloop,
            context: ptr::null_mut(),
            stream: ptr::null_mut(),
            discovery: Box::default(),
            subscribed: false,
            active: None,
            requested: Instant::now(),
            stream_started: Instant::now(),
            last_refresh: Instant::now(),
        };
        unsafe {
            monitor.context = (monitor.api.context_new)(
                (monitor.api.mainloop_api)(mainloop),
                c"VarPaper playback visualization".as_ptr(),
            );
            ensure!(
                !monitor.context.is_null(),
                "Cannot create playback monitor context"
            );
            // Do not start a daemon when monitoring is unavailable.
            ensure!(
                (monitor.api.context_connect)(monitor.context, ptr::null(), 1, ptr::null_mut())
                    == 0,
                "Cannot connect to playback server"
            );
        }
        Ok(monitor)
    }
    fn operation(&self, operation: Handle) -> Result<()> {
        ensure!(!operation.is_null(), "Playback monitor operation failed");
        unsafe { (self.api.operation_unref)(operation) };
        Ok(())
    }
    fn close_stream(&mut self) {
        if !self.stream.is_null() {
            unsafe {
                (self.api.stream_disconnect)(self.stream);
                (self.api.stream_unref)(self.stream);
            }
            self.stream = ptr::null_mut();
        }
    }
}
impl Capture for Monitor {
    fn poll(&mut self) -> Result<Option<Input>> {
        unsafe {
            ensure!(
                (self.api.iterate)(self.mainloop, 0, ptr::null_mut()) >= 0,
                "Playback mainloop failed"
            );
            match (self.api.context_state)(self.context) {
                4 => {}
                5 | 6 => bail!("Playback server disconnected"),
                _ => {
                    ensure!(
                        self.requested.elapsed() < Duration::from_secs(3),
                        "Playback server connection timed out"
                    );
                    return Ok(None);
                }
            }
            let data = (&mut *self.discovery as *mut Discovery).cast();
            if !self.subscribed {
                (self.api.subscribe_callback)(self.context, Some(subscription), data);
                self.operation((self.api.subscribe)(
                    self.context,
                    0x81,
                    ptr::null_mut(),
                    ptr::null_mut(),
                ))?;
                self.subscribed = true;
                self.discovery.changed = true;
            }
            if let Some(error) = self.discovery.error.take() {
                bail!("{error}");
            }
            if self.discovery.server_pending || self.discovery.sink_pending {
                ensure!(
                    self.requested.elapsed() < Duration::from_secs(3),
                    "Playback monitor discovery timed out"
                );
            } else if self.discovery.changed || self.last_refresh.elapsed() > Duration::from_secs(1)
            {
                self.discovery.changed = false;
                self.discovery.server_pending = true;
                self.requested = Instant::now();
                self.last_refresh = Instant::now();
                self.operation((self.api.server_info)(self.context, server_info, data))?;
            }
            if let Some(sink) = self.discovery.sink.take() {
                self.discovery.sink_pending = true;
                self.requested = Instant::now();
                self.operation((self.api.sink_info)(
                    self.context,
                    sink.as_ptr(),
                    sink_info,
                    data,
                ))?;
            }
            if let Some(name) = self.discovery.monitor.take() {
                if self.active.as_ref() != Some(&name) {
                    self.close_stream();
                    let spec = SampleSpec {
                        format: if cfg!(target_endian = "little") { 5 } else { 6 },
                        rate: SAMPLE_RATE as u32,
                        channels: 2,
                    };
                    self.stream = (self.api.stream_new)(
                        self.context,
                        c"System playback spectrum".as_ptr(),
                        &spec,
                        ptr::null_mut(),
                    );
                    ensure!(
                        !self.stream.is_null(),
                        "Cannot create playback monitor stream"
                    );
                    let buffers = BufferAttr {
                        maxlength: 128 * 1024,
                        tlength: u32::MAX,
                        prebuf: u32::MAX,
                        minreq: u32::MAX,
                        fragsize: 2048 * 8,
                    };
                    // DONT_MOVE prevents automatic fallback to an unrelated input.
                    ensure!(
                        (self.api.record)(self.stream, name.as_ptr(), &buffers, 0x2200) == 0,
                        "Cannot connect playback monitor stream"
                    );
                    self.active = Some(name.clone());
                    self.stream_started = Instant::now();
                    return Ok(Some(Input::Monitor(name.to_string_lossy().into_owned())));
                }
            }
            if self.stream.is_null() {
                return Ok(None);
            }
            match (self.api.stream_state)(self.stream) {
                2 => {}
                3 | 4 => bail!("Playback monitor stream disconnected"),
                _ => {
                    ensure!(
                        self.stream_started.elapsed() < Duration::from_secs(3),
                        "Playback monitor stream timed out"
                    );
                    return Ok(None);
                }
            }
            let mut bytes = ptr::null();
            let mut length = 0;
            ensure!(
                (self.api.peek)(self.stream, &mut bytes, &mut length) == 0,
                "Playback monitor read failed"
            );
            if length == 0 {
                return Ok(None);
            }
            ensure!(
                length <= 128 * 1024 && length % 8 == 0,
                "Invalid playback monitor PCM buffer"
            );
            let samples = if bytes.is_null() {
                vec![0.0; length / 4]
            } else {
                // PulseAudio owns the buffer until drop; copy bounded bytes,
                // avoiding assumptions about its float pointer alignment.
                std::slice::from_raw_parts(bytes.cast::<u8>(), length)
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|b| f32::from_ne_bytes(*b))
                    .collect()
            };
            ensure!(
                (self.api.drop_buffer)(self.stream) == 0,
                "Playback monitor buffer release failed"
            );
            Ok(Some(Input::Samples(samples)))
        }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.close_stream();
        unsafe {
            if !self.context.is_null() {
                (self.api.subscribe_callback)(self.context, None, ptr::null_mut());
                (self.api.context_disconnect)(self.context);
                (self.api.context_unref)(self.context);
            }
            (self.api.mainloop_free)(self.mainloop);
        }
        // Discovery and library outlive disconnect/unref and all callbacks.
    }
}
