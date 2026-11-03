#![cfg(all(feature = "native", windows, target_has_atomic = "64"))]
use qsl::native::{NativeListener, NativeRegion, NativeSocket, SharedRegion};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
fn name() -> String {
    static ID: AtomicU64 = AtomicU64::new(0);
    format!(
        r"Local\qsl-test-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        ID.fetch_add(1, Ordering::Relaxed)
    )
}
fn deadline(socket: &NativeSocket) {
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
}
#[test]
fn local_region_alignment_capacity_reset_and_value_drop() {
    assert!(NativeRegion::new(0).is_err());
    let mut region = NativeRegion::new(65536).unwrap();
    region.prefault();
    assert_eq!(region.capacity(), 65536);
    let first;
    {
        let mut a = region.buffer(15, 1).unwrap();
        first = a.as_mut_ptr();
        a.write_from_slice(b"stable").unwrap();
        let mut b = region.buffer(4097, 4096).unwrap();
        assert_eq!(b.as_mut_ptr() as usize % 4096, 0);
        b.write_from_slice(b"second").unwrap();
        assert_eq!(&*a, b"stable");
        assert!(region.buffer(usize::MAX, 64).is_err());
        assert!(region.buffer(1, 3).is_err());
        assert!(region.buffer(65536, 1).is_err());
        assert_eq!(&*b, b"second");
        let dropped = Arc::new(AtomicU64::new(0));
        struct Tracked(Arc<AtomicU64>);
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let value = region
            .slot::<Tracked>()
            .unwrap()
            .write(Tracked(dropped.clone()));
        drop(value);
        assert_eq!(dropped.load(Ordering::Relaxed), 1);
    }
    region.reset();
    assert_eq!(region.used(), 0);
    assert_eq!(region.buffer(15, 1).unwrap().as_mut_ptr(), first);
}
#[test]
fn shared_pending_aborted_immutable_and_named_lifetime() {
    let name = name();
    let region = SharedRegion::create(&name, 32768).unwrap();
    assert_eq!(
        SharedRegion::create(&name, 32768).err().unwrap().kind(),
        std::io::ErrorKind::AlreadyExists
    );
    let second = unsafe { SharedRegion::open(&name) }.unwrap();
    assert_eq!(second.capacity(), 32768);
    assert!(region.reserve(1, 3).is_err());
    assert!(region.reserve(32768, 64).is_err());
    let mut pending = region.reserve(100, 64).unwrap();
    let offset = pending.offset();
    assert_eq!(pending.as_mut_ptr() as usize % 64, 0);
    assert!(unsafe { second.read_published(offset) }.unwrap().is_none());
    pending.write_from_slice("Привет, QSL".as_bytes()).unwrap();
    assert!(unsafe { second.read_published(offset) }.unwrap().is_none());
    let published = pending.publish();
    let read = unsafe { second.read_published(offset) }.unwrap().unwrap();
    assert_eq!(&*read, "Привет, QSL".as_bytes());
    let aborted = region.reserve(4, 1).unwrap();
    let dead = aborted.offset();
    drop(aborted);
    assert_eq!(
        unsafe { second.read_published(dead) }.err().unwrap().kind(),
        std::io::ErrorKind::BrokenPipe
    );
    drop(region);
    drop(second);
    drop(published);
    // The immutable view keeps the mapping alive.
    assert!(SharedRegion::create(&name, 100).is_err());
    assert_eq!(&*read, "Привет, QSL".as_bytes());
    drop(read);
    assert!(SharedRegion::create(&name, 100).is_ok());
}
#[test]
fn shared_concurrent_allocations_do_not_overlap() {
    let region = SharedRegion::create(&name(), 1024 * 1024).unwrap();
    let threads = (0..4u8)
        .map(|thread| {
            let region = region.clone();
            std::thread::spawn(move || {
                let mut result = Vec::new();
                for index in 0..128u8 {
                    let mut block = region.reserve(257, 64).unwrap();
                    let mut expected = [thread; 257];
                    expected[0] = index;
                    block.write_from_slice(&expected).unwrap();
                    let bytes = block.publish();
                    result.push((bytes.offset(), expected));
                }
                result
            })
        })
        .collect::<Vec<_>>();
    let mut offsets = std::collections::HashSet::new();
    for thread in threads {
        for (offset, expected) in thread.join().unwrap() {
            assert!(offsets.insert(offset));
            assert_eq!(
                &*unsafe { region.read_published(offset) }.unwrap().unwrap(),
                &expected
            );
        }
    }
    assert_eq!(offsets.len(), 512);
}
#[test]
fn ipc_child() {
    let Ok(name) = std::env::var("QSL_TEST_IPC_NAME") else {
        return;
    };
    let offset = std::env::var("QSL_TEST_IPC_OFFSET")
        .unwrap()
        .parse()
        .unwrap();
    let region = unsafe { SharedRegion::open(&name) }.unwrap();
    let request = unsafe { region.read_published(offset) }.unwrap().unwrap();
    assert_eq!(&*request, "Привет из родителя 🦀".as_bytes());
    let mut response = region.reserve(256, 64).unwrap();
    response
        .write_from_slice("Ответ отдельного процесса 🦀".as_bytes())
        .unwrap();
    let response = response.publish();
    println!("QSL_IPC_OFFSET={}", response.offset());
}
#[test]
fn shared_real_child_process_roundtrip() {
    let name = name();
    let region = SharedRegion::create(&name, 65536).unwrap();
    let mut request = region.reserve(256, 64).unwrap();
    request
        .write_from_slice("Привет из родителя 🦀".as_bytes())
        .unwrap();
    let request = request.publish();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "ipc_child", "--nocapture"])
        .env("QSL_TEST_IPC_NAME", &name)
        .env("QSL_TEST_IPC_OFFSET", request.offset().to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= until {
            let _ = child.kill();
            let _ = child.wait();
            panic!("IPC child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let line = stdout
        .lines()
        .find(|s| s.starts_with("QSL_IPC_OFFSET="))
        .expect(&stdout);
    let offset = line.trim_start_matches("QSL_IPC_OFFSET=").parse().unwrap();
    let response = unsafe { region.read_published(offset) }.unwrap().unwrap();
    assert_eq!(&*response, "Ответ отдельного процесса 🦀".as_bytes());
}
fn socket_roundtrip(address: SocketAddr) {
    let listener = NativeListener::bind(address, 2 * 1024 * 1024, 16).unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let mut socket = listener.accept().unwrap();
        deadline(&socket);
        let region = NativeRegion::new(2 * 1024 * 1024).unwrap();
        let mut buffer = region.buffer(1024 * 1024, 64).unwrap();
        for len in [0, 1, 1024, 1024 * 1024] {
            assert_eq!(socket.receive_into(&mut buffer).unwrap(), len);
            assert!(buffer.iter().all(|&b| b == 0xe7));
            socket.send_frame(&buffer).unwrap();
        }
    });
    let mut socket = NativeSocket::connect(address, 2 * 1024 * 1024).unwrap();
    deadline(&socket);
    let region = NativeRegion::new(2 * 1024 * 1024).unwrap();
    let mut buffer = region.buffer(1024 * 1024, 64).unwrap();
    for len in [0, 1, 1024, 1024 * 1024] {
        socket.send_frame(&vec![0xe7; len]).unwrap();
        assert_eq!(socket.receive_into(&mut buffer).unwrap(), len);
        assert!(buffer.iter().all(|&b| b == 0xe7));
    }
    server.join().unwrap();
}
#[test]
fn native_tcp_ipv4_roundtrip() {
    socket_roundtrip("127.0.0.1:0".parse().unwrap());
}
#[test]
fn native_tcp_ipv6_roundtrip() {
    let listener = std::net::TcpListener::bind("[::1]:0").unwrap();
    match TcpStream::connect(listener.local_addr().unwrap()) {
        Ok(_) => socket_roundtrip("[::1]:0".parse().unwrap()),
        Err(error) if error.raw_os_error() == Some(10013) => {
            eprintln!("SKIP IPv6 runtime check: std TCP also denied by host (WSAEACCES 10013)");
        }
        Err(error) => panic!("IPv6 baseline failed: {error}"),
    }
}

#[test]
fn native_tcp_fragmented_frame_directly_to_shared_memory() {
    let listener = NativeListener::bind("127.0.0.1:0".parse().unwrap(), 1024, 8).unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let mut socket = listener.accept().unwrap();
        deadline(&socket);
        let shared = SharedRegion::create(&name(), 4096).unwrap();
        let mut buffer = shared.reserve(1024, 64).unwrap();
        assert_eq!(socket.receive_shared(&mut buffer).unwrap(), 17);
        assert_eq!(
            qsl::Utf8Guard::try_from_bytes(&buffer).unwrap().as_str(),
            "fragmented binary"
        );
        let ready = buffer.publish();
        assert_eq!(&*ready, b"fragmented binary");
        socket.send_frame(&ready).unwrap();
    });
    let mut socket = TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    for byte in 17u32.to_le_bytes().into_iter().chain(*b"fragmented binary") {
        socket.write_all(&[byte]).unwrap();
    }
    let mut reply = [0; 21];
    socket.read_exact(&mut reply).unwrap();
    assert_eq!(&reply[..4], &17u32.to_le_bytes());
    assert_eq!(&reply[4..], b"fragmented binary");
    server.join().unwrap();
}
#[test]
fn native_tcp_rejects_oversize_and_truncated_frames_without_exposing_stale_data() {
    for truncated in [false, true] {
        let listener = NativeListener::bind("127.0.0.1:0".parse().unwrap(), 8, 8).unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let mut socket = listener.accept().unwrap();
            deadline(&socket);
            let region = NativeRegion::new(4096).unwrap();
            let mut buffer = region.buffer(8, 1).unwrap();
            buffer.write_from_slice(b"old").unwrap();
            let kind = socket.receive_into(&mut buffer).err().unwrap().kind();
            assert_eq!(
                kind,
                if truncated {
                    std::io::ErrorKind::UnexpectedEof
                } else {
                    std::io::ErrorKind::InvalidData
                }
            );
            assert!(buffer.is_empty());
            assert_eq!(
                socket.receive_into(&mut buffer).err().unwrap().kind(),
                std::io::ErrorKind::BrokenPipe
            );
        });
        let mut socket = TcpStream::connect(address).unwrap();
        socket
            .write_all(&(if truncated { 8u32 } else { 9u32 }).to_le_bytes())
            .unwrap();
        socket.write_all(b"cut").unwrap();
        drop(socket);
        server.join().unwrap();
    }
}

#[test]
fn shared_release_acquire_publishes_complete_payload_without_thread_join() {
    let region = SharedRegion::create(&name(), 128 * 1024).unwrap();
    let mut write = region.reserve(65536, 64).unwrap();
    let offset = write.offset();
    assert!(unsafe { region.read_published(offset) }.unwrap().is_none());
    let start = Arc::new(std::sync::Barrier::new(2));
    let writer_start = start.clone();
    let writer = std::thread::spawn(move || {
        writer_start.wait();
        for (index, slot) in write.spare_capacity_mut().iter_mut().enumerate() {
            slot.write((index % 251) as u8);
        }
        unsafe {
            write.set_len(65536);
        }
        write.publish()
    });
    start.wait();
    let deadline = Instant::now() + Duration::from_secs(5);
    let read = loop {
        if let Some(bytes) = unsafe { region.read_published(offset) }.unwrap() {
            break bytes;
        }
        assert!(Instant::now() < deadline, "publication timed out");
        std::thread::yield_now();
    };
    assert_eq!(read.len(), 65536);
    assert!(read.iter().enumerate().all(|(i, &b)| b == (i % 251) as u8));
    drop(writer.join().unwrap());
}
