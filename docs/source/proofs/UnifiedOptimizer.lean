import Std

/-!
Arithmetic obligations for the unified optimizer and dyn v1.
No admitted proofs. This does not establish Rust ownership, initialized memory,
FFI layout, UTF-8 validity, CPU instruction semantics, or measured speed.
-/
namespace UnifiedOptimizer

/-- A nonnegative interval rank is exactly invertible, including signed minima. -/
theorem interval_restore (minimum value : Int) : (value - minimum) + minimum = value := by omega

/-- Keeping the integer operand and exact remainder avoids lossy quantization. -/
theorem operand_restore (rank step : Nat) : step * (rank / step) + rank % step = rank := by
  have h := Nat.mod_add_div rank step
  omega

/-- Actual length zero has a valid shifted representation and is recovered. -/
theorem shifted_length_restore (length : Nat) : (length + 1) - 1 = length := by omega

/-- A dynamic header and its contents fit only if both are inside the field extent. -/
theorem dynamic_extent (start header length boundary : Nat)
    (bound : start + header + length ≤ boundary) :
    start + header ≤ boundary ∧ ∀ i, i < length → start + header + i < boundary := by
  constructor
  · omega
  · intro i hi
    omega

/-- The table and trailer partition the checked suffix; the trailer is byte-addressed. -/
theorem trailer_backwards (index_start trailer_start back : Nat)
    (ordered : index_start ≤ trailer_start)
    (distance : back = trailer_start - index_start) :
    trailer_start - back = index_start := by omega

/-- Logical values do not depend on an invertible physical field permutation. -/
theorem physical_permutation_preserves (value : Nat → Int) (perm inverse : Nat → Nat)
    (roundtrip : ∀ i, perm (inverse i) = i) (i : Nat) :
    value (perm (inverse i)) = value i := by rw [roundtrip]

/-- Zero-based MSB-first indexing uses byte floor(offset/8) and bit 7-offset%8. -/
theorem msb_fifth_byte : 32 / 8 = 4 ∧ 7 - 32 % 8 = 7 := by decide

/-- Reserving count equal-width scalars covers each complete scalar read. -/
theorem scalar_island_window (start count bytes index limit : Nat)
    (inside : index < count) (bounded : start + count * bytes ≤ limit) :
    start + index * bytes + bytes ≤ limit := by
  have h := Nat.mul_le_mul_right bytes (Nat.succ_le_of_lt inside)
  simp only [Nat.succ_mul] at h
  omega

/-- A mixed-width segment stays inside its prevalidated prefix/current/suffix sum. -/
theorem mixed_island_window (start prefixBytes current suffix limit : Nat)
    (bounded : start + prefixBytes + current + suffix ≤ limit) :
    start + prefixBytes + current ≤ limit := by omega

/-- The final conjunction cannot discard an invalid field, irrespective of scan order. -/
theorem aggregate_validation (checks : List Bool) (seed : Bool) :
    checks.foldl (fun a b => a && b) seed = true ↔
      seed = true ∧ ∀ b ∈ checks, b = true := by
  induction checks generalizing seed with
  | nil => simp
  | cons x xs ih => simp [List.foldl, ih, Bool.and_eq_true, and_assoc]

/-- A single aggregate budget charge equals sequential charges on a valid span. -/
theorem aggregate_budget (budget a b : Nat) : budget - a - b = budget - (a + b) := by omega

/-- Aligned progression deltas recover exactly; off-step values must be rejected. -/
theorem progression_restore (minimum delta step : Nat) (aligned : delta % step = 0) :
    minimum + (delta / step) * step = minimum + delta := by
  have h := Nat.mod_add_div delta step
  simp only [aligned, Nat.zero_add] at h
  rw [Nat.mul_comm] at h
  omega

/-- Two exact 64-bit chunks reassemble a rank without a 135-bit temporary. -/
theorem split_rank_restore (rank : Nat) :
    rank % 18446744073709551616 + 18446744073709551616 * (rank / 18446744073709551616) = rank := by
  exact Nat.mod_add_div rank 18446744073709551616

/-- A valid 128-bit rank has two individually bounded 64-bit pieces. -/
theorem split_rank_bounds (rank : Nat) (bound : rank < 340282366920938463463374607431768211456) :
    rank % 18446744073709551616 < 18446744073709551616 ∧
    rank / 18446744073709551616 < 18446744073709551616 := by omega

/-- Eight repetitions restore the bit residue for a constant scalar width. -/
theorem array_residue_period (offset count width : Nat) :
    (offset + (count + 8) * width) % 8 = (offset + count * width) % 8 := by
  simp [Nat.add_mul, Nat.add_mod, Nat.mul_mod]

/-- An array's byte barrier produces a byte-aligned boundary. -/
theorem array_byte_barrier (bits : Nat) : ((bits + 7) / 8 * 8) % 8 = 0 := by
  simp

/-- A checked contiguous copy stays in the initialized prefix. This is only
    the integer interval obligation, not a Rust pointer/provenance proof. -/
theorem initialized_copy_interval (start count initialized : Nat)
    (bound : start + count ≤ initialized) :
    ∀ i, i < count → start + i < initialized := by omega

/-- A strict size bound can be checked before allocating an exact-sized output. -/
theorem exact_size_guard (size maximum : Nat) (accepted : size ≤ maximum) :
    ¬ maximum < size := by omega

/-- Nonzero shifted length stays representable when capacity leaves one state. -/
theorem shifted_word_bound (length capacity word_max : Nat)
    (fits : length ≤ capacity) (room : capacity < word_max) :
    0 < length + 1 ∧ length + 1 ≤ word_max := by omega

/-- A field permutation with an explicit inverse preserves the field identity. -/
theorem canonical_field_inverse {α : Type} (forward backward : α → α)
    (inverse : ∀ x, backward (forward x) = x) (field : α) :
    backward (forward field) = field := inverse field

/-- Reserving a whole fixed heap object bounds every initialized prefix.
    This replaces repeated budget charges, not per-token destination bounds. -/
theorem heap_reservation_covers_prefix (produced capacity objectSize budget : Nat)
    (within : produced ≤ capacity) (layout : capacity ≤ objectSize)
    (reserved : objectSize ≤ budget) : produced ≤ budget := by omega

/-- A second fixed allocation is admitted only from the remaining allowance.
    Physical allocator metadata and Rust ownership are outside this arithmetic. -/
theorem aggregate_heap_guard (spent requested budget : Nat)
    (guard : requested ≤ budget - spent) (prior : spent ≤ budget) :
    spent + requested ≤ budget := by omega

end UnifiedOptimizer
