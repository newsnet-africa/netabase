# NetabaseSchema Derive Macro

The `NetabaseSchema` derive macro provides automatic implementation of all the traits needed to use your structs and enums as database schemas in the netabase system. This macro generates all the boilerplate code for working with libp2p Kademlia records, including proper key generation and serialization.

## Quick Start

Simply add the derive macro to your struct or enum:

```rust
use netabase_macros::NetabaseSchema;
use bincode::{Encode, Decode};

#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
}
```

That's it! The macro automatically generates:
- ✅ `From<libp2p::kad::Record>` implementation
- ✅ Key type (`UserKey`) with all required traits
- ✅ Key generation methods
- ✅ Serialization/deserialization
- ✅ Record conversion utilities

## Features

### ✅ Fixed Key Generation Bug
Previous versions had a bug where all struct instances returned the same key. This has been **completely fixed**:

```rust
let user1 = User { id: "123".to_string(), name: "Alice".to_string(), email: "alice@example.com".to_string() };
let user2 = User { id: "456".to_string(), name: "Bob".to_string(), email: "bob@example.com".to_string() };

assert_eq!(user1.key().as_str(), "123");  // ✅ Correct: uses user1's ID
assert_eq!(user2.key().as_str(), "456");  // ✅ Correct: uses user2's ID
assert_ne!(user1.key().as_str(), user2.key().as_str());  // ✅ Different keys!
```

### ✅ Automatic From Implementation
No need to manually implement `From<libp2p::kad::Record>` - it's generated automatically:

```rust
// This code is generated automatically:
impl From<libp2p::kad::Record> for User {
    fn from(record: libp2p::kad::Record) -> Self {
        // Handles deserialization with proper error handling
    }
}
```

## Usage Examples

### Basic Struct

```rust
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct BlogPost {
    #[key]
    slug: String,
    title: String,
    content: String,
    author: String,
    published_at: u64,
}

let post = BlogPost {
    slug: "hello-world".to_string(),
    title: "Hello, World!".to_string(),
    content: "My first post".to_string(),
    author: "Alice".to_string(),
    published_at: 1640995200,
};

// Key is automatically generated from the slug
assert_eq!(post.key().as_str(), "hello-world");

// Convert to/from libp2p records
let record = post.to_kad_record().unwrap();
let recovered = BlogPost::from(record);
assert_eq!(post.slug, recovered.slug);
```

### Enums

```rust
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
enum Product {
    Digital {
        #[key]
        sku: String,
        name: String,
        download_url: String,
    },
    Physical {
        #[key]
        barcode: String,
        name: String,
        weight: f32,
    },
}

let digital = Product::Digital {
    sku: "SOFT001".to_string(),
    name: "My Software".to_string(),
    download_url: "https://example.com/download".to_string(),
};

let physical = Product::Physical {
    barcode: "123456789".to_string(),
    name: "My Widget".to_string(),
    weight: 1.5,
};

// Each variant uses its own key field
assert_eq!(digital.key().as_str(), "SOFT001");
assert_eq!(physical.key().as_str(), "123456789");
```

### Round-trip Conversion

```rust
// Original data
let user = User {
    id: "user123".to_string(),
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
};

// Convert to libp2p record
let record = libp2p::kad::Record::from(user.clone());

// Convert back (using auto-generated From implementation)
let recovered = User::from(record);

// Verify data integrity
assert_eq!(user.id, recovered.id);
assert_eq!(user.name, recovered.name);
assert_eq!(user.email, recovered.email);
assert_eq!(user.key().as_str(), recovered.key().as_str());
```

## Generated Code

When you use `#[derive(NetabaseSchema)]`, the macro generates:

### 1. Key Type
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserKey(pub String);
```

### 2. Key Methods
```rust
impl UserKey {
    pub fn new(value: String) -> Self { Self(value) }
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn into_string(self) -> String { self.0 }
}
```

### 3. NetabaseSchema Implementation
```rust
impl NetabaseSchema for User {
    type Key = UserKey;
    
    fn key(&self) -> Self::Key {
        UserKey(format!("{}", self.id))  // Uses actual field value per instance
    }
}
```

### 4. Automatic From Implementation
```rust
impl From<libp2p::kad::Record> for User {
    fn from(record: libp2p::kad::Record) -> Self {
        // Automatic deserialization with error handling
    }
}

impl From<libp2p::kad::Record> for UserKey {
    fn from(record: libp2p::kad::Record) -> Self {
        // Key extraction from record
    }
}
```

### 5. Utility Methods
```rust
impl User {
    pub fn to_kad_record(&self) -> Result<libp2p::kad::Record, bincode::error::EncodeError> {
        // Convert to libp2p record
    }
    
    pub fn kad_key(&self) -> libp2p::kad::RecordKey {
        // Get the record key
    }
}
```

## Requirements

Your struct or enum must:

1. Implement `Clone`, `Encode`, and `Decode`
2. Have exactly one field marked with `#[key]` per struct/variant
3. Key fields should be types that implement `Display` (like `String`, `u64`, etc.)

```rust
// ✅ Good
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,      // Single key field
    name: String,
}

// ❌ Bad - no key field
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct User {
    id: String,      // Missing #[key]
    name: String,
}

// ❌ Bad - multiple key fields
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,      // Multiple key fields not allowed
    #[key]
    email: String,
}
```

## Migration from Manual Implementation

If you were previously implementing these traits manually, you can remove all the manual implementations and just use the derive macro:

```rust
// Before (manual implementation):
impl From<libp2p::kad::Record> for User {
    fn from(record: libp2p::kad::Record) -> Self {
        // Manual deserialization code...
    }
}

impl NetabaseSchema for User {
    type Key = UserKey;
    fn key(&self) -> Self::Key {
        // Manual key generation...
    }
}

// After (derive macro):
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,
    name: String,
}
// That's it! All implementations are generated automatically.
```

## Error Handling

The generated `From<libp2p::kad::Record>` implementation includes proper error handling:

- **Deserialization errors**: Will panic with a descriptive message if the record data is corrupted or incompatible
- **UTF-8 errors**: Will panic if record keys contain invalid UTF-8 data
- **Version compatibility**: Ensures records are compatible with the current schema

For production use where you need graceful error handling, consider using the generated `TryFrom` implementation (also automatically generated) instead of `From`.

## Performance

The derive macro generates efficient code:

- **Key generation**: No caching overhead - each call computes the key directly from field values
- **Serialization**: Uses bincode for fast, compact serialization
- **Memory usage**: No static storage or memory leaks
- **Thread safety**: All generated code is thread-safe

## Testing

The macro includes comprehensive test coverage ensuring:

- ✅ Different instances have different keys
- ✅ Same field values produce same keys  
- ✅ Round-trip serialization works correctly
- ✅ All generated traits work as expected
- ✅ No memory leaks or static storage issues

You can run the tests with:
```bash
cargo test key_generation_test
cargo test --example complete_example
```
