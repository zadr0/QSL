import Std

/-!
Arithmetic contracts for the QSL deserialization modernization.
No axioms, no admitted proofs. This model does not verify Rust pointer validity,
provenance, aliasing, atomic memory ordering, LLVM, or operating-system wait APIs.
-/
namespace Modernization

/-- Every byte actually copied belongs to the checked input window.
The unused high bytes of a register are local zeroes, not extra input reads. -/
theorem exact_load_window (offset count length : Nat)
    (bounded : offset + count ≤ length) :
    ∀ i, i < count → offset + i < length := by
  omega

/-- Splitting a long group preserves its exact useful byte count. -/
theorem register_split (bytes : Nat) :
    min bytes 16 + (bytes - min bytes 16) = bytes := by
  omega

/-- Only the initialized prefix may be exposed or dropped. -/
theorem initialized_prefix (length capacity index : Nat)
    (bounded : length ≤ capacity) (initialized : index < length) :
    index < capacity := by omega

/-- Removing the last initialized element cannot expose an uninitialized slot. -/
theorem prefix_pop (length capacity : Nat) (bounded : length ≤ capacity) :
    length - 1 ≤ capacity := by omega

def rank (minimum value : Int) : Int := value - minimum + 1

/-- Positive shifted ranks cannot collide with the reserved absent code 0. -/
theorem rank_positive (minimum value : Int) (bounded : minimum ≤ value) :
    0 < rank minimum value := by unfold rank; omega

/-- Decoding a present shifted value is exact, including negative minima. -/
theorem rank_restore (minimum value : Int) :
    rank minimum value - 1 + minimum = value := by unfold rank; omega

theorem rank_injective (minimum a b : Int)
    (same : rank minimum a = rank minimum b) : a = b := by
  unfold rank at same; omega

/-- A selected machine width must contain the entire rank, without wrapping. -/
theorem rank_fits (minimum maximum value limit : Int)
    (lo : minimum ≤ value) (hi : value ≤ maximum)
    (fits : maximum - minimum + 1 < limit) :
    0 < rank minimum value ∧ rank minimum value < limit := by
  unfold rank; omega

/-- Some(0) sets presence even when its stored scalar is also zero. -/
def optionalCode : Option Nat → Bool × Nat
  | none => (false, 0)
  | some n => (true, n)

def optionalDecode (code : Bool × Nat) : Option Nat :=
  if code.1 then some code.2 else none

theorem bitmap_roundtrip (value : Option Nat) :
    optionalDecode (optionalCode value) = value := by
  cases value <;> simp [optionalCode, optionalDecode]

theorem zero_not_absent : optionalCode (some 0) ≠ optionalCode none := by decide

/-- Static output capacity follows from the schema bound and valid values. -/
theorem proven_cache (actual schemaBound capacity : Nat)
    (valid : actual ≤ schemaBound) (cache : schemaBound ≤ capacity) :
    actual ≤ capacity := by omega

/-- Hybrid prefix and raw-tail budgets are charged once, before allocation. -/
theorem hybrid_budget (prefixBytes tail limit : Nat) (hp : prefixBytes ≤ limit)
    (ht : tail ≤ limit - prefixBytes) : prefixBytes + tail ≤ limit := by omega

/-- A checked boundary separates disjoint prefix and tail byte intervals. -/
theorem hybrid_boundary (header prefixLength total i j : Nat)
    (_fits : header + prefixLength ≤ total)
    (inPrefix : header ≤ i ∧ i < header + prefixLength)
    (inTail : header + prefixLength ≤ j ∧ j < total) : i < j := by omega

/-- Pool reservation changes available and owned counts without losing a slot.
This is a conservation model, not a concurrency or lost-wakeup proof. -/
theorem pool_reserve (available occupied capacity : Nat)
    (invariant : available + occupied = capacity) (free : 0 < available) :
    available - 1 + (occupied + 1) = capacity := by omega

theorem pool_release (available occupied capacity : Nat)
    (invariant : available + occupied = capacity) (owned : 0 < occupied) :
    available + 1 + (occupied - 1) = capacity := by omega

/-- Within the documented limit an owned buffer fits; above it, it cannot be
selected by this predicate, even if explicit inline storage was requested. -/
def inlineEligible (maximum limit : Nat) : Bool := decide (maximum ≤ limit)

theorem strict_stack_limit (maximum limit : Nat) (over : limit < maximum) :
    inlineEligible maximum limit = false := by simp [inlineEligible]; omega

end Modernization
