use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let numbers = packed_numbers(3);
    let scalar = numbers.serialize_checked()?;
    let text = WholeText {
        text: "A".repeat(1024),
    };
    let raw = text.serialize_checked()?;
    let mut scratch = [0; 8192];
    let mut wire = [0; 8208];
    let mut restoration = [0; 8192];
    let mut output = std::mem::MaybeUninit::uninit();
    let frame = text.serialize_qps_to(&mut scratch, &mut wire)?;
    println!("scalar QPS: 64 useful B -> {} wire B", scalar.len());
    println!(
        "whole payload: {} inner B -> {} QPB3 B ({:?})",
        raw.len(),
        frame.as_slice().len(),
        frame.mode()
    );
    let restored =
        WholeText::deserialize_qps_into(frame.as_slice(), &mut restoration, &mut output)?;
    assert_eq!(restored.text, text.text);
    unsafe { output.assume_init_drop() };
    Ok(())
}
