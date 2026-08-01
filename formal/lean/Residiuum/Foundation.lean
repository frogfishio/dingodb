import Residiuum.Identity
import Residiuum.Observation
import Residiuum.State
import Residiuum.WellFormed
import Residiuum.Operations
import Residiuum.Observe
import Residiuum.Vectors

/-!
# Residiuum.Foundation

FAS-2 root: re-exports kernel modules and packages gate theorems.
-/


namespace Residiuum

/-- FAS-2 package marker theorem: foundation kernel type-checks and proves Init WF. -/
theorem fas2_foundation_ok : WellFormed Init ∧
    (∀ i : Input, i.operationId ∈ foundationOperationIds) :=
  ⟨init_well_formed, input_has_operation_id⟩

end Residiuum
