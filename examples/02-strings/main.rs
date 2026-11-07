use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = texts().remove(0);
    let plain = OwnedText { text: text.clone() };
    let bytes = plain.serialize_checked()?;
    let borrowed = BorrowedText::deserialize_checked(&bytes)?;
    let guard = OwnedReuseText::deserialize_checked(&bytes)?;
    let inline = InlineText { text: text.clone() }.serialize_checked()?;
    let compressed = CompressedText { text: text.clone() }.serialize_checked()?;
    println!(
        "{} UTF-8 characters = {} bytes; String/borrow/COPY frame={} B; inline={} B; LZ={} B",
        text.chars().count(),
        text.len(),
        bytes.len(),
        inline.len(),
        compressed.len()
    );
    println!("The phrase selected LZ raw fallback: the 2-byte difference is a length header, not compression.");
    let uniform = uniform_texts().remove(0);
    let packed = CompressedText {
        text: uniform.clone(),
    }
    .serialize_checked()?;
    println!(
        "Uniform UTF-8 control: {} useful B -> {} LZ wire B",
        uniform.len(),
        packed.len()
    );
    println!(
        "borrowed={} chars, COPY String={} bytes; fixed string Rust type: [u8; 4]",
        borrowed.text.chars().count(),
        guard.text.len()
    );
    assert_eq!(
        CompressedText::deserialize_checked(&compressed)?
            .text
            .as_str(),
        text
    );
    let fixed = FixedText { text: *b"rust" };
    assert_eq!(fixed.serialize_checked()?, b"rust");
    Ok(())
}
