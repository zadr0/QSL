use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = rt_numbers(3);
    let expected = numbers(3).serialize_checked()?;
    let mut leases = Vec::new();
    for _ in 0..16 {
        leases.push(value.serialize()?);
    }
    assert_eq!(&leases[0][..], expected);
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let lease = value.serialize().unwrap();
        tx.send(lease.as_slice().to_vec()).unwrap();
    });
    assert!(rx
        .recv_timeout(std::time::Duration::from_millis(20))
        .is_err());
    println!("16 live RT leases; the 17th waits for release");
    drop(leases.pop());
    assert_eq!(
        rx.recv_timeout(std::time::Duration::from_secs(2))?,
        expected
    );
    worker.join().unwrap();
    drop(leases);
    Ok(())
}
