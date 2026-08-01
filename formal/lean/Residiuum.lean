import Residiuum.Foundation

/-!
# Residiuum

FAS-2 abstract semantic kernel root.
FAS-1 smoke theorem retained for toolchain check compatibility.
-/

/-- FAS-1 smoke: trivial theorem for toolchain gate. -/
theorem fas1_smoke : True := trivial

/-- FAS-2 package marker re-export. -/
theorem fas2_ok : Residiuum.WellFormed Residiuum.Init := Residiuum.init_well_formed
