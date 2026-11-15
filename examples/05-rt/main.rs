use qsl_examples::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = rt_numbers(3);
    let expected = numbers(3).serialize_checked()?;
    let mut leases = Vec::new();
    for _ in 0..16 {
        leases.push(value.serialize()?);
    }
    assert_eq!(&leases[0][..], expected);
    assert!(matches!(
        value.serialize(),
        Err(qsl::QslError::PoolExhausted)
    ));
    println!("16 live RT leases; the 17th returns PoolExhausted without waiting");
    drop(leases.pop());
    let lease = value.serialize()?;
    assert_eq!(lease.as_slice(), expected);
    println!("Dropping a lease makes its region immediately available again");
    Ok(())
}
