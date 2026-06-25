package airplane.gates.reward

# Reference policy for the Rust harness: reward references must not use
# engagement/dependence terms. The pack's forbiddenTerms field is the deny-list
# declaration, not a violation by itself.

deny[msg] {
  forbidden := {lower(t) | t := input.followup.reward.forbiddenTerms[_]}
  signal := input.followup.reward.usedSignals[_]
  forbidden[lower(signal)]
  msg := sprintf("reward references forbidden engagement term %q", [signal])
}

deny[msg] {
  not input.followup.reward.usedSignals
  forbidden := {lower(t) | t := input.followup.reward.forbiddenTerms[_]}
  signal := input.followup.reward.autonomySignals[_]
  forbidden[lower(signal)]
  msg := sprintf("reward autonomy signal includes forbidden engagement term %q", [signal])
}
