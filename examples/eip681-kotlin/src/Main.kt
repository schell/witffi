// EIP-681 Kotlin/JNI demo — calls Rust via the generated JNI bridge.
//
// Run with:
//   make          (builds Rust + Kotlin, then runs)
//   make run      (just runs, assumes already built)

package zcash.eip681.demo

import zcash.eip681.Eip681
import zcash.eip681.TransactionRequest

fun main() {
    val eip681 = Eip681()

    println("=== EIP-681 Kotlin/JNI Demo ===")
    println()

    // ---- Native ETH transfer ----
    val nativeUri =
        "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"
    println("Parsing: $nativeUri")

    val native = eip681.parserParse(nativeUri)
    when (native) {
        is TransactionRequest.Native -> {
            val r = native.value
            println("  Type:      Native ETH transfer")
            println("  Schema:    ${r.schemaPrefix}")
            println("  To:        ${r.recipientAddress}")
            println("  Chain ID:  ${r.chainId ?: "mainnet (default)"}")
            println("  Value:     ${r.valueAtomic?.let { "${it.size} bytes (big-endian)" } ?: "none"}")
            println("  Display:   ${r.display}")
        }
        is TransactionRequest.Erc20 -> println("  (unexpected Erc20)")
        is TransactionRequest.Unrecognised -> println("  (unexpected Unrecognised)")
    }
    println()

    // ---- ERC-20 token transfer ----
    val erc20Uri =
        "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000"
    println("Parsing: $erc20Uri")

    val erc20 = eip681.parserParse(erc20Uri)
    when (erc20) {
        is TransactionRequest.Native -> println("  (unexpected Native)")
        is TransactionRequest.Erc20 -> {
            val r = erc20.value
            println("  Type:      ERC-20 token transfer")
            println("  Token:     ${r.tokenContractAddress}")
            println("  To:        ${r.recipientAddress}")
            println("  Chain ID:  ${r.chainId ?: "mainnet (default)"}")
            println("  Amount:    ${r.valueAtomic.size} bytes (big-endian)")
            println("  Display:   ${r.display}")
        }
        is TransactionRequest.Unrecognised -> println("  (unexpected Unrecognised)")
    }
    println()

    // ---- Invalid input (should throw RuntimeException) ----
    val badUri = "not-a-valid-uri"
    println("Parsing: $badUri")
    try {
        eip681.parserParse(badUri)
        println("  ERROR: expected an exception but got none")
    } catch (e: RuntimeException) {
        println("  Caught expected error: ${e.message}")
    }

    println()
    println("Done.")
}
