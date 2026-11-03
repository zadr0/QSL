use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = rt_numbers(3);
    let expected = numbers(3).serialize_checked()?;
    let mut leases = Vec::new();
    for _ in 0..16 {
        leases.push(value.serialize()?);
    }
    assert!(matches!(
        value.serialize(),
        Err(qsl::QslError::PoolExhausted)
    ));
    assert_eq!(&leases[0][..], expected);
    println!(
        "16 live RT leases, {} B each; the 17th returns PoolExhausted",
        expected.len()
    );
    drop(leases);
    let lease = value.serialize()?;
    println!("after release: slot reused, {} B", lease.len());
    Ok(())
}
