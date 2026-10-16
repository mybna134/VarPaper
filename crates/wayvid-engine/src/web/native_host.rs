//! Launch the installed CEF runtime in its platform sandbox.
use super::*;
use std::os::fd::AsRawFd;
use std::process::Stdio;

impl Host {
    pub(super) fn shared() -> Result<Rc<RefCell<Self>>> {
        HOST.with(|slot| {
            if let Some(host) = slot.borrow().upgrade().filter(|host| !host.borrow().dead) {
                return Ok(host);
            }
            let host = Rc::new(RefCell::new(Self::spawn()?));
            *slot.borrow_mut() = Rc::downgrade(&host);
            Ok(host)
        })
    }
    fn spawn() -> Result<Self> {
        let runtime = runtime_path()?
            .canonicalize()
            .context("Install Web wallpaper support in Settings")?;
        ensure!(
            runtime.join("libcef.so").is_file(),
            "Chromium runtime library is missing"
        );
        let (socket, child_socket) = UnixStream::pair()?;
        socket.set_read_timeout(Some(Duration::from_secs(8)))?;
        socket.set_write_timeout(Some(Duration::from_secs(8)))?;
        let fd = child_socket.as_raw_fd();
        let host = validated_host_path()?;
        // Flatpak blocks Chromium's nested namespace/setuid sandbox. Zypak
        // launches its children through the Flatpak portal instead.
        let mut command = if Path::new("/.flatpak-info").is_file() {
            let mut command = Command::new("/app/bin/zypak-wrapper");
            command.arg(host);
            command.env("ZYPAK_CEF_LIBRARY_PATH", runtime.join("libcef.so"));
            command
        } else {
            Command::new(host)
        };
        command
            .arg(format!("--ipc-fd={fd}"))
            .env("VARPAPER_CEF_RUNTIME", &runtime)
            .env("LD_LIBRARY_PATH", &runtime)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        unsafe {
            command.pre_exec(move || {
                if libc::setsid() < 0 || libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command
            .spawn()
            .context("Cannot launch the Web wallpaper host")?;
        drop(child_socket);
        #[cfg(test)]
        track(0, true);
        Ok(Self {
            socket,
            child,
            request: 0,
            next_id: 0,
            dead: false,
        })
    }
}
