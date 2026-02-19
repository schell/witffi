// swift-tools-version: 6.0
//
// EIP-681 Swift Example — demonstrates consuming a witffi-generated FFI library.
//
// IMPORTANT: Before building, run `./build.sh` to:
//   1. Build the Rust FFI library (cargo build --release -p eip681-ffi)
//   2. Generate the Swift bindings (witffi generate --lang swift)
//   3. Copy the C headers into Sources/CZcashEip681/include/

import PackageDescription

let package = Package(
    name: "Eip681Swift",
    targets: [
        // C target wrapping the Rust FFI headers.
        // The headers (ffi.h, witffi_types.h) are copied here by build.sh.
        .target(
            name: "CZcashEip681",
            path: "Sources/CZcashEip681",
            publicHeadersPath: "include"
        ),

        // Swift library containing the generated witffi bindings.
        // Links against the Rust static library built by cargo.
        .target(
            name: "Eip681",
            dependencies: ["CZcashEip681"],
            path: "Sources/Eip681",
            linkerSettings: [
                .unsafeFlags([
                    "-L../../target/release",
                    "-leip681_ffi",
                ]),
            ]
        ),

        // Example CLI that parses EIP-681 URIs.
        .executableTarget(
            name: "eip681-example",
            dependencies: ["Eip681"],
            path: "Sources/eip681-example"
        ),

        // Tests exercising the Swift bindings.
        .testTarget(
            name: "Eip681Tests",
            dependencies: ["Eip681"]
        ),
    ]
)
