import Std

/-!
Arithmetic models used by the QSL overview. These lemmas describe the stated
preconditions; they do not verify the Rust implementation, pointer provenance,
CPU semantics, concurrency, UTF-8, or performance.
-/
namespace QslOverview

/-- A decoder that is a left inverse makes the encoder injective on this domain. -/
theorem roundtrip_implies_injective {A B : Type} (encode : A → B) (decode : B → A)
    (roundtrip : ∀ value, decode (encode value) = value) :
    ∀ a b, encode a = encode b → a = b := by
  intro a b same
  have decoded := congrArg decode same
  simpa only [roundtrip] using decoded

/-- Check subtraction only after establishing position <= length. -/
theorem checked_read_window (position count length : Nat)
    (positionFits : position ≤ length) (countFits : count ≤ length - position) :
    ∀ index, index < count → position + index < length := by
  intro index inside
  omega

/-- Every accepted LZ token preserves the bounded initialized-prefix invariant. -/
theorem lz_token_bound (produced count capacity : Nat)
    (prefixFits : produced ≤ capacity) (tokenFits : count ≤ capacity - produced) :
    produced + count ≤ capacity := by omega

/-- Overlapping matches may read from the prefix grown by earlier copied bytes. -/
theorem lz_match_source (produced distance copied : Nat)
    (positive : 0 < distance) (history : distance ≤ produced) :
    produced + copied - distance < produced + copied := by omega

/-- Subtraction of a repeated byte base cannot borrow across two valid lanes.
The lane preconditions, not a raw machine subtraction alone, justify the identity. -/
theorem two_lane_base_subtraction (lo hi base : Nat)
    (lowBound : base ≤ lo) (highBound : base ≤ hi) :
    (lo + 256 * hi) - (base + 256 * base) =
      (lo - base) + 256 * (hi - base) := by omega

def optionalBytes (a b : Bool) : Nat :=
  (if a then 1 else 0) + (if b then 1 else 0)

/-- Equal payload lengths do not identify arbitrary subsets of optional fields. -/
theorem optional_length_ambiguous :
    optionalBytes true false = optionalBytes false true ∧
      (true, false) ≠ (false, true) := by decide

/-- For two one-byte fields, an empty payload identifies the empty subset. -/
theorem optional_empty_exact (a b : Bool) :
    optionalBytes a b = 0 ↔ a = false ∧ b = false := by
  cases a <;> cases b <;> decide

/-- For two one-byte fields, the full payload identifies both present fields. -/
theorem optional_full_exact (a b : Bool) :
    optionalBytes a b = 2 ↔ a = true ∧ b = true := by
  cases a <;> cases b <;> decide

end QslOverview
