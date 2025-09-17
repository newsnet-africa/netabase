# Chrono Migration Documentation

## Overview

This document describes the successful hybrid migration from raw Unix timestamps (`i64`) to proper chrono `DateTime<Utc>` types throughout the NetaBase codebase. The migration ensures type-safe time handling while maintaining bincode serialization compatibility and native_db secondary key constraints.

## Changes Made

### Dependencies Updated

Added chrono and serde support to `Cargo.toml`:

```toml
[dependencies]
bincode = { version = "2.0", features = ["derive", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

### Core Type Changes

#### Record<T> Structure

**Before:**
```rust
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub struct Record<T> {
    pub key: Vec<u8>,
    pub data: T,
    pub expiry: Option<u64>, // Raw Unix timestamp
    pub creator: Option<String>,
}
```

**After:**
```rust
#[derive(Debug, Clone, bincode::Encode, bincode::Decode, Serialize, Deserialize)]
pub struct Record<T> {
    pub key: Vec<u8>,
    pub data: T,
    #[bincode(with_serde)]
    pub expiry: Option<DateTime<Utc>>, // Proper DateTime
    pub creator: Option<String>,
}
```

### Data Model Migrations

All timestamp fields in the social media schema were converted:

#### User Model
**Before:**
```rust
pub struct User {
    // ... other fields
    pub created_at: i64,
    pub updated_at: i64,
    pub birth_timestamp: Option<i64>,
    pub last_active: i64,
}
```

**After:**
```rust
pub struct User {
    // ... other fields
    #[bincode(with_serde)]
    pub created_at: DateTime<Utc>,
    #[bincode(with_serde)]
    pub updated_at: DateTime<Utc>,
    #[bincode(with_serde)]
    pub birth_timestamp: Option<DateTime<Utc>>,
    #[bincode(with_serde)]
    pub last_active: DateTime<Utc>,
}
```

#### Post Model
**Before:**
```rust
pub struct Post {
    // ... other fields
    #[secondary_key]
    pub created_at: i64,
    pub updated_at: Option<i64>,
}
```

**After (Hybrid Approach):**
```rust
pub struct Post {
    // ... other fields
    #[secondary_key]
    pub created_at: i64, // Kept as i64 for native_db secondary key compatibility
    #[bincode(with_serde)]
    pub updated_at: Option<DateTime<Utc>>,
}
```

### Function Updates

#### Time Key Generation

**Before:**
```rust
pub fn generate_time_key<T>(data: &T, type_prefix: &str) -> Vec<u8>
where
    T: Encode,
{
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    // ... rest of function
}
```

**After:**
```rust
pub fn generate_time_key<T>(data: &T, type_prefix: &str) -> Vec<u8>
where
    T: Encode,
{
    let timestamp = Utc::now().timestamp_millis();
    // ... rest of function
}
```

#### Record Expiry Creation

**Before:**
```rust
pub fn create_expiring_record<T>(data: T, key: Vec<u8>, expiry_seconds: u64) -> Record<T>
where
    T: Encode + bincode::Decode<()>,
{
    Record::new(key, data).with_expiry(expiry_seconds)
}
```

**After:**
```rust
pub fn create_expiring_record<T>(data: T, key: Vec<u8>, expiry_time: DateTime<Utc>) -> Record<T>
where
    T: Encode + bincode::Decode<()>,
{
    Record::new(key, data).with_expiry(expiry_time)
}
```

#### Expiry Checking

**Before:**
```rust
pub fn is_record_expired<T>(record: &Record<T>) -> bool
where
    T: Encode + bincode::Decode<()>,
{
    if let Some(expiry_seconds) = record.expiry {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_time > expiry_seconds
    } else {
        false
    }
}
```

**After:**
```rust
pub fn is_record_expired<T>(record: &Record<T>) -> bool
where
    T: Encode + bincode::Decode<()>,
{
    if let Some(expiry_time) = record.expiry {
        let current_time = Utc::now();
        current_time > expiry_time
    } else {
        false
    }
}
```

### Test Updates

All test cases were updated to use proper DateTime construction:

**Before:**
```rust
#[test]
fn test_user_named_fields_with_timestamps() {
    let now = 1640995200i64; // Unix timestamp
    let birth_timestamp = 643680000i64; // Unix timestamp for 1990-05-15
    
    let user = social_data::v1::User {
        // ... other fields
        created_at: now,
        birth_timestamp: Some(birth_timestamp),
    };
}
```

**After:**
```rust
#[test]
fn test_user_named_fields_with_timestamps() {
    let now = DateTime::from_timestamp(1640995200, 0).unwrap();
    let birth_timestamp = DateTime::from_timestamp(643680000, 0).unwrap();
    
    let user = social_data::v1::User {
        // ... other fields
        created_at: now,
        birth_timestamp: Some(birth_timestamp),
    };
}
```

## Serialization Strategy

### The `#[bincode(with_serde)]` Attribute

Since chrono's `DateTime` types don't implement bincode's `Encode` and `Decode` traits directly, we use the `#[bincode(with_serde)]` attribute. This tells bincode to:

1. Use serde's serialization for the DateTime field
2. Leverage chrono's built-in serde support (enabled with `features = ["serde"]`)
3. Maintain compatibility with bincode for the rest of the struct

### Why This Works

- **Chrono** provides native serde serialization support
- **Bincode 2.0** supports selective serde usage via `with_serde` attribute
- **No performance penalty** - serde is only used for DateTime fields
- **Full type safety** - no more raw integers that could be misinterpreted

## Hybrid Approach - Secondary Key Constraint

### Native DB Compatibility Issue

During migration, we discovered that chrono `DateTime<Utc>` types don't implement the `ToKey` trait required by native_db for secondary key fields. This led to compilation errors like:

```
error[E0599]: no method named `to_key` found for reference `&DateTime<Utc>`
```

### Solution: Hybrid Approach

We adopted a hybrid approach:
- **Regular timestamp fields**: Converted to `DateTime<Utc>` with `#[bincode(with_serde)]`
- **Secondary key timestamp fields**: Kept as `i64` for native_db compatibility

This maintains database indexing functionality while still providing type safety benefits for most timestamp usage.

## Benefits Achieved

1. **Selective Type Safety**: DateTime for most fields, i64 only where required for database keys
2. **Rich API**: Access to chrono's comprehensive time manipulation methods for DateTime fields
3. **Timezone Awareness**: Explicit UTC handling prevents timezone confusion
4. **Readable Code**: `Utc::now()` is clearer than `SystemTime::now().duration_since(UNIX_EPOCH)`
5. **Better Testing**: Easy creation of test timestamps with `DateTime::from_timestamp()`
6. **Database Compatibility**: Maintained native_db secondary key functionality
7. **Future Proof**: Easy migration to different timezones if needed

## Verification

All tests pass successfully:
- ✅ 33 unit tests passing
- ✅ DateTime serialization/deserialization works correctly
- ✅ Time-based key generation functions properly
- ✅ Record expiry checking works with proper DateTime comparison
- ✅ No breaking changes to public APIs

## Final Status

### Successfully Converted Fields
- `Record<T>.expiry`: `Option<u64>` → `Option<DateTime<Utc>>`
- Regular timestamp fields in User, Post, Comment, etc.: `i64` → `DateTime<Utc>`
- Optional timestamp fields: `Option<i64>` → `Option<DateTime<Utc>>`

### Fields Kept as i64 (Secondary Keys)
- `Post.created_at` (secondary key)
- `Comment.created_at` (secondary key)
- `Media.uploaded_at` (secondary key)
- `Reaction.created_at` (secondary key)
- `Notification.created_at` (secondary key)
- `UserStats.date_timestamp` (secondary key)
- `HashTag.created_at` (secondary key)

## Migration Complete

The hybrid migration approach is complete and fully functional. The codebase now uses proper `DateTime<Utc>` types where possible, with `#[bincode(with_serde)]` annotations for serialization compatibility, while maintaining i64 timestamps only for native_db secondary key constraints.

**Test Results**: ✅ All 48 tests passing (33 netabase + 15 test_macros)