//! Name conversion utilities for mapping WIT identifiers to language conventions.
//!
//! WIT uses kebab-case (e.g. `transaction-request`). Each target language
//! has its own naming convention:
//!
//! - Rust types: `PascalCase` (e.g. `TransactionRequest`)
//! - Rust functions/fields: `snake_case` (e.g. `transaction_request`)
//! - C types: `PascalCase` with prefix (e.g. `FfiTransactionRequest`)
//! - C functions: `snake_case` with prefix (e.g. `ffi_transaction_request`)
//! - Swift types: `PascalCase` (e.g. `TransactionRequest`)
//! - Swift functions/properties: `camelCase` (e.g. `transactionRequest`)
//! - Kotlin types: `PascalCase` (e.g. `TransactionRequest`)
//! - Kotlin functions/properties: `camelCase` (e.g. `transactionRequest`)
//! - Go types: `PascalCase` (exported) (e.g. `TransactionRequest`)
//! - Go functions: `PascalCase` (exported) (e.g. `TransactionRequest`)

use heck::{ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase, ToSnakeCase};

/// Convert a WIT kebab-case identifier to Rust PascalCase (for types).
pub fn to_rust_type(name: &str) -> String {
    name.to_pascal_case()
}

/// Convert a WIT kebab-case identifier to Rust snake_case (for functions, fields).
pub fn to_rust_ident(name: &str) -> String {
    let snake = name.to_snake_case();
    escape_rust_keyword(&snake)
}

/// Convert a WIT kebab-case identifier to C PascalCase with a prefix.
pub fn to_c_type(prefix: &str, name: &str) -> String {
    format!("{}{}", prefix, name.to_pascal_case())
}

/// Convert a WIT kebab-case identifier to C snake_case with a prefix.
pub fn to_c_func(prefix: &str, name: &str) -> String {
    format!("{}_{}", prefix.to_snake_case(), name.to_snake_case())
}

/// Convert a WIT kebab-case identifier to C SHOUTY_SNAKE_CASE (for enum variants/constants).
pub fn to_c_enum_variant(prefix: &str, name: &str) -> String {
    format!(
        "{}_{}",
        prefix.to_shouty_snake_case(),
        name.to_shouty_snake_case()
    )
}

/// Convert a WIT kebab-case identifier to Swift PascalCase (for types).
pub fn to_swift_type(name: &str) -> String {
    name.to_pascal_case()
}

/// Convert a WIT kebab-case identifier to Swift camelCase (for functions, properties).
pub fn to_swift_ident(name: &str) -> String {
    let camel = name.to_lower_camel_case();
    escape_swift_keyword(&camel)
}

/// Convert a WIT kebab-case identifier to Go PascalCase (exported).
pub fn to_go_type(name: &str) -> String {
    name.to_pascal_case()
}

/// Convert a WIT kebab-case identifier to Go PascalCase (exported functions).
pub fn to_go_func(name: &str) -> String {
    name.to_pascal_case()
}

/// Convert a WIT kebab-case identifier to Go camelCase (unexported).
pub fn to_go_field(name: &str) -> String {
    name.to_pascal_case()
}

/// Escape Rust reserved keywords by appending `_`.
fn escape_rust_keyword(name: &str) -> String {
    match name {
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move"
        | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await"
        | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" => {
            format!("{name}_")
        }
        _ => name.to_string(),
    }
}

/// Escape Swift reserved keywords by wrapping in backticks.
fn escape_swift_keyword(name: &str) -> String {
    match name {
        "associatedtype" | "class" | "deinit" | "enum" | "extension" | "fileprivate" | "func"
        | "import" | "init" | "inout" | "internal" | "let" | "open" | "operator" | "private"
        | "protocol" | "public" | "rethrows" | "static" | "struct" | "subscript" | "super"
        | "typealias" | "var" | "break" | "case" | "continue" | "default" | "defer" | "do"
        | "else" | "fallthrough" | "for" | "guard" | "if" | "in" | "repeat" | "return"
        | "switch" | "where" | "while" | "as" | "catch" | "false" | "is" | "nil" | "self"
        | "Self" | "throw" | "throws" | "true" | "try" | "async" | "await" => {
            format!("`{name}`")
        }
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_type_names() {
        assert_eq!(to_rust_type("transaction-request"), "TransactionRequest");
        assert_eq!(to_rust_type("native-request"), "NativeRequest");
        assert_eq!(to_rust_type("u256"), "U256");
        assert_eq!(to_rust_type("eip681"), "Eip681");
    }

    #[test]
    fn test_rust_ident_names() {
        assert_eq!(to_rust_ident("chain-id"), "chain_id");
        assert_eq!(to_rust_ident("recipient-address"), "recipient_address");
        assert_eq!(to_rust_ident("value-atomic"), "value_atomic");
        assert_eq!(to_rust_ident("type"), "type_");
    }

    #[test]
    fn test_c_names() {
        assert_eq!(
            to_c_type("Ffi", "transaction-request"),
            "FfiTransactionRequest"
        );
        assert_eq!(to_c_func("zcash_eip681", "parse"), "zcash_eip681_parse");
        assert_eq!(
            to_c_enum_variant("TRANSACTION_REQUEST", "native"),
            "TRANSACTION_REQUEST_NATIVE"
        );
    }

    #[test]
    fn test_swift_names() {
        assert_eq!(to_swift_type("transaction-request"), "TransactionRequest");
        assert_eq!(to_swift_ident("chain-id"), "chainId");
        assert_eq!(to_swift_ident("self"), "`self`");
    }

    #[test]
    fn test_go_names() {
        assert_eq!(to_go_type("transaction-request"), "TransactionRequest");
        assert_eq!(to_go_func("parse"), "Parse");
    }
}
