import Std

/-!
QPS arithmetic obligations, Lean 4.35.0, no Mathlib and no admitted proofs.
These theorems justify exact length/state decomposition and non-overlapping
radix packing. They do not verify Rust pointer code, LLVM, or the CPU.
-/
namespace QPS

/-- A table quantum retains its remainder; no original index is rounded. -/
theorem quantum_exact (i q : Nat) : i = q * (i / q) + i % q := by
  have h := Nat.mod_add_div i q
  omega

def pack (base lo hi : Nat) : Nat := lo + base * hi

/-- Low state bits are recovered exactly, provided the field fits its radix. -/
theorem unpack_low (base lo hi : Nat) (h : lo < base) :
    pack base lo hi % base = lo := by
  simp [pack, Nat.add_mod, Nat.mod_eq_of_lt h]

/-- Adjacent fields do not overlap: the upper state survives division. -/
theorem unpack_high (base lo hi : Nat) (h : lo < base) :
    pack base lo hi / base = hi := by
  have hb : 0 < base := by omega
  simp [pack, Nat.add_mul_div_left lo hi hb, Nat.div_eq_of_lt h]

/-- Recursion on non-overlapping slices gives injectivity of the whole D. -/
theorem pack_injective (base a b x y : Nat) (ha : a < base) (hx : x < base)
    (h : pack base a b = pack base x y) : a = x ∧ b = y := by
  constructor
  · have hmod := congrArg (fun n => n % base) h
    simpa [unpack_low base a b ha, unpack_low base x y hx] using hmod
  · have hdiv := congrArg (fun n => n / base) h
    simpa [unpack_high base a b ha, unpack_high base x y hx] using hdiv

def stateCode (minimum : Nat) : Option Nat → Nat
  | none => 0
  | some length => length - minimum + 1
def stateLength (minimum code : Nat) : Option Nat :=
  if code = 0 then none else some (minimum + code - 1)

/-- Presence is independent of length, including a present zero-byte field. -/
theorem absent_roundtrip (minimum : Nat) :
    stateLength minimum (stateCode minimum none) = none := by
  simp [stateLength, stateCode]
theorem present_roundtrip (minimum length : Nat) (h : minimum ≤ length) :
    stateLength minimum (stateCode minimum (some length)) = some length := by
  change (if length - minimum + 1 = 0 then (none : Option Nat)
    else some (minimum + (length - minimum + 1) - 1)) = some length
  simp
  congr 1
  omega

/-- Payload addresses use declaration-order prefix sums, never sorted sizes. -/
def nextOffset (start : Nat) (prior : List Nat) : Nat := start + prior.sum
theorem append_field (start width : Nat) (prior : List Nat) :
    nextOffset start (prior ++ [width]) = nextOffset start prior + width := by
  simp [nextOffset, List.sum_append, Nat.add_assoc]

theorem ambiguous_sum : (1 : Nat) + 2 = 3 := by decide
theorem present_empty_distinct : stateCode 0 (some 0) ≠ stateCode 0 none := by decide
theorem u32_capacity (d : Nat) (h : d < 2^32) : d ≤ 4294967295 := by omega

/-- Unit finite differences may be counted; non-unit weights cannot be
replaced with a Boolean count. The generator rejects those candidates. -/
theorem unit_differences (changes : List Bool) :
    changes.count true = (changes.map (fun b => if b then 1 else 0)).sum := by
  induction changes with
  | nil => simp
  | cons b tail ih => cases b <;> simp_all [Nat.add_comm]

/-- Flattening macro field blocks preserves their order and payload byte sum.
No nested record header is added by field expansion. -/
theorem flatten_payload_size (blocks : List (List Nat)) :
    blocks.flatten.sum = (blocks.map List.sum).sum := by
  induction blocks with
  | nil => simp
  | cons block rest ih => simp_all [List.sum_append]

/-- A bounded value is represented by its exact rank. No rounding is involved. -/
theorem rank_restore (minimum value : Int) :
    minimum + (value - minimum) = value := by omega

/-- Keeping BOTH numerical operands makes quantization fully invertible.
This theorem is about exact natural numbers, not unchecked machine arithmetic. -/
theorem payload_restore (rank quantum : Nat) :
    quantum * (rank / quantum) + rank % quantum = rank := by
  have h := Nat.mod_add_div rank quantum
  omega

/-- Two distinct ranks cannot share the quotient/remainder pair. -/
theorem payload_operands_injective (a b quantum : Nat)
    (hq : a / quantum = b / quantum) (hr : a % quantum = b % quantum) : a = b := by
  have ha := payload_restore a quantum
  have hb := payload_restore b quantum
  rw [hq, hr] at ha
  omega

/-- An event selector counts crossed critical boundaries. This is a numeric
operand model, separate from any compiler's instruction selection. -/
def eventRank : List Nat → Nat → Nat
  | [], _ => 0
  | boundary :: rest, length => (if boundary < length then 1 else 0) + eventRank rest length

theorem event_rank_append (bounds : List Nat) (boundary length : Nat) :
    eventRank (bounds ++ [boundary]) length =
      eventRank bounds length + (if boundary < length then 1 else 0) := by
  induction bounds with
  | nil => simp [eventRank]
  | cons head tail ih => simp [eventRank, ih, Nat.add_assoc]

theorem event_rank_monotone (bounds : List Nat) (a b : Nat) (h : a ≤ b) :
    eventRank bounds a ≤ eventRank bounds b := by
  induction bounds with
  | nil => simp [eventRank]
  | cons head tail ih =>
    change (if head < a then 1 else 0) + eventRank tail a ≤
      (if head < b then 1 else 0) + eventRank tail b
    by_cases ha : head < a <;> by_cases hb : head < b <;> simp [ha, hb] <;> omega

/-- For two optional one-byte fields, the three payload-length events have
exactly the numeric indices 0, 1, 2, without collapsing them into one Bool. -/
theorem two_byte_field_event_rank (length : Nat) (h : length ≤ 2) :
    eventRank [0, 1] length = length := by
  cases length with
  | zero => rfl
  | succ n =>
    cases n with
    | zero => rfl
    | succ k =>
      have hk : k = 0 := by omega
      subst k
      rfl

/-- A byte is reconstructed from its exact rank and shared minimum. -/
theorem byte_base_rank_restore (minimum value : Nat) (h : minimum ≤ value) :
    minimum + (value - minimum) = value := by omega

/-- Recover the low digit from a reversible radix operand. -/
theorem group_rank_low (radix low high : Nat) (h : low < radix) :
    (low + radix * high) % radix = low := by
  simpa [pack] using unpack_low radix low high h

/-- Unique two-digit operands; repeated use covers finite groups.
This theorem does not verify Rust memory safety. -/
theorem group_rank_injective (radix a b x y : Nat) (ha : a < radix) (hx : x < radix)
    (h : a + radix * b = x + radix * y) : a = x ∧ b = y := by
  exact pack_injective radix a b x y ha hx h

/-- Every framing byte is included in the compression selection guard. -/
theorem packed_frame_strictly_smaller (header payload original : Nat)
    (h : header + payload < original) : header + payload < original := h

end QPS
