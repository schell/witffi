package eip681

import (
	"testing"
)

func TestParseNativeTransfer(t *testing.T) {
	uri := "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"
	result, err := ParserParse(uri)
	if err != nil {
		t.Fatalf("ParserParse failed: %v", err)
	}

	native, ok := result.(TransactionRequestNative)
	if !ok {
		t.Fatalf("expected TransactionRequestNative, got %T", result)
	}

	r := native.Value
	if r.RecipientAddress != "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359" {
		t.Errorf("recipient = %q, want %q", r.RecipientAddress, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359")
	}
	if r.SchemaPrefix != "ethereum" {
		t.Errorf("schema = %q, want %q", r.SchemaPrefix, "ethereum")
	}
	if r.ChainId != nil {
		t.Errorf("chainId = %v, want nil", *r.ChainId)
	}
	if r.ValueAtomic == nil {
		t.Error("valueAtomic is nil, want non-nil")
	}
	if r.GasLimit != nil {
		t.Errorf("gasLimit = %v, want nil", r.GasLimit)
	}
	if r.GasPrice != nil {
		t.Errorf("gasPrice = %v, want nil", r.GasPrice)
	}
	if r.Display == "" {
		t.Error("display is empty")
	}
}

func TestParseErc20Transfer(t *testing.T) {
	uri := "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000"
	result, err := ParserParse(uri)
	if err != nil {
		t.Fatalf("ParserParse failed: %v", err)
	}

	erc20, ok := result.(TransactionRequestErc20)
	if !ok {
		t.Fatalf("expected TransactionRequestErc20, got %T", result)
	}

	r := erc20.Value
	if r.TokenContractAddress != "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" {
		t.Errorf("token = %q, want %q", r.TokenContractAddress, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
	}
	if r.RecipientAddress != "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359" {
		t.Errorf("recipient = %q, want %q", r.RecipientAddress, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359")
	}
	if len(r.ValueAtomic) == 0 {
		t.Error("valueAtomic is empty")
	}
	if r.Display == "" {
		t.Error("display is empty")
	}
}

func TestParseInvalidInput(t *testing.T) {
	_, err := ParserParse("not-a-valid-uri")
	if err == nil {
		t.Fatal("expected error for invalid input, got nil")
	}
}

func TestParseNativeRoundTrip(t *testing.T) {
	uri := "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000"
	result, err := ParserParse(uri)
	if err != nil {
		t.Fatalf("first parse failed: %v", err)
	}

	native, ok := result.(TransactionRequestNative)
	if !ok {
		t.Fatalf("expected TransactionRequestNative, got %T", result)
	}

	// Re-parse the display string
	result2, err := ParserParse(native.Value.Display)
	if err != nil {
		t.Fatalf("round-trip parse failed: %v", err)
	}

	native2, ok := result2.(TransactionRequestNative)
	if !ok {
		t.Fatalf("round-trip: expected TransactionRequestNative, got %T", result2)
	}

	if native.Value.RecipientAddress != native2.Value.RecipientAddress {
		t.Errorf("recipient mismatch: %q vs %q", native.Value.RecipientAddress, native2.Value.RecipientAddress)
	}
	if native.Value.SchemaPrefix != native2.Value.SchemaPrefix {
		t.Errorf("schema mismatch: %q vs %q", native.Value.SchemaPrefix, native2.Value.SchemaPrefix)
	}
}
