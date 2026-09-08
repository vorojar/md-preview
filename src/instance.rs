//! Local launch forwarding. The OS file lock owns the endpoint lifetime; a crashed
//! process releases it automatically, so stale endpoint data cannot block restart.
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::hash::{BuildHasher, Hasher};
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Launch {
    pub paths: Vec<PathBuf>,
    pub edit: bool,
}

#[derive(Serialize, Deserialize)]
struct Endpoint {
    port: u16,
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Request {
    token: String,
    launch: Launch,
}

pub struct Primary {
    _lock: File,
    listener: TcpListener,
    endpoint: Endpoint,
}

pub fn acquire(directory: &Path, launch: &Launch) -> io::Result<Option<Primary>> {
    std::fs::create_dir_all(directory)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let lock = options.open(directory.join("instance.lock"))?;
    let endpoint_path = directory.join("instance.json");
    match lock.try_lock() {
        Ok(()) => {
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
            let token = (0..2)
                .map(|_| {
                    format!(
                        "{:016x}",
                        std::collections::hash_map::RandomState::new()
                            .build_hasher()
                            .finish()
                    )
                })
                .collect::<String>();
            let endpoint = Endpoint {
                port: listener.local_addr()?.port(),
                token,
            };
            // Windows file locks can exclude reads, so publish outside the locked file.
            let mut endpoint_file = options.truncate(true).open(&endpoint_path)?;
            serde_json::to_writer(&mut endpoint_file, &endpoint)?;
            endpoint_file.flush()?;
            Ok(Some(Primary {
                _lock: lock,
                listener,
                endpoint,
            }))
        }
        Err(std::fs::TryLockError::WouldBlock) => {
            // The winner may still be publishing its endpoint after taking the lock.
            let mut last_error = io::Error::other("Running instance is not ready");
            for _ in 0..30 {
                let endpoint = File::open(&endpoint_path).and_then(|file| {
                    serde_json::from_reader::<_, Endpoint>(file).map_err(io::Error::from)
                });
                match endpoint.and_then(|endpoint| forward(&endpoint, launch)) {
                    Ok(()) => return Ok(None),
                    Err(error) => last_error = error,
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(last_error)
        }
        Err(std::fs::TryLockError::Error(error)) => Err(error),
    }
}

fn forward(endpoint: &Endpoint, launch: &Launch) -> io::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(1))?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    serde_json::to_writer(
        &mut stream,
        &serde_json::json!({"token": endpoint.token, "launch": launch}),
    )?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut ack = [0];
    stream.read_exact(&mut ack)?;
    if ack != [1] {
        return Err(io::Error::other("Launch was not accepted"));
    }
    Ok(())
}

impl Primary {
    pub fn listen(self, handler: impl Fn(Launch) -> bool + Send + 'static) {
        std::thread::spawn(move || {
            // Keep the lock alive alongside the listener.
            let _lock = self._lock;
            for stream in self.listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
                let request =
                    serde_json::from_reader::<_, Request>((&mut stream).take(1024 * 1024));
                if let Ok(request) = request {
                    if request.token == self.endpoint.token && handler(request.launch) {
                        let _ = stream.write_all(&[1]);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secondary_launch_forwards_paths_and_edit_flag() {
        let directory = std::env::temp_dir().join(format!("mdp-instance-{}", std::process::id()));
        let launch = Launch {
            paths: vec![directory.join("含 空格.md")],
            edit: true,
        };
        let primary = acquire(&directory, &launch).unwrap().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        primary.listen(move |launch| tx.send(launch).is_ok());
        assert!(acquire(&directory, &launch).unwrap().is_none());
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap(), launch);
        let empty = Launch {
            paths: vec![],
            edit: false,
        };
        assert!(acquire(&directory, &empty).unwrap().is_none());
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap(), empty);
    }

    #[test]
    fn rejects_foreign_tokens_and_accepts_concurrent_launches() {
        let directory =
            std::env::temp_dir().join(format!("mdp-instance-concurrent-{}", std::process::id()));
        let initial = Launch {
            paths: vec![],
            edit: false,
        };
        let primary = acquire(&directory, &initial).unwrap().unwrap();
        let foreign = Endpoint {
            port: primary.endpoint.port,
            token: "wrong-token".to_string(),
        };
        let (tx, rx) = std::sync::mpsc::channel();
        primary.listen(move |launch| tx.send(launch).is_ok());
        assert!(forward(&foreign, &initial).is_err());
        assert!(rx.try_recv().is_err());
        let threads = (0..8)
            .map(|index| {
                let directory = directory.clone();
                std::thread::spawn(move || {
                    let launch = Launch {
                        paths: vec![directory.join(format!("{index}.md"))],
                        edit: false,
                    };
                    assert!(acquire(&directory, &launch).unwrap().is_none());
                })
            })
            .collect::<Vec<_>>();
        for thread in threads {
            thread.join().unwrap();
        }
        let mut paths = (0..8)
            .map(|_| rx.recv_timeout(Duration::from_secs(2)).unwrap().paths[0].clone())
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), 8);
    }

    #[test]
    fn released_lock_allows_restart_with_stale_endpoint() {
        let directory =
            std::env::temp_dir().join(format!("mdp-instance-restart-{}", std::process::id()));
        let launch = Launch {
            paths: vec![],
            edit: false,
        };
        let first = acquire(&directory, &launch).unwrap().unwrap();
        drop(first);
        assert!(acquire(&directory, &launch).unwrap().is_some());
    }
}
