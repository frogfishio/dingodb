import Residiuum.Identity
import Residiuum.Observation
import Residiuum.State
import Residiuum.WellFormed
import Residiuum.Operations
import Residiuum.Observe
import Residiuum.Vectors
import Residiuum.Refinement
import Residiuum.Consistency

/-!
# Residiuum.Foundation

FAS-2 kernel + FAS-3 refinement + FAS-4 consistency re-export.
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

/-- FAS-4 package marker: consistency family type-checks. -/
theorem fas4_consistency_ok :
    WellFormed Init :=
  Residiuum.Consistency.healthy_island_init_wf

end Residiuum
