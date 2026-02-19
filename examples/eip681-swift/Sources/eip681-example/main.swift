// EIP-681 Swift Example
//
// Demonstrates using the witffi-generated Swift bindings to parse
// EIP-681 Ethereum transaction request URIs.
//
// Run with: swift run eip681-example

import Eip681

// ---- Native ETH transfer ----

let nativeUri = "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"

print("Parsing native transfer:")
print("  URI: \(nativeUri)")

do {
    let request = try Eip681.parserParse(nativeUri)

    switch request {
    case .native(let native):
        print("  Result: Native ETH transfer")
        print("    Schema:    \(native.schemaPrefix)")
        print("    Recipient: \(native.recipientAddress)")
        if let chainId = native.chainId {
            print("    Chain ID:  \(chainId)")
        } else {
            print("    Chain ID:  (default / mainnet)")
        }
        if let value = native.valueAtomic {
            print("    Value:     \(value.count) bytes (big-endian u256)")
        }
        print("    Display:   \(native.display)")

    case .erc20(let erc20):
        print("  Unexpected: ERC-20 (\(erc20.tokenContractAddress))")

    case .unrecognised(let raw):
        print("  Unexpected: Unrecognised (\(raw))")
    }
} catch {
    print("  Error: \(error)")
}

print()

// ---- ERC-20 token transfer ----

let erc20Uri = "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000"

print("Parsing ERC-20 transfer:")
print("  URI: \(erc20Uri)")

do {
    let request = try Eip681.parserParse(erc20Uri)

    switch request {
    case .erc20(let erc20):
        print("  Result: ERC-20 token transfer")
        print("    Token:     \(erc20.tokenContractAddress)")
        print("    Recipient: \(erc20.recipientAddress)")
        if let chainId = erc20.chainId {
            print("    Chain ID:  \(chainId)")
        }
        print("    Value:     \(erc20.valueAtomic.count) bytes (big-endian u256)")
        print("    Display:   \(erc20.display)")

    case .native(let native):
        print("  Unexpected: Native (\(native.recipientAddress))")

    case .unrecognised(let raw):
        print("  Unexpected: Unrecognised (\(raw))")
    }
} catch {
    print("  Error: \(error)")
}

print()

// ---- Error handling ----

let invalidUri = "not-a-valid-uri"

print("Parsing invalid input:")
print("  URI: \(invalidUri)")

do {
    _ = try Eip681.parserParse(invalidUri)
    print("  Unexpected: parsing succeeded")
} catch {
    print("  Error (expected): \(error)")
}
