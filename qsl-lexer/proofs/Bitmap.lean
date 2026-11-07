import Std
namespace QslBitmap

def marker (width trailingZeros : Nat) : Nat := width - trailingZeros
/-- A nonzero native mask has a trailing-zero position below its width. -/
theorem marker_positive (width trailingZeros : Nat) (nonzero : trailingZeros < width) :
    0 < marker width trailingZeros := by unfold marker; omega
/-- The one-time maximum-marker check proves the table index is in bounds. -/
theorem table_index_bounded (width trailingZeros count : Nat)
    (nonzero : trailingZeros < width) (checked : marker width trailingZeros ≤ count) :
    marker width trailingZeros - 1 < count := by unfold marker at *; omega
/-- A marker encoded at width-index is recovered exactly, including 128-bit words. -/
theorem marker_roundtrip (width index : Nat) (bounded : index ≤ width) :
    marker width (width-index) = index := by unfold marker; omega
/-- Child input windows stay inside the enclosing validated region. -/
theorem nested_window (start count length index : Nat)
    (frame : start+count ≤ length) (field : index < count) :
    start+index < length := by omega
end QslBitmap
