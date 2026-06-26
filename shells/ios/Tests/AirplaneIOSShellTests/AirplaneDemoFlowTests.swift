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

@Test func simulatorMLXTextInferenceReturnsRawSchemaCompatibleSpans() throws {
    let provider = SimulatorMLXSwiftTextInferenceProvider()
    let raw = try provider.complete(
        TextInferenceRequest(
            system: "Return spans.",
            user: AirplaneDemoFlow.sampleNote,
            jsonSchemaName: "airplane.identifier_spans",
            jsonSchema: #"{"type":"object"}"#,
            backend: BackendSelection()
        )
    )

    let object = try #require(JSONSerialization.jsonObject(with: Data(raw.utf8)) as? [String: Any])
    let spans = try #require(object["spans"] as? [[String: Any]])

    #expect(spans.count == 4)
    #expect(spans.contains { $0["text"] as? String == "COACH-4821" && $0["entity"] as? String == "MEMBER_ID" })
}

@Test func simulatorScrubBackendUsesRawInferenceProvider() {
    struct SingleSpanInference: TextInferenceProviding {
        let runtime: BackendRuntime = .mlxSwiftMock

        func complete(_ request: TextInferenceRequest) throws -> String {
            #"{"spans":[{"text":"COACH-4821","entity":"MEMBER_ID"}]}"#
        }
    }

    var flow = AirplaneDemoFlow()

    flow.capture()
    flow.scrubAndGate(provider: SimulatorMLXSwiftScrubBackend(inference: SingleSpanInference()))

    #expect(flow.redactions == [BackendRedaction(entity: "MEMBER_ID", layer: "mlx-swift-mock")])
    #expect(flow.scrubbedText.contains("[MEMBER_ID]"))
    #expect(flow.scrubbedText.contains("Jordan Lee"))
    #expect(flow.gateResult == DemoGateResult(passed: false, residualCount: 3))
    #expect(flow.record == nil)
}

@Test func realMLXRuntimeIsVisibleButHardwareGatedInSimulator() {
    var flow = AirplaneDemoFlow()

    flow.selectBackend(.onDeviceMLXSwift)
    #expect(flow.backend.runtime == .onDeviceMLXSwift)
    #expect(flow.backend.model == "ternary-bonsai-1.7b@mlx-text-unwired")
    #expect(!flow.selectedBackendCanRunInSimulator)

    flow.capture()
    flow.scrubAndGate()

    #expect(flow.phase == .gated)
    #expect(flow.gateResult == DemoGateResult(passed: false, residualCount: 1))
    #expect(flow.lastRequest == nil)
    #expect(flow.lastResponse == nil)
    #expect(flow.record == nil)
    #expect(flow.deliveredPayload == nil)
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

@Test func backendDTOsDecodeSharedScrubResponseFixture() throws {
    let fixture = try readRepoFixture("docs/contracts/scrub-response.sample.json")
    let response = try JSONDecoder().decode(BackendScrubResponse.self, from: fixture)

    #expect(response.gatePass)
    #expect(response.residualCount == 0)
    #expect(response.redactions.map(\.entity).contains("MEMBER_ID"))
    #expect(response.record.clientPseudonym == "client ready circle")
    #expect(response.record.autonomyDelta.signals == ["self_initiated", "commitment_completed"])
}

private func readRepoFixture(_ relativePath: String) throws -> Data {
    let fm = FileManager.default
    var dir = URL(fileURLWithPath: fm.currentDirectoryPath, isDirectory: true)
    for _ in 0..<8 {
        let candidate = dir.appendingPathComponent(relativePath)
        if fm.fileExists(atPath: candidate.path) {
            return try Data(contentsOf: candidate)
        }
        dir.deleteLastPathComponent()
    }
    Issue.record("Missing fixture \(relativePath) from \(fm.currentDirectoryPath)")
    return Data()
}
