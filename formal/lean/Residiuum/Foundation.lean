import Residiuum.Identity
import Residiuum.Observation
import Residiuum.State
import Residiuum.WellFormed
import Residiuum.Operations
import Residiuum.Observe
import Residiuum.Vectors
import Residiuum.Refinement

/-!
# Residiuum.Foundation

FAS-2 kernel + FAS-3 refinement re-export.
-/

namespace Residiuum

/-- FAS-2 package marker theorem: foundation kernel type-checks and proves Init WF. -/
theorem fas2_foundation_ok : WellFormed Init ∧
    (∀ i : Input, i.operationId ∈ foundationOperationIds) :=
  ⟨init_well_formed, input_has_operation_id⟩

/-- FAS-3 package marker: authority-binding vertical slice is present. -/
theorem fas3_refinement_ok :
    Residiuum.Refinement.alphaState Residiuum.Refinement.ConcreteState.empty = Init :=
  Residiuum.Refinement.init_correspondence

end Residiuum
