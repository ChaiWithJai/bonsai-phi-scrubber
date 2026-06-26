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

public struct DemoRedaction: Equatable, Sendable {
    public let entity: String
    public let replacement: String
}

public struct DemoGateResult: Equatable, Sendable {
    public let passed: Bool
    public let residualCount: Int
}

public struct DemoRecord: Equatable, Sendable {
    public let summary: String
    public let followUp: String
}

public struct DemoSecureStoreSnapshot: Equatable, Sendable {
    public let rawNoteStored: Bool
    public let redactionMapStored: Bool
    public let redactionCount: Int
}

public struct AirplaneDemoFlow: Equatable, Sendable {
    public static let sampleNote = """
    Jordan Lee, member COACH-4821, met with coach Maya on March 12. Jordan wants to practice \
    a five minute breathing routine before Monday standup and asked not to share the note until \
    identifiers are removed.
    """

    public private(set) var phase: DemoPhase = .idle
    public private(set) var capturedText: String = ""
    public private(set) var scrubbedText: String = ""
    public private(set) var redactions: [DemoRedaction] = []
    public private(set) var gateResult: DemoGateResult?
    public private(set) var record: DemoRecord?
    public private(set) var deliveredPayload: DemoRecord?

    private var secureStore = SimulatorOnlySecureStore()

    public init() {}

    public var secureStoreSnapshot: DemoSecureStoreSnapshot {
        secureStore.snapshot
    }

    public mutating func reset() {
        self = AirplaneDemoFlow()
    }

    public mutating func capture(_ note: String = AirplaneDemoFlow.sampleNote) {
        capturedText = note
        scrubbedText = ""
        redactions = []
        gateResult = nil
        record = nil
        deliveredPayload = nil
        secureStore.save(rawNote: note, redactions: [])
        phase = .capturing
    }

    public mutating func scrubAndGate() {
        guard phase == .capturing else { return }

        phase = .scrubbing
        let result = SimulatorRedactor.scrub(capturedText)
        scrubbedText = result.text
        redactions = result.redactions
        secureStore.save(rawNote: capturedText, redactions: redactions)

        let residualCount = SimulatorVerifier.residualIdentifierCount(in: scrubbedText)
        gateResult = DemoGateResult(passed: residualCount == 0, residualCount: residualCount)
        phase = .gated
    }

    public mutating func structureCleanRecord() {
        guard phase == .gated, gateResult?.passed == true else { return }

        record = DemoRecord(
            summary: "Client practiced an autonomy-supportive coaching plan with identifiers removed.",
            followUp: "Practice a five minute breathing routine before the next work standup."
        )
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

private struct SimulatorOnlySecureStore: Equatable, Sendable {
    private var rawNote: String?
    private var redactionMap: [DemoRedaction] = []

    var snapshot: DemoSecureStoreSnapshot {
        DemoSecureStoreSnapshot(
            rawNoteStored: rawNote != nil,
            redactionMapStored: !redactionMap.isEmpty,
            redactionCount: redactionMap.count
        )
    }

    mutating func save(rawNote: String, redactions: [DemoRedaction]) {
        self.rawNote = rawNote
        self.redactionMap = redactions
    }
}

private enum SimulatorRedactor {
    private static let replacements: [(needle: String, entity: String, replacement: String)] = [
        ("Jordan Lee", "PERSON", "[PERSON]"),
        ("Maya", "PERSON", "[PERSON]"),
        ("COACH-4821", "MEMBER_ID", "[MEMBER_ID]"),
        ("March 12", "DATE", "[DATE]")
    ]

    static func scrub(_ text: String) -> (text: String, redactions: [DemoRedaction]) {
        var output = text
        var redactions: [DemoRedaction] = []

        for item in replacements where output.contains(item.needle) {
            output = output.replacingOccurrences(of: item.needle, with: item.replacement)
            redactions.append(DemoRedaction(entity: item.entity, replacement: item.replacement))
        }

        return (output, redactions)
    }
}

private enum SimulatorVerifier {
    private static let blockedTokens = [
        "Jordan Lee",
        "Maya",
        "COACH-4821",
        "March 12"
    ]

    static func residualIdentifierCount(in text: String) -> Int {
        blockedTokens.reduce(0) { count, token in
            text.localizedCaseInsensitiveContains(token) ? count + 1 : count
        }
    }
}
