# Key Generation Feature Documentation

## Overview

The NetabaseSchema macro system now supports flexible key generation for database schemas. Users can define how keys are generated for their structs and enums using two main approaches:

1. **Field Keys**: Mark specific fields with `#[key]` to use them as keys
2. **Function Keys**: Use `#[key_fn = "function_name"]` to specify a custom key generation function

## Field Keys

### Basic Usage

Mark a field with `#[key]` to use it as the schema's key:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
pub struct User {
    #[key]
    pub id: u64,
    pub name: String,
    pub email: String,
}
```

### Supported Field Types

Field keys support any type that implements `Encode` and `Decode` from the `bincode` crate:

- Primitive types: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `f32`, `f64`, `bool`
- Strings: `String`, `&str`
- Arrays: `[T; N]` where `T` is encodable
- Tuples: `(T1, T2, ...)` where all elements are encodable
- Collections: `Vec<T>`, `Option<T>`, etc.

### Enum Field Keys

For enums, you can mark fields in variants:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
pub enum EntityEvent {
    Created { #[key] id: u64, name: String },
    Updated { #[key] id: u64, changes: Vec<String> },
    Deleted { #[key] id: u64, reason: String },
}
```

### Tuple Variant Keys

For tuple variants, mark the key field with `#[key]`:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
pub enum Notification {
    Message(#[key] u64, String, u64),    // (message_id, content, timestamp)
    Mention(#[key] u64, u64, String, u64), // (mention_id, user_id, content, timestamp)
    Reaction(#[key] u64, u64, String, u64), // (reaction_id, message_id, emoji, timestamp)
}
```

## Function Keys

### Basic Usage

Use the `#[key_fn = "function_name"]` attribute to specify a custom key generation function:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
#[key_fn = "user_key"]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

fn user_key(user: &User) -> u64 {
    user.id
}
```

### Function Signature Requirements

Key generation functions must:

1. Accept a single parameter that is a reference to the schema type (`&YourStruct` or `&YourEnum`)
2. Return a type that implements `Encode` and `Decode`
3. Be accessible in the same scope as the schema definition

### Advanced Examples

#### Composite Keys

Generate keys from multiple fields:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
#[key_fn = "user_post_key"]
pub struct UserPost {
    pub user_id: u64,
    pub post_id: u64,
    pub title: String,
    pub content: String,
}

fn user_post_key(post: &UserPost) -> (u64, u64) {
    (post.user_id, post.post_id)
}
```

#### Hash-based Keys

Generate keys using hashing:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
#[key_fn = "document_key"]
pub struct Document {
    pub content: String,
    pub author: String,
    pub created_at: u64,
}

fn document_key(doc: &Document) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    doc.content.hash(&mut hasher);
    hasher.finish().to_be_bytes().to_vec()
}
```

#### Enum Key Functions

Handle multiple enum variants:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
#[key_fn = "entity_event_key"]
pub enum EntityEvent {
    Created { id: u64, name: String },
    Updated { id: u64, changes: Vec<String> },
    Deleted { id: u64, reason: String },
}

fn entity_event_key(event: &EntityEvent) -> u64 {
    match event {
        EntityEvent::Created { id, .. } => *id,
        EntityEvent::Updated { id, .. } => *id,
        EntityEvent::Deleted { id, .. } => *id,
    }
}
```

#### String-based Keys

Generate human-readable keys:

```rust
#[derive(Clone, Encode, Decode, NetabaseSchema)]
#[key_fn = "transaction_key"]
pub struct Transaction {
    pub account_id: u64,
    pub amount: i64,
    pub timestamp: u64,
    pub description: String,
}

fn transaction_key(tx: &Transaction) -> String {
    format!("{}:{}:{}", tx.account_id, tx.timestamp, tx.amount)
}
```

## Validation Rules

### General Rules

1. **Mutual Exclusivity**: You cannot use both field keys (`#[key]`) and function keys (`#[key_fn]`) on the same schema
2. **Single Key Field**: Only one field per struct/variant can be marked with `#[key]`
3. **Required Keys**: Every schema must have either field keys or a function key

### Function Key Validation

1. **Function Name**: Must be a valid Rust identifier (letters, numbers, underscores, starting with letter or underscore)
2. **Parameter Type**: Function must accept the correct schema type as a reference
3. **Return Type**: Must be encodable/decodable with bincode

### Error Messages

The macro provides helpful error messages for common issues:

- `"Schema key and field keys are mutually exclusive"`
- `"Only one field can be marked with #[key] per struct/variant"`
- `"Schema must have at least one key (either function key or field key)"`
- `"Key function name must start with a letter or underscore"`
- `"Function parameter type must match schema type"`

## Generated Code

### get_key() Method

The macro generates a `get_key()` method for each schema that returns `Vec<u8>`:

```rust
impl User {
    pub fn get_key(&self) -> Vec<u8> {
        // For field keys: encode the field value
        bincode::encode_to_vec(&self.id, bincode::config::standard())
            .unwrap_or_else(|_| vec![])
    }
}

impl UserPost {
    pub fn get_key(&self) -> Vec<u8> {
        // For function keys: call the function and encode result
        let key_value = user_post_key(self);
        bincode::encode_to_vec(&key_value, bincode::config::standard())
            .unwrap_or_else(|_| vec![])
    }
}
```

### schema_name() Method

All schemas also get a `schema_name()` method:

```rust
impl User {
    pub fn schema_name() -> &'static str {
        "User"
    }
}
```

## Best Practices

### Choose the Right Approach

- **Use field keys** when you have a single field that naturally serves as the key
- **Use function keys** when you need to:
  - Combine multiple fields
  - Transform the key (e.g., hashing, formatting)
  - Handle complex enum variants
  - Generate derived keys

### Performance Considerations

- Field keys are slightly more efficient as they avoid function calls
- Function keys provide more flexibility at the cost of a function call
- Choose encodable return types wisely (smaller types = smaller keys)

### Key Uniqueness

- Ensure your key generation produces unique keys for different instances
- For hash-based keys, consider collision probability
- For composite keys, ensure the combination is unique

### Error Handling

- The generated `get_key()` method returns an empty vector on encoding errors
- In production, consider implementing custom error handling
- Test your key generation thoroughly with edge cases

## Examples Repository

See the `test_macros/src/lib.rs` file for comprehensive examples of all key generation patterns, including:

- Basic field keys for structs and enums
- Tuple variant keys
- Function-based composite keys
- Hash-based keys
- String formatting keys
- Complex enum handling

## Migration Guide

### From Field Keys to Function Keys

If you want to change from field keys to function keys:

1. Remove `#[key]` attributes from fields
2. Add `#[key_fn = "function_name"]` to the schema
3. Implement the key generation function
4. Ensure the function returns the same key format for compatibility

### Adding New Key Types

When adding support for new key types:

1. Ensure the type implements `Encode` and `Decode`
2. Test with the actual data you expect to use
3. Consider the serialized size of your keys
4. Validate uniqueness properties