import Testing
import Foundation
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
    #expect(flow.lastRequest?.backend.runtime == .mlxSwiftMock)
    #expect(flow.lastResponse?.gatePass == true)
    #expect(flow.gateResult == DemoGateResult(passed: true, residualCount: 0))
    #expect(flow.secureStoreSnapshot.redactionMapStored)
    #expect(flow.secureStoreSnapshot.redactionCount == 4)
    #expect(flow.redactions.allSatisfy { $0.layer == "mlx-swift-mock" })
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

@Test func backendSelectionRoutesSimulatorProvider() {
    var flow = AirplaneDemoFlow()

    flow.selectBackend(.edgeHTTPMock)
    flow.capture()
    flow.scrubAndGate()

    #expect(flow.backend.runtime == .edgeHTTPMock)
    #expect(flow.lastRequest?.backend.model == "ternary-bonsai-1.7b@mock-edge-http")
    #expect(flow.redactions.allSatisfy { $0.layer == "edge-http-mock" })
    #expect(flow.record?.clientPseudonym == "client ready circle")
}

@Test func scrubResponseUsesWebBackendCompatibleKeys() throws {
    var flow = AirplaneDemoFlow()

    flow.capture()
    flow.scrubAndGate()

    let response = try #require(flow.lastResponse)
    let data = try JSONEncoder().encode(response)
    let object = try #require(JSONSerialization.jsonObject(with: data) as? [String: Any])
    let record = try #require(object["record"] as? [String: Any])

    #expect(object["scrubbed_text"] != nil)
    #expect(object["gate_pass"] as? Bool == true)
    #expect(object["residual_count"] as? Int == 0)
    #expect(object["redactions"] != nil)
    #expect(record["client_pseudonym"] as? String == "client ready circle")
    #expect(record["follow_ups"] != nil)
    #expect(record["risk_flags"] != nil)
    #expect(record["autonomy_delta"] != nil)
    #expect(record["next_touch"] as? String == "scheduled")
}
