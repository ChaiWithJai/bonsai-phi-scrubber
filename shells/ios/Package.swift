// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "AirplaneIOSShell",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "AirplaneIOSShell",
            targets: ["AirplaneIOSShell"]
        )
    ],
    targets: [
        .target(
            name: "AirplaneIOSShell"
        ),
        .testTarget(
            name: "AirplaneIOSShellTests",
            dependencies: ["AirplaneIOSShell"]
        )
    ]
)
