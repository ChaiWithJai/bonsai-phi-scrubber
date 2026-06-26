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
    case onDeviceMLXSwift = "on-device mlx-swift"

    public var label: String {
        switch self {
        case .mlxSwiftMock: "MLX Swift mock"
        case .edgeHTTPMock: "Edge HTTP mock"
        case .onDeviceMLXSwift: "On-device MLX Swift"
        }
    }

    public var isRunnableInSimulator: Bool {
        switch self {
        case .mlxSwiftMock, .edgeHTTPMock: true
        case .onDeviceMLXSwift: false
        }
    }

    public var detail: String {
        switch self {
        case .mlxSwiftMock:
            "Simulator stand-in for in-process Bonsai MLX text on iPhone 11 class hardware."
        case .edgeHTTPMock:
            "Simulator stand-in for the laptop `/api/scrub` JSON contract."
        case .onDeviceMLXSwift:
            "Hardware-gated target: real MLX text weights, client-side constraints, and iPhone 11/A13 measurement."
        }
    }
}

public struct BackendSelection: Codable, Equatable, Sendable {
    public var runtime: BackendRuntime
    public var deviceClass: String
    public var model: String
    public var constraints: [String]

    public init(
        runtime: BackendRuntime = .mlxSwiftMock,
        deviceClass: String = "iPhone 11 / A13 simulator budget",
        model: String = "ternary-bonsai-1.7b@mock-mlx",
        constraints: [String] = [
            "schema-compatible response",
            "client-side constrained decoding required for real MLX",
            "M3-T00 device measurement required"
        ]
    ) {
        self.runtime = runtime
        self.deviceClass = deviceClass
        self.model = model
        self.constraints = constraints
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

public struct InferenceSampling: Codable, Equatable, Sendable {
    public let temperature: Double
    public let topK: Int
    public let topP: Double
    public let maxTokens: Int
    public let seed: UInt64

    public init(
        temperature: Double = 0,
        topK: Int = 1,
        topP: Double = 1,
        maxTokens: Int = 512,
        seed: UInt64 = 11
    ) {
        self.temperature = temperature
        self.topK = topK
        self.topP = topP
        self.maxTokens = maxTokens
        self.seed = seed
    }
}

public struct TextInferenceRequest: Codable, Equatable, Sendable {
    public let system: String
    public let user: String
    public let jsonSchemaName: String
    public let jsonSchema: String
    public let sampling: InferenceSampling
    public let backend: BackendSelection

    public init(
        system: String,
        user: String,
        jsonSchemaName: String,
        jsonSchema: String,
        sampling: InferenceSampling = InferenceSampling(),
        backend: BackendSelection
    ) {
        self.system = system
        self.user = user
        self.jsonSchemaName = jsonSchemaName
        self.jsonSchema = jsonSchema
        self.sampling = sampling
        self.backend = backend
    }
}

public protocol TextInferenceProviding: Sendable {
    var runtime: BackendRuntime { get }
    func complete(_ request: TextInferenceRequest) throws -> String
}

public protocol AirplaneScrubBackend: Sendable {
    var runtime: BackendRuntime { get }
    func scrub(_ request: BackendScrubRequest) throws -> BackendScrubResponse
}

public struct SimulatorMLXSwiftTextInferenceProvider: TextInferenceProviding {
    public let runtime: BackendRuntime = .mlxSwiftMock

    public init() {}

    public func complete(_ request: TextInferenceRequest) throws -> String {
        """
        {"spans":[{"text":"Jordan Lee","entity":"PERSON"},{"text":"Maya","entity":"PERSON"},{"text":"COACH-4821","entity":"MEMBER_ID"},{"text":"March 12","entity":"DATE"}]}
        """
    }
}

public struct SimulatorEdgeHTTPTextInferenceProvider: TextInferenceProviding {
    public let runtime: BackendRuntime = .edgeHTTPMock

    public init() {}

    public func complete(_ request: TextInferenceRequest) throws -> String {
        """
        {"spans":[{"text":"Jordan Lee","entity":"PERSON"},{"text":"Maya","entity":"PERSON"},{"text":"COACH-4821","entity":"MEMBER_ID"},{"text":"March 12","entity":"DATE"}]}
        """
    }
}

public struct SimulatorMLXSwiftScrubBackend: AirplaneScrubBackend {
    public let runtime: BackendRuntime = .mlxSwiftMock
    private let inference: TextInferenceProviding

    public init(inference: TextInferenceProviding = SimulatorMLXSwiftTextInferenceProvider()) {
        self.inference = inference
    }

    public func scrub(_ request: BackendScrubRequest) throws -> BackendScrubResponse {
        try SimulatorBackend.scrub(request, inference: inference, layer: "mlx-swift-mock")
    }
}

public struct SimulatorEdgeHTTPScrubBackend: AirplaneScrubBackend {
    public let runtime: BackendRuntime = .edgeHTTPMock
    private let inference: TextInferenceProviding

    public init(inference: TextInferenceProviding = SimulatorEdgeHTTPTextInferenceProvider()) {
        self.inference = inference
    }

    public func scrub(_ request: BackendScrubRequest) throws -> BackendScrubResponse {
        try SimulatorBackend.scrub(request, inference: inference, layer: "edge-http-mock")
    }
}

public enum BackendSelectionError: Error, Equatable, Sendable {
    case hardwareRuntimeUnavailable(BackendRuntime)
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

    public var selectedBackendCanRunInSimulator: Bool {
        backend.runtime.isRunnableInSimulator
    }

    public mutating func reset() {
        let selected = backend
        self = AirplaneDemoFlow(backend: selected)
    }

    public mutating func selectBackend(_ runtime: BackendRuntime) {
        backend.runtime = runtime
        switch runtime {
        case .mlxSwiftMock:
            backend.model = "ternary-bonsai-1.7b@mock-mlx"
            backend.deviceClass = "iPhone 11 / A13 simulator budget"
        case .edgeHTTPMock:
            backend.model = "ternary-bonsai-1.7b@mock-edge-http"
            backend.deviceClass = "Mac edge node HTTP contract"
        case .onDeviceMLXSwift:
            backend.model = "ternary-bonsai-1.7b@mlx-text-unwired"
            backend.deviceClass = "physical iPhone 11 / A13 measurement gate"
        }
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

    public mutating func scrubAndGate(provider: AirplaneScrubBackend? = nil) {
        guard phase == .capturing else { return }

        phase = .scrubbing
        guard backend.runtime.isRunnableInSimulator else {
            gateResult = DemoGateResult(passed: false, residualCount: 1)
            phase = .gated
            return
        }

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
        record = response.gatePass ? response.record : nil
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

private func defaultProvider(for runtime: BackendRuntime) -> AirplaneScrubBackend {
    switch runtime {
    case .mlxSwiftMock:
        SimulatorMLXSwiftScrubBackend()
    case .edgeHTTPMock, .onDeviceMLXSwift:
        SimulatorEdgeHTTPScrubBackend()
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
    private struct InferenceSpan: Decodable {
        let text: String
        let entity: String
    }

    private struct InferenceSpanResponse: Decodable {
        let spans: [InferenceSpan]
    }

    private static let blockedTokens = [
        "Jordan Lee",
        "Maya",
        "COACH-4821",
        "March 12"
    ]

    static func scrub(
        _ request: BackendScrubRequest,
        inference: TextInferenceProviding,
        layer: String
    ) throws -> BackendScrubResponse {
        let rawCompletion = try inference.complete(
            TextInferenceRequest(
                system: "Return only JSON spans for identifiers in the user's synthetic coaching note.",
                user: request.text,
                jsonSchemaName: "airplane.identifier_spans",
                jsonSchema: #"{"type":"object","required":["spans"],"properties":{"spans":{"type":"array","items":{"type":"object","required":["text","entity"],"properties":{"text":{"type":"string"},"entity":{"type":"string"}}}}}}"#,
                backend: request.backend
            )
        )
        let spans = try JSONDecoder().decode(
            InferenceSpanResponse.self,
            from: Data(rawCompletion.utf8)
        ).spans

        var output = request.text
        var redactions: [BackendRedaction] = []

        for span in spans where output.contains(span.text) {
            output = output.replacingOccurrences(of: span.text, with: "[\(span.entity)]")
            redactions.append(BackendRedaction(entity: span.entity, layer: layer))
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
