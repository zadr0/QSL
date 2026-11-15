use qsl_examples::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = texts().remove(0);
    let plain = OwnedText { text: text.clone() };
    let bytes: Vec<u8> = plain.serialize_checked()?;
    let borrowed = BorrowedText::deserialize_checked(&bytes)?;
    assert_eq!(borrowed.text, text);

    // Reuse an initialized String. Validate before calling the unchecked reader.
    let mut reuse = OwnedReuseText {
        text: String::with_capacity(4096),
    };
    OwnedReuseText::validate(&bytes)?;
    let pointer = reuse.text.as_ptr();
    unsafe { OwnedReuseText::deserialize_into(bytes.as_ptr(), bytes.len(), &mut reuse) };
    assert_eq!(reuse.text, text);
    assert_eq!(reuse.text.as_ptr(), pointer);

    // The 55-byte bounded container fits STACK_LIMIT=256: no Box or String.
    let short = "Рад, так Рад!";
    let small: qsl::InlineString<55> = qsl::InlineString::try_from_str(short)?;
    let inline: qsl::StackOut<56> = InlineText { text: small }.serialize_checked()?;
    assert_eq!(
        InlineText::deserialize_checked(&inline)?.text.as_str(),
        short
    );
    let raw: qsl::StackString<55> = qsl::StackString::try_from_str(short)?;
    let raw_inline: qsl::StackOut<59> = AggressiveText { text: raw }.serialize_checked()?;
    assert_eq!(
        AggressiveText::deserialize_checked(&raw_inline)?
            .text
            .as_str(),
        short
    );

    // The 4096-byte bounded container exceeds the limit. Allocate directly on
    // the heap instead of constructing a large InlineString on the stack first.
    let large: Box<qsl::InlineString<4096>> = qsl::InlineString::try_boxed_from_str(&text)?;
    let compressed: Vec<u8> = CompressedText { text: large }.serialize_checked()?;
    assert_eq!(
        CompressedText::deserialize_checked(&compressed)?
            .text
            .as_str(),
        text
    );
    println!(
        "{} UTF-8 characters = {} bytes; raw frame={} B; QSL-LZ frame={} B",
        text.chars().count(),
        text.len(),
        bytes.len(),
        compressed.len()
    );
    println!("Small string: InlineString<55> on stack, QSL-LZ frame={} B; StackString<55> raw frame={} B",
        inline.len(), raw_inline.len());
    println!("Large string: heap storage and Vec output; borrowed read and String reuse verified");

    let uniform = uniform_texts().remove(0);
    let packed = CompressedText {
        text: qsl::InlineString::try_boxed_from_str(&uniform)?,
    }
    .serialize_checked()?;
    assert!(packed.len() < uniform.len() / 2);
    assert_eq!(
        CompressedText::deserialize_checked(&packed)?.text.as_str(),
        uniform
    );
    println!(
        "Uniform UTF-8 control: {} useful B -> {} QSL-LZ wire B",
        uniform.len(),
        packed.len()
    );
    // The schema maximum selects storage even when this particular value is short.
    let short_heap = CompressedText {
        text: qsl::InlineString::try_boxed_from_str("QSL")?,
    };
    let _: Vec<u8> = short_heap.serialize_checked()?;

    let fixed = FixedText { text: *b"rust" };
    let fixed_frame: qsl::StackOut<4> = fixed.serialize_checked()?;
    assert_eq!(fixed_frame, b"rust");
    Ok(())
}
