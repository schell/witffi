// EIP-681 Go/CGo demo — calls Rust via the generated CGo bindings.
//
// Run with:
//
//	make          (builds Rust + Go, then runs)
//	make run      (just runs, assumes already built)
package main

import (
	"fmt"
	"os"

	eip681 "github.com/schell/witffi/examples/eip681-go"
)

func main() {
	fmt.Println("=== EIP-681 Go/CGo Demo ===")
	fmt.Println()

	// ---- Native ETH transfer ----

	nativeURI := "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"
	fmt.Printf("Parsing: %s\n", nativeURI)

	result, err := eip681.ParserParse(nativeURI)
	if err != nil {
		fmt.Fprintf(os.Stderr, "  Error: %v\n", err)
		os.Exit(1)
	}

	switch v := result.(type) {
	case eip681.TransactionRequestNative:
		r := v.Value
		fmt.Println("  Type:      Native ETH transfer")
		fmt.Printf("  Schema:    %s\n", r.SchemaPrefix)
		fmt.Printf("  To:        %s\n", r.RecipientAddress)
		if r.ChainId != nil {
			fmt.Printf("  Chain ID:  %d\n", *r.ChainId)
		} else {
			fmt.Println("  Chain ID:  mainnet (default)")
		}
		if r.ValueAtomic != nil {
			fmt.Printf("  Value:     %d bytes (big-endian u256)\n", len(r.ValueAtomic))
		}
		fmt.Printf("  Display:   %s\n", r.Display)
	default:
		fmt.Printf("  Unexpected variant: %T\n", result)
	}
	fmt.Println()

	// ---- ERC-20 token transfer ----

	erc20URI := "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000"
	fmt.Printf("Parsing: %s\n", erc20URI)

	result2, err := eip681.ParserParse(erc20URI)
	if err != nil {
		fmt.Fprintf(os.Stderr, "  Error: %v\n", err)
		os.Exit(1)
	}

	switch v := result2.(type) {
	case eip681.TransactionRequestErc20:
		r := v.Value
		fmt.Println("  Type:      ERC-20 token transfer")
		fmt.Printf("  Token:     %s\n", r.TokenContractAddress)
		fmt.Printf("  To:        %s\n", r.RecipientAddress)
		if r.ChainId != nil {
			fmt.Printf("  Chain ID:  %d\n", *r.ChainId)
		} else {
			fmt.Println("  Chain ID:  mainnet (default)")
		}
		fmt.Printf("  Amount:    %d bytes (big-endian u256)\n", len(r.ValueAtomic))
		fmt.Printf("  Display:   %s\n", r.Display)
	default:
		fmt.Printf("  Unexpected variant: %T\n", result2)
	}
	fmt.Println()

	// ---- u256-to-string ----

	u256 := make([]byte, 32)
	u256[31] = 42 // big-endian representation of 42
	fmt.Printf("U256ToString(%v):\n", u256)

	s := eip681.FunctionsU256ToString(u256)
	fmt.Printf("  Result: %s\n", s)
	fmt.Println()

	// ---- Error handling ----

	badURI := "not-a-valid-uri"
	fmt.Printf("Parsing: %s\n", badURI)

	_, err = eip681.ParserParse(badURI)
	if err != nil {
		fmt.Printf("  Caught expected error: %v\n", err)
	} else {
		fmt.Println("  ERROR: expected an error but got none")
		os.Exit(1)
	}

	fmt.Println()
	fmt.Println("Done.")
}
