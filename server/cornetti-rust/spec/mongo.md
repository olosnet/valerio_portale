# Module: mongo (src/mongo/)

## Purpose

Provides MongoDB integration: connection management, a custom `CornettiObjectId` wrapper
with human-readable serialization, base traits for models and modules, error conversion
with transient detection, and helpers for indexes and decimal conversions.

Requires the `mongo` feature.

## ADDED Requirements

### Requirement: MongoDB connection service

`MongoDBService::new()` SHALL connect to a MongoDB instance using a connection URI
built from `MongoDBConfig`. The URI SHALL optionally include authentication credentials.
The service SHALL expose `db()` and `client()` accessors.

See `MongoDBService` in `src/mongo/services.rs`.

#### Scenario: Connect without authentication
- WHEN `db_username` and `db_password` are both `None`
- THEN the URI SHALL use the format `mongodb://{host}:{port}/{db_name}`

#### Scenario: Connect with authentication
- WHEN both username and password are provided
- THEN the URI SHALL include credentials and `authSource`/`authMechanism` parameters

### Requirement: CornettiObjectId wrapper

`CornettiObjectId` SHALL wrap MongoDB's `ObjectId`. In human-readable formats (JSON),
it SHALL serialize as a 24-character hex string. In binary formats (BSON), it SHALL
serialize as the native `ObjectId`. Deserialization of invalid hex strings SHALL
produce a default `ObjectId`.

See `CornettiObjectId` in `src/mongo/types.rs`.

#### Scenario: JSON serialization
- WHEN a `CornettiObjectId` is serialized to JSON
- THEN the output SHALL be a hex string

#### Scenario: Invalid hex deserialization from `&str`
- WHEN `CornettiObjectId::from("not-a-valid-oid")` is called
- THEN a new default `ObjectId` SHALL be returned (no error)

### Requirement: Base model traits

`MongoBaseModel` SHALL require `_id`, `created`, `modified` fields, BSON serialization,
touch semantics, and a collection name. `PartialMongoBaseModel` SHALL provide the same
without `_id`/`created` for update models.

See `src/mongo/traits.rs`.

#### Scenario: Model touch
- WHEN `touch()` is called on a `MongoBaseModel`
- THEN the `modified` timestamp SHALL be updated to the current time

### Requirement: Module registration and migration

`MongoBaseModule::register()` SHALL perform incremental version-based migration:
for each version from the stored version up to `module_version()`, it SHALL call
`create_indexes` and `seed`, then update the `modules` collection with the new version
and permissions.

See `MongoBaseModule` in `src/mongo/traits.rs`.

#### Scenario: First-time registration
- WHEN no module version is stored in the `modules` collection
- THEN all versions from 0 to `module_version()` SHALL be migrated

#### Scenario: No-op on same version
- WHEN the stored version equals `module_version()`
- THEN no migrations SHALL run, only a version log message SHALL be emitted

### Requirement: Error classification

MongoDB errors SHALL be classified as:
- **Duplicate key** (code 11000) → 409 Conflict
- **Transient** (I/O, connection pool cleared, server selection, retryable labels) → 503
- **All others** → 500. BSON errors SHALL always produce 400.

See `src/mongo/errors.rs`.

#### Scenario: Duplicate key produces conflict
- WHEN a MongoDB write fails with error code 11000
- THEN `From<mongodb::error::Error>` SHALL return a 409 `CornettiError`

#### Scenario: Transient error for retry
- WHEN `is_transient_mongo_error` is called on a connection pool cleared error
- THEN it SHALL return `true`

### Requirement: Optional ObjectId serde helpers

The system SHALL provide serde helpers in `optional_objectid_as_human_readable` and
`vec_objectid_as_human_readable` that serialize `Option<CornettiObjectId>` and
`Vec<CornettiObjectId>` as hex strings.

See `src/mongo/serde.rs`.

### Requirement: Decimal128 conversion

The system SHALL provide `decimal128_to_decimal` and `decimal_to_decimal128` converters
between BSON `Decimal128` and `rust_decimal::Decimal`.

See `src/mongo/helpers.rs`.
