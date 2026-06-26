import Foundation

public enum DemoPhase: String, CaseIterable, Sendable {
    case idle = "Idle"
    case capturing = "Capturing"
    case scrubbing = "Scrubbing"
    case gated = "Gated"
    case structured = "Structured"
    case sendHeld = "Send held"
    case flushing = "Flushing"
    case delivered = "Delivered"
}

public enum BackendRuntime: String, CaseIterable, Codable, Equatable, Sendable {
    case mlxSwiftMock = "mlx-swift mock"
    case edgeHTTPMock = "edge HTTP mock"

    public var label: String {
        switch self {
        case .mlxSwiftMock: "MLX Swift mock"
        case .edgeHTTPMock: "Edge HTTP mock"
        }
    }

    public var detail: String {
        switch self {
        case .mlxSwiftMock:
            "Simulator stand-in for in-process Bonsai MLX text on iPhone 11 class hardware."
        case .edgeHTTPMock:
            "Simulator stand-in for the laptop `/api/scrub` JSON contract."
        }
    }
}

public struct BackendSelection: Codable, Equatable, Sendable {
    public var runtime: BackendRuntime
    public var deviceClass: String
    public var model: String

    public init(
        runtime: BackendRuntime = .mlxSwiftMock,
        deviceClass: String = "iPhone 11 / A13 simulator budget",
        model: String = "ternary-bonsai-1.7b@mock-mlx"
    ) {
        self.runtime = runtime
        self.deviceClass = deviceClass
        self.model = model
    }
}

public struct BackendScrubRequest: Codable, Equatable, Sendable {
    public let text: String
    public let backend: BackendSelection

    public init(text: String, backend: BackendSelection) {
        self.text = text
        self.backend = backend
    }
}

public struct BackendRedaction: Codable, Equatable, Sendable {
    public let entity: String
    public let layer: String

    public init(entity: String, layer: String) {
        self.entity = entity
        self.layer = layer
    }
}

public struct BackendCommitment: Codable, Equatable, Sendable {
    public let text: String
    public let status: String

    public init(text: String, status: String = "open") {
        self.text = text
        self.status = status
    }
}

public struct BackendAutonomyDelta: Codable, Equatable, Sendable {
    public let logged: Bool
    public let signals: [String]
    public let direction: String

    public init(logged: Bool = true, signals: [String], direction: String) {
        self.logged = logged
        self.signals = signals
        self.direction = direction
    }
}

public struct BackendRecord: Codable, Equatable, Sendable {
    public let clientPseudonym: String
    public let themes: [String]
    public let commitments: [BackendCommitment]
    public let followUps: [String]
    public let riskFlags: [String]
    public let autonomyDelta: BackendAutonomyDelta
    public let nextTouch: String

    public init(
        clientPseudonym: String,
        themes: [String],
        commitments: [BackendCommitment],
        followUps: [String],
        riskFlags: [String],
        autonomyDelta: BackendAutonomyDelta,
        nextTouch: String
    ) {
        self.clientPseudonym = clientPseudonym
        self.themes = themes
        self.commitments = commitments
        self.followUps = followUps
        self.riskFlags = riskFlags
        self.autonomyDelta = autonomyDelta
        self.nextTouch = nextTouch
    }

    enum CodingKeys: String, CodingKey {
        case clientPseudonym = "client_pseudonym"
        case themes
        case commitments
        case followUps = "follow_ups"
        case riskFlags = "risk_flags"
        case autonomyDelta = "autonomy_delta"
        case nextTouch = "next_touch"
    }
}

public struct BackendScrubResponse: Codable, Equatable, Sendable {
    public let scrubbedText: String
    public let redactions: [BackendRedaction]
    public let gatePass: Bool
    public let residualCount: Int
    public let record: BackendRecord

    public init(
        scrubbedText: String,
        redactions: [BackendRedaction],
        gatePass: Bool,
        residualCount: Int,
        record: BackendRecord
    ) {
        self.scrubbedText = scrubbedText
        self.redactions = redactions
        self.gatePass = gatePass
        self.residualCount = residualCount
        self.record = record
    }

    enum CodingKeys: String, CodingKey {
        case scrubbedText = "scrubbed_text"
        case redactions
        case gatePass = "gate_pass"
        case residualCount = "residual_count"
        case record
    }
}

public struct DemoGateResult: Equatable, Sendable {
    public let passed: Bool
    public let residualCount: Int
}

public struct DemoSecureStoreSnapshot: Equatable, Sendable {
    public let rawNoteStored: Bool
    public let redactionMapStored: Bool
    public let redactionCount: Int
}

public protocol AirplaneInferenceProvider: Sendable {
    var runtime: BackendRuntime { get }
    func scrub(_ request: BackendScrubRequest) throws -> BackendScrubResponse
}

public struct SimulatorMLXSwiftTextProvider: AirplaneInferenceProvider {
    public let runtime: BackendRuntime = .mlxSwiftMock

    public init() {}

    public func scrub(_ request: BackendScrubRequest) throws -> BackendScrubResponse {
        SimulatorBackend.scrub(request, layer: "mlx-swift-mock")
    }
}

public struct SimulatorEdgeHTTPProvider: AirplaneInferenceProvider {
    public let runtime: BackendRuntime = .edgeHTTPMock

    public init() {}

    public func scrub(_ request: BackendScrubRequest) throws -> BackendScrubResponse {
        SimulatorBackend.scrub(request, layer: "edge-http-mock")
    }
}

public struct AirplaneDemoFlow: Equatable, Sendable {
    public static let sampleNote = """
    Jordan Lee, member COACH-4821, met with coach Maya on March 12. Jordan wants to practice \
    a five minute breathing routine before Monday standup and asked not to share the note until \
    identifiers are removed.
    """

    public private(set) var phase: DemoPhase = .idle
    public private(set) var backend = BackendSelection()
    public private(set) var capturedText: String = ""
    public private(set) var scrubbedText: String = ""
    public private(set) var redactions: [BackendRedaction] = []
    public private(set) var gateResult: DemoGateResult?
    public private(set) var record: BackendRecord?
    public private(set) var deliveredPayload: BackendRecord?
    public private(set) var lastRequest: BackendScrubRequest?
    public private(set) var lastResponse: BackendScrubResponse?

    private var secureStore = SimulatorOnlySecureStore()

    public init(backend: BackendSelection = BackendSelection()) {
        self.backend = backend
    }

    public var secureStoreSnapshot: DemoSecureStoreSnapshot {
        secureStore.snapshot
    }

    public mutating func reset() {
        let selected = backend
        self = AirplaneDemoFlow(backend: selected)
    }

    public mutating func selectBackend(_ runtime: BackendRuntime) {
        backend.runtime = runtime
        backend.model = runtime == .mlxSwiftMock
            ? "ternary-bonsai-1.7b@mock-mlx"
            : "ternary-bonsai-1.7b@mock-edge-http"
    }

    public mutating func capture(_ note: String = AirplaneDemoFlow.sampleNote) {
        capturedText = note
        scrubbedText = ""
        redactions = []
        gateResult = nil
        record = nil
        deliveredPayload = nil
        lastRequest = nil
        lastResponse = nil
        secureStore.save(rawNote: note, redactions: [])
        phase = .capturing
    }

    public mutating func scrubAndGate(provider: AirplaneInferenceProvider? = nil) {
        guard phase == .capturing else { return }

        phase = .scrubbing
        let activeProvider = provider ?? defaultProvider(for: backend.runtime)
        let request = BackendScrubRequest(text: capturedText, backend: backend)
        lastRequest = request

        guard let response = try? activeProvider.scrub(request) else {
            gateResult = DemoGateResult(passed: false, residualCount: 1)
            phase = .gated
            return
        }

        scrubbedText = response.scrubbedText
        redactions = response.redactions
        record = response.record
        lastResponse = response
        secureStore.save(rawNote: capturedText, redactions: redactions)
        gateResult = DemoGateResult(passed: response.gatePass, residualCount: response.residualCount)
        phase = .gated
    }

    public mutating func structureCleanRecord() {
        guard phase == .gated, gateResult?.passed == true, record != nil else { return }
        phase = .structured
    }

    public mutating func holdSendWhileOffline() {
        guard phase == .structured, record != nil else { return }
        phase = .sendHeld
    }

    public mutating func reconnectAndFlush() {
        guard phase == .sendHeld, let record else { return }
        phase = .flushing
        deliveredPayload = record
        phase = .delivered
    }
}

private func defaultProvider(for runtime: BackendRuntime) -> AirplaneInferenceProvider {
    switch runtime {
    case .mlxSwiftMock:
        SimulatorMLXSwiftTextProvider()
    case .edgeHTTPMock:
        SimulatorEdgeHTTPProvider()
    }
}

private struct SimulatorOnlySecureStore: Equatable, Sendable {
    private var rawNote: String?
    private var redactionMap: [BackendRedaction] = []

    var snapshot: DemoSecureStoreSnapshot {
        DemoSecureStoreSnapshot(
            rawNoteStored: rawNote != nil,
            redactionMapStored: !redactionMap.isEmpty,
            redactionCount: redactionMap.count
        )
    }

    mutating func save(rawNote: String, redactions: [BackendRedaction]) {
        self.rawNote = rawNote
        self.redactionMap = redactions
    }
}

private enum SimulatorBackend {
    private static let replacements: [(needle: String, entity: String, replacement: String)] = [
        ("Jordan Lee", "PERSON", "[PERSON]"),
        ("Maya", "PERSON", "[PERSON]"),
        ("COACH-4821", "MEMBER_ID", "[MEMBER_ID]"),
        ("March 12", "DATE", "[DATE]")
    ]

    private static let blockedTokens = [
        "Jordan Lee",
        "Maya",
        "COACH-4821",
        "March 12"
    ]

    static func scrub(_ request: BackendScrubRequest, layer: String) -> BackendScrubResponse {
        var output = request.text
        var redactions: [BackendRedaction] = []

        for item in replacements where output.contains(item.needle) {
            output = output.replacingOccurrences(of: item.needle, with: item.replacement)
            redactions.append(BackendRedaction(entity: item.entity, layer: layer))
        }

        let residualCount = blockedTokens.reduce(0) { count, token in
            output.localizedCaseInsensitiveContains(token) ? count + 1 : count
        }

        return BackendScrubResponse(
            scrubbedText: output,
            redactions: redactions,
            gatePass: residualCount == 0,
            residualCount: residualCount,
            record: BackendRecord(
                clientPseudonym: "client ready circle",
                themes: ["routine building", "workplace preparation"],
                commitments: [BackendCommitment(text: "five minute breathing routine")],
                followUps: [
                    "Before the next touch, try this once on your own: five minute breathing routine."
                ],
                riskFlags: [],
                autonomyDelta: BackendAutonomyDelta(
                    signals: ["self_initiated", "commitment_completed"],
                    direction: "client_led"
                ),
                nextTouch: "scheduled"
            )
        )
    }
}
