# Key Field Validation Implementation Summary

This document summarizes the key field validation constraints implemented for the NetabaseSchema derive macro.

## Implemented Constraints

### 1. Struct Key Field Limits
- **Rule**: Structs can have **at most 1 key field**
- **Validation**: Compile-time error if more than 1 field has `#[key]` attribute
- **Error Message**: "Schema 'StructName' has multiple key fields. Structs can have at most 1 key field."

### 2. Enum Variant Key Field Requirements  
- **Rule**: Each enum variant must have **exactly 1 key field** (unless enum has top-level closure)
- **Validation**: Compile-time error if variant has 0 or >1 key fields
- **Error Messages**:
  - No key: "Enum variant 'EnumName::VariantName' has no key fields. Each enum variant must have exactly one field marked with #[key] or the enum must have a top-level #[key = closure] attribute."
  - Multiple keys: "Enum variant 'EnumName::VariantName' has multiple key fields. Enum variants can have at most 1 key field."

### 3. Enum Top-Level Key Closure Support
- **Rule**: Enums can have a top-level `#[key = |item| ...]` closure instead of per-variant keys
- **Behavior**: When enum has top-level closure, individual variant key fields are not required
- **Validation**: If enum has top-level closure, variants with individual key fields generate warnings (but still compile)

## Implementation Approach

### Direct Macro Validation
The validation is implemented directly in the `netabase_schema` derive macro using:
- Custom helper functions (`count_key_fields`, `has_key_attribute`)
- Compile-time error generation using `compile_error!` macro
- Early return with error tokens when validation fails

### Validation Flow
1. Parse the input item (struct or enum)
2. Count key fields in struct/enum variants
3. Check constraints based on item type
4. Generate `compile_error!` tokens if constraints violated
5. Continue with normal macro expansion if validation passes

## Valid Usage Examples

### Valid Struct (1 key field)
```rust
#[derive(NetabaseSchema)]
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
}
```

### Valid Enum (1 key per variant)
```rust
#[derive(NetabaseSchema)]
enum UserType {
    Regular {
        #[key]
        user_id: String,
        name: String,
    },
    Admin {
        #[key]
        admin_id: String,
        permissions: Vec<String>,
    },
}
```

### Valid Enum (top-level closure)
```rust
#[derive(NetabaseSchema)]
#[key = |item| match item {
    UserType::Regular { name, .. } => format!("user_{}", name),
    UserType::Admin { admin_name, .. } => format!("admin_{}", admin_name),
}]
enum UserType {
    Regular {
        name: String,
        email: String,
    },
    Admin {
        admin_name: String,
        permissions: Vec<String>,
    },
}
```

### Valid Special Cases
- **Unit enum variants**: No key field required for unit variants
- **Tuple structs**: Key fields work with tuple structs: `struct Data(#[key] String, i32);`

## Invalid Usage Examples (Compilation Errors)

### Invalid Struct (multiple keys)
```rust
#[derive(NetabaseSchema)]
struct User {
    #[key]
    id: String,
    #[key]
    email: String,  // ERROR: Multiple key fields
    name: String,
}
```

### Invalid Enum Variant (multiple keys)
```rust
#[derive(NetabaseSchema)]
enum UserType {
    User {
        #[key]
        id: String,
        #[key]
        email: String,  // ERROR: Multiple key fields in variant
        name: String,
    },
}
```

### Invalid Enum Variant (no keys)
```rust
#[derive(NetabaseSchema)]
enum UserType {
    User {
        id: String,     // ERROR: No key field
        name: String,
    },
}
```

## Key Features

### Compile-Time Safety
- All validation happens at compile time
- Clear, descriptive error messages
- No runtime performance impact

### Flexible Key Generation
- Simple field keys: `#[key]`
- Field-level closures: `#[key = |field_value| ...]`
- Item-level closures: `#[key = |item| ...]` (for enums)

### Backward Compatibility
- Existing valid schemas continue to work
- Only adds new validation constraints
- Does not change generated code for valid schemas

## Testing

The validation constraints are tested using:
- **Valid case tests**: Ensure valid schemas compile and work correctly
- **Invalid case tests**: Ensure invalid schemas fail compilation with appropriate error messages
- **Edge case tests**: Unit variants, tuple structs, mixed scenarios

Test files:
- `test_validation/src/simple_tests.rs` - Basic valid usage
- `test_validation/src/invalid_tests.rs` - Invalid usage examples (commented out)
- `test_validation/src/test_invalid_struct.rs` - Specific invalid case for testing

## Future Enhancements

Potential future improvements:
1. **Custom error spans**: Point to specific fields/attributes in error messages
2. **Enhanced closure validation**: Validate closure signatures and return types
3. **Key field type validation**: Ensure key fields implement required traits
4. **Advanced enum support**: Support for more complex enum key generation patterns