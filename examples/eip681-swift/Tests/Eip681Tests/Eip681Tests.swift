import Testing

@testable import Eip681

@Test func parseNativeTransfer() throws {
    let uri = "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"
    let result = try Eip681.parserParse(uri)

    guard case .native(let native) = result else {
        Issue.record("Expected .native, got \(result)")
        return
    }

    #expect(native.recipientAddress == "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359")
    #expect(native.schemaPrefix == "ethereum")
    #expect(native.chainId == nil)
    #expect(native.valueAtomic != nil)
    #expect(native.gasLimit == nil)
    #expect(native.gasPrice == nil)
    #expect(!native.display.isEmpty)
}

@Test func parseErc20Transfer() throws {
    let uri =
        "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000"
    let result = try Eip681.parserParse(uri)

    guard case .erc20(let erc20) = result else {
        Issue.record("Expected .erc20, got \(result)")
        return
    }

    #expect(erc20.tokenContractAddress == "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    #expect(erc20.recipientAddress == "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359")
    #expect(!erc20.valueAtomic.isEmpty)
    #expect(!erc20.display.isEmpty)
}

@Test func parseInvalidInputThrows() {
    #expect(throws: WitFFIError.self) {
        try Eip681.parserParse("not-a-valid-uri")
    }
}

@Test func parseNativeTransferRoundTrips() throws {
    // Verify the display string can be re-parsed to the same result.
    let uri = "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"
    let result = try Eip681.parserParse(uri)

    guard case .native(let native) = result else {
        Issue.record("Expected .native, got \(result)")
        return
    }

    // Re-parse the display string
    let result2 = try Eip681.parserParse(native.display)

    guard case .native(let native2) = result2 else {
        Issue.record("Expected .native on re-parse, got \(result2)")
        return
    }

    #expect(native.recipientAddress == native2.recipientAddress)
    #expect(native.schemaPrefix == native2.schemaPrefix)
    #expect(native.chainId == native2.chainId)
    #expect(native.valueAtomic == native2.valueAtomic)
}
