# Key Enum Implementation Summary

## Overview

This document summarizes the implementation of model-specific key enums in the Netabase macro system. The new structure replaces the previous plain-type key approach with a type-safe, hierarchical key system that distinguishes between primary and secondary keys at the type level.

## New Key Structure

### Model-Specific Key Enums

For each model in the schema, the system now generates individual key enums with the following structure:

```rust
// For models with secondary keys (e.g., User)
pub enum UserKeys {
    Primary(String),                    // Primary key type
    Secondary(UserSecondaryKeys),       // Secondary key variants
}

pub enum UserSecondaryKeys {
    username(String),                   // Secondary key: username
    email(String),                      // Secondary key: email
}

// For models without secondary keys (e.g., PrimitiveTest)
pub enum PrimitiveTestKeys {
    Primary(String),                    // Only primary key variant
}
```

### Database-Level Key Enum

The main database key enum now contains variants for each model's key type:

```rust
pub enum SocialMediaSchemaKey {
    User(UserKeys),
    Post(PostKeys),
    Comment(CommentKeys),
    Media(MediaKeys),
    Reaction(ReactionKeys),
    Notification(NotificationKeys),
    UserStats(UserStatsKeys),
    HashTag(HashTagKeys),
    PrimitiveTest(PrimitiveTestKeys),
    TestUnit(TestUnitKeys),
    TestTuple(TestTupleKeys),
}
```

## Implementation Details

### Code Generation

The implementation involves several key components:

1. **Visitor Enhancement**: Modified `NativeModel` struct to extract secondary key information:
   ```rust
   pub struct NativeModel<'ast> {
       pub model: &'ast ItemStruct,
       pub path: Vec<Pair<PathSegment, Token![::]>>,
       pub primary_key_type: Option<Type>,
       pub secondary_keys: Vec<(Ident, Type)>,  // NEW: Secondary key info
   }
   ```

2. **New Generator Functions**:
   - `generate_model_specific_key_enums()`: Creates individual key enums for each model
   - `generate_database_key_variants()`: Creates variants for the main database key enum
   - Updated conversion generation to handle the new structure

3. **Smart Secondary Key Generation**: The system only generates `Secondary` variants for models that actually have secondary keys, avoiding unnecessary complexity for models with only primary keys.

### Key Features

#### Type Safety
- Each model has its own key type, preventing confusion between different models
- Primary and secondary keys are distinguished at the type level
- Compile-time guarantees that the correct key type is used for each model

#### Performance Benefits
- Different key types can be optimized differently
- Primary keys can be handled with fast paths
- Secondary key lookups are clearly identified

#### Extensibility
- Easy to add new secondary keys to any model
- No impact on other models when adding keys to one model
- Clear separation of concerns

## Usage Examples

### Creating Keys

```rust
// Primary key access
let user_primary = UserKeys::Primary("user123".to_string());
let db_key = SocialMediaSchemaKey::User(user_primary);

// Secondary key access
let username_key = UserKeys::Secondary(
    UserSecondaryKeys::username("john_doe".to_string())
);
let db_username_key = SocialMediaSchemaKey::User(username_key);
```

### Pattern Matching

```rust
match db_key {
    SocialMediaSchemaKey::User(UserKeys::Primary(id)) => {
        // Handle user primary key lookup
        println!("Looking up user by ID: {}", id);
    },
    SocialMediaSchemaKey::User(UserKeys::Secondary(UserSecondaryKeys::username(username))) => {
        // Handle user lookup by username
        println!("Looking up user by username: {}", username);
    },
    SocialMediaSchemaKey::User(UserKeys::Secondary(UserSecondaryKeys::email(email))) => {
        // Handle user lookup by email
        println!("Looking up user by email: {}", email);
    },
    _ => {
        // Handle other key types
    }
}
```

## Benefits

### 1. Type Safety
- **Before**: `SocialMediaSchemaKey::UserKey(String)` - unclear what the string represents
- **After**: `SocialMediaSchemaKey::User(UserKeys::Primary(String))` - explicit primary key
- **After**: `SocialMediaSchemaKey::User(UserKeys::Secondary(UserSecondaryKeys::username(String)))` - explicit secondary key type

### 2. Performance
- Primary key operations can be optimized separately from secondary key operations
- Different secondary keys can have different optimization strategies
- Query planning can make better decisions based on key type

### 3. Maintainability
- Clear separation between primary and secondary keys
- Easy to understand what type of lookup is being performed
- Self-documenting code structure

### 4. Extensibility
- Adding new secondary keys doesn't affect existing code
- Each model's key structure is independent
- Easy to add new key types in the future

## Migration Guide

### Old Structure
```rust
// Old approach - unclear key types
let key = SocialMediaSchemaKey::UserKey("user123".to_string());
```

### New Structure
```rust
// New approach - explicit key types
let primary_key = SocialMediaSchemaKey::User(UserKeys::Primary("user123".to_string()));
let username_key = SocialMediaSchemaKey::User(UserKeys::Secondary(
    UserSecondaryKeys::username("john_doe".to_string())
));
```

## Technical Implementation Notes

### Macro Generation
- The `netabase_schema` macro now generates multiple enums per model
- Secondary key enums are only generated when needed
- All generated enums include appropriate derive attributes for serialization

### Bincode Serialization
- Model-specific key enums derive `bincode::Encode` and `bincode::Decode`
- Main database key enum uses custom bincode implementation for efficiency
- Maintains backward compatibility with existing serialized data (when possible)

### Conversion Handling
- Updated conversion generation handles the new key structure
- Primary key conversions are straightforward
- Secondary key conversions are marked as TODO for future implementation

## Future Enhancements

1. **Secondary Key Query Optimization**: Implement efficient secondary key to primary key lookups
2. **Index Generation**: Automatically generate appropriate database indices for secondary keys
3. **Query Builder**: Create type-safe query builders that leverage the key structure
4. **Caching**: Implement key-type-aware caching strategies

## Testing

The implementation includes comprehensive tests demonstrating:
- Key creation and pattern matching
- Type safety guarantees
- Handling of models with and without secondary keys
- Database-level key enum usage

All tests pass successfully, confirming the implementation works as designed.