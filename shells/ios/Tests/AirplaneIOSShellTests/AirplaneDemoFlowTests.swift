import Testing
@testable import AirplaneIOSShell

@Test func simulatorFlowBlocksEgressUntilGateAndHoldStatesComplete() {
    var flow = AirplaneDemoFlow()

    #expect(flow.phase == .idle)
    #expect(flow.deliveredPayload == nil)

    flow.capture()
    #expect(flow.phase == .capturing)
    #expect(flow.secureStoreSnapshot.rawNoteStored)
    #expect(flow.deliveredPayload == nil)

    flow.structureCleanRecord()
    #expect(flow.record == nil)
    #expect(flow.phase == .capturing)

    flow.scrubAndGate()
    #expect(flow.phase == .gated)
    #expect(flow.gateResult == DemoGateResult(passed: true, residualCount: 0))
    #expect(flow.secureStoreSnapshot.redactionMapStored)
    #expect(flow.secureStoreSnapshot.redactionCount == 4)
    #expect(!flow.scrubbedText.contains("Jordan Lee"))
    #expect(!flow.scrubbedText.contains("COACH-4821"))
    #expect(flow.deliveredPayload == nil)

    flow.structureCleanRecord()
    #expect(flow.phase == .structured)
    #expect(flow.deliveredPayload == nil)

    flow.holdSendWhileOffline()
    #expect(flow.phase == .sendHeld)
    #expect(flow.deliveredPayload == nil)

    flow.reconnectAndFlush()
    #expect(flow.phase == .delivered)
    #expect(flow.deliveredPayload == flow.record)
}

@Test func resetClearsSimulatorPrivateStateAndDeliveredPayload() {
    var flow = AirplaneDemoFlow()

    flow.capture()
    flow.scrubAndGate()
    flow.structureCleanRecord()
    flow.holdSendWhileOffline()
    flow.reconnectAndFlush()
    flow.reset()

    #expect(flow.phase == .idle)
    #expect(flow.capturedText.isEmpty)
    #expect(flow.scrubbedText.isEmpty)
    #expect(flow.redactions.isEmpty)
    #expect(flow.gateResult == nil)
    #expect(flow.record == nil)
    #expect(flow.deliveredPayload == nil)
    #expect(!flow.secureStoreSnapshot.rawNoteStored)
    #expect(!flow.secureStoreSnapshot.redactionMapStored)
}
