package airplane.gates.scope_boundary

# Reference policy for the Rust harness: a coach pack must carry a human
# escalation path and must not claim clinical/therapy behavior.

deny["scopeBoundary.escalationRequired must be true"] {
  input.followup.scopeBoundary.escalationRequired != true
}

deny["scopeBoundary.onClinicalRisk must name the escalation action"] {
  input.followup.scopeBoundary.onClinicalRisk == ""
}

clinical_terms := {
  "diagnose",
  "diagnosis",
  "treat",
  "treatment",
  "therapy",
  "therapist",
  "psychotherapy",
  "prescribe",
  "medication",
  "patient",
  "clinical intervention",
}

deny[msg] {
  value := walk(input)[[_], v]
  is_string(v)
  term := clinical_terms[_]
  contains(lower(v), term)
  msg := sprintf("clinical-claim language is not allowed in a coach pack: %q", [term])
}
