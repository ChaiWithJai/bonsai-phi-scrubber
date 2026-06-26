import Foundation

#if canImport(SwiftUI)
import SwiftUI

@MainActor
public final class AirplaneDemoViewModel: ObservableObject {
    @Published public private(set) var flow = AirplaneDemoFlow()

    public init() {}

    public func reset() {
        flow.reset()
    }

    public func selectBackend(_ runtime: BackendRuntime) {
        flow.selectBackend(runtime)
    }

    public func advance() {
        switch flow.phase {
        case .idle:
            flow.capture()
        case .capturing:
            flow.scrubAndGate()
        case .scrubbing:
            break
        case .gated:
            flow.structureCleanRecord()
        case .structured:
            flow.holdSendWhileOffline()
        case .sendHeld:
            flow.reconnectAndFlush()
        case .flushing, .delivered:
            flow.reset()
        }
    }
}

public struct AirplaneDemoView: View {
    @StateObject private var model = AirplaneDemoViewModel()

    public init() {}

    public var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 18) {
                phaseHeader
                backendSelector
                content
                Spacer(minLength: 0)
                controls
            }
            .padding()
            .navigationTitle("Airplane Mode")
        }
    }

    private var phaseHeader: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(model.flow.phase.rawValue)
                .font(.largeTitle.bold())
            Text("Simulator scaffold: UI state flow only. No mlx-swift inference, Secure Enclave storage, radio proof, or R1 device measurement is claimed here.")
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
    }

    private var backendSelector: some View {
        VStack(alignment: .leading, spacing: 8) {
            Picker("Backend", selection: Binding(
                get: { model.flow.backend.runtime },
                set: { model.selectBackend($0) }
            )) {
                ForEach(BackendRuntime.allCases, id: \.self) { runtime in
                    Text(runtime.isRunnableInSimulator ? runtime.label : "\(runtime.label) locked")
                        .tag(runtime)
                }
            }
            .pickerStyle(.segmented)

            Text(model.flow.backend.runtime.detail)
                .font(.caption)
                .foregroundStyle(.secondary)
            Text("Profile: \(model.flow.backend.deviceClass)")
                .font(.caption.monospaced())
                .foregroundStyle(.secondary)
        }
        .disabled(model.flow.phase != .idle)
    }

    @ViewBuilder
    private var content: some View {
        switch model.flow.phase {
        case .idle:
            Text("Ready to simulate capture of a synthetic coaching note.")
        case .capturing:
            labeledBlock("Captured synthetic note", model.flow.capturedText)
        case .scrubbing, .gated:
            VStack(alignment: .leading, spacing: 12) {
                labeledBlock("De-identified note", model.flow.scrubbedText)
                if let response = model.flow.lastResponse {
                    Text("Backend: \(model.flow.backend.runtime.label) · \(response.redactions.count) redactions · schema-compatible response")
                        .font(.footnote.monospaced())
                        .foregroundStyle(.secondary)
                }
                if let gate = model.flow.gateResult {
                    Label(
                        gate.passed ? "Verifier passed: zero residual simulated identifiers" : "Verifier blocked",
                        systemImage: gate.passed ? "checkmark.seal" : "xmark.octagon"
                    )
                    .foregroundStyle(gate.passed ? .green : .red)
                }
                Text("Simulator store contains raw note: \(model.flow.secureStoreSnapshot.rawNoteStored ? "yes" : "no"); redaction map entries: \(model.flow.secureStoreSnapshot.redactionCount)")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
        case .structured, .sendHeld, .flushing, .delivered:
            VStack(alignment: .leading, spacing: 12) {
                if let record = model.flow.record {
                    labeledBlock("Structured record", recordSummary(record))
                }
                if model.flow.phase == .sendHeld {
                    Label("Send held while offline", systemImage: "airplane")
                        .foregroundStyle(.orange)
                }
                if model.flow.deliveredPayload != nil {
                    Label("Delivered simulated clean payload", systemImage: "paperplane.fill")
                        .foregroundStyle(.green)
                }
            }
        }
    }

    private func recordSummary(_ record: BackendRecord) -> String {
        let themes = record.themes.joined(separator: " · ")
        let commitments = record.commitments
            .map { "\($0.text) · \($0.status)" }
            .joined(separator: "\n")
        let followUps = record.followUps.joined(separator: "\n")
        let risks = record.riskFlags.isEmpty ? "none" : record.riskFlags.joined(separator: " · ")
        return """
        client_pseudonym: \(record.clientPseudonym)
        themes: \(themes)
        commitments: \(commitments)
        follow_ups: \(followUps)
        risk_flags: \(risks)
        next_touch: \(record.nextTouch)
        """
    }

    private var controls: some View {
        HStack {
            Button(action: model.advance) {
                Label(primaryActionTitle, systemImage: primaryActionIcon)
            }
            .buttonStyle(.borderedProminent)
            .disabled(model.flow.phase == .idle && !model.flow.selectedBackendCanRunInSimulator)

            Button(action: model.reset) {
                Label("Reset", systemImage: "arrow.counterclockwise")
            }
            .buttonStyle(.bordered)
        }
    }

    private var primaryActionTitle: String {
        switch model.flow.phase {
        case .idle: model.flow.selectedBackendCanRunInSimulator ? "Capture" : "Hardware gated"
        case .capturing: "Scrub"
        case .scrubbing: "Wait"
        case .gated: "Structure"
        case .structured: "Hold Send"
        case .sendHeld: "Flush"
        case .flushing, .delivered: "Restart"
        }
    }

    private var primaryActionIcon: String {
        switch model.flow.phase {
        case .idle: "mic"
        case .capturing: "wand.and.stars"
        case .scrubbing: "hourglass"
        case .gated: "checkmark.shield"
        case .structured: "pause.circle"
        case .sendHeld: "wifi"
        case .flushing, .delivered: "arrow.clockwise"
        }
    }

    private func labeledBlock(_ title: String, _ text: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.headline)
            Text(text)
                .font(.body.monospaced())
                .textSelection(.enabled)
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(.thinMaterial)
                .clipShape(RoundedRectangle(cornerRadius: 8))
        }
    }
}

#if DEBUG
#Preview {
    AirplaneDemoView()
}
#endif
#endif
