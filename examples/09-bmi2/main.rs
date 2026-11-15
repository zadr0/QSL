use qsl::optimizer::kernels;
use std::hint::black_box;

include!(concat!(env!("OUT_DIR"), "/bmi2.rs"));

fn main() -> qsl::QslResult<()> {
    let values = black_box([0u32, 1, 2047, 4095]);
    // Validate ranks once before entering an unchecked packing kernel.
    kernels::validate4(values, [12; 4])?;
    let scalar = kernels::pack4::<12, 12, 12, 12>(values);
    assert_eq!(kernels::unpack4::<12, 12, 12, 12>(scalar), values);
    let packet = Bmi2Packet {
        f0: values[0],
        f1: values[1],
        f2: values[2],
        f3: values[3],
    };
    let wire: qsl::StackOut<6> = packet.serialize_checked()?;
    assert_eq!(wire.as_ref(), &scalar.to_le_bytes()[..6]);
    let restored = Bmi2Packet::deserialize_checked(&wire)?;
    assert_eq!([restored.f0, restored.f1, restored.f2, restored.f3], values);
    println!(
        "4 x 12 bits: 16 native B -> 6 wire B; bytes={:02x?}",
        wire.as_ref()
    );
    println!(
        "Generated writer: {}; generated reader: scalar",
        if cfg!(all(
            target_arch = "x86_64",
            target_endian = "little",
            target_feature = "bmi2"
        )) {
            "PEXT (BMI2 target enabled)"
        } else {
            "scalar fallback (BMI2 target not enabled)"
        }
    );

    // This executable may use std for detection; the kernels themselves are no_std.
    // The feature check is outside the kernels, with no per-field dispatch.
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("bmi2") {
        // SAFETY: CPU detection succeeded; validate4 established all four ranges.
        let packed = unsafe { kernels::x86::pack4_bmi2::<12, 12, 12, 12>(&values) };
        assert_eq!(packed, scalar);
        // SAFETY: packed uses only the canonical low 48 bits, and BMI2 is present.
        let decoded = unsafe { kernels::x86::unpack4_bmi2::<12, 12, 12, 12>(black_box(packed)) };
        assert_eq!(decoded, values);
        println!(
            "CPU supports BMI2: PEXT/PDEP kernels executed; scalar/BMI2 bytes and values match"
        );
    } else {
        println!("CPU lacks BMI2: skipped ISA kernels; portable round-trip passed");
    }
    #[cfg(not(target_arch = "x86_64"))]
    println!("Non-x86-64 target: portable round-trip passed");

    let invalid = Bmi2Packet {
        f0: 4096,
        f1: 0,
        f2: 0,
        f3: 0,
    };
    assert!(invalid.serialize_checked().is_err());
    println!("Checked writer rejects 4096: it does not fit the declared 12-bit domain");
    Ok(())
}
