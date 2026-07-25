# Module: mongo (src/mongo/)

## Purpose

Provides MongoDB integration: connection management, a `CornettiObjectId` type alias
for `bson::oid::ObjectId` with serde helpers for human-readable serialization, base traits
for models and modules including `TryMergeFrom` for DTO-to-model merging, error conversion
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

### Requirement: CornettiObjectId type alias

`CornettiObjectId` SHALL be a type alias for `bson::oid::ObjectId`, provided for backward
compatibility. Consumers SHOULD prefer `bson::oid::ObjectId` directly. Parsing from a hex
string SHALL use the standalone `parse_object_id()` function, which SHALL return a
`Result` and SHALL NOT silently default on invalid input.

See `CornettiObjectId` and `parse_object_id` in `src/mongo/types.rs`.

#### Scenario: Type alias resolves to ObjectId
- WHEN a consumer uses `CornettiObjectId` in a type position
- THEN it SHALL be semantically identical to `bson::oid::ObjectId`

#### Scenario: Valid hex parsing succeeds
- WHEN `parse_object_id` is called with a valid 24-character hex string
- THEN an `Ok(ObjectId)` SHALL be returned

#### Scenario: Invalid hex parsing returns error
- WHEN `parse_object_id` is called with an invalid hex string
- THEN an `Err(bson::error::Error)` SHALL be returned

### Requirement: Base model traits

`MongoBaseModel` SHALL require `_id`, `created`, `modified` fields, BSON serialization,
touch semantics, and a collection name. `PartialMongoBaseModel` SHALL provide the same
without `_id`/`created` for update models.

See `src/mongo/traits.rs`.

#### Scenario: Model touch
- WHEN `touch()` is called on a `MongoBaseModel`
- THEN the `modified` timestamp SHALL be updated to the current time

### Requirement: DTO-to-Model merge via TryMergeFrom

The `TryMergeFrom<T>` trait SHALL provide a method for merging an update DTO into a
Mongo model loaded from the database. Only fields present in the DTO SHALL be
overwritten; fields not in the DTO (e.g. `_id`, `created`, `default`) SHALL be
preserved from the database-loaded model. Implementations SHALL return `Ok(())` on
successful merge, or a `CornettiError` if a field cannot be converted (e.g. an
invalid ObjectId hex string).

See `TryMergeFrom` in `src/mongo/traits.rs`.

#### Scenario: Successful merge overwrites DTO fields
- WHEN `try_merge_from` is called with an update DTO containing names and descriptions
- THEN the model's corresponding fields SHALL be updated to the DTO values
- AND fields not present in the DTO (`_id`, `created`, etc.) SHALL remain unchanged

#### Scenario: Infallible merge returns Ok
- WHEN an implementation performs only infallible field assignments (no ObjectId parsing)
- THEN `try_merge_from` SHALL return `Ok(())`

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

Errors are constructed via the centralized error factory system (`errors::mongo`,
`errors::conflict`), with `internal_detail` set to the original error string.

See `src/mongo/adapters.rs`.

#### Scenario: Duplicate key produces conflict
- WHEN a MongoDB write fails with error code 11000
- THEN `From<mongodb::error::Error>` SHALL return a 409 `CornettiError`

#### Scenario: Transient error for retry
- WHEN `is_transient_mongo_error` is called on a connection pool cleared error
- THEN it SHALL return `true`

### Requirement: ObjectId serde helpers

The system SHALL provide serde helpers in `optional_objectid_as_human_readable` and
`vec_objectid_as_human_readable` that serialize `Option<ObjectId>` and
`Vec<ObjectId>` as hex strings in human-readable formats. Deserialization of invalid
hex strings SHALL return an error rather than silently producing a default value.

See `src/mongo/serde.rs`.

### Requirement: Decimal128 conversion

The system SHALL provide `decimal128_to_decimal` and `decimal_to_decimal128` converters
between BSON `Decimal128` and `rust_decimal::Decimal`.

See `src/mongo/helpers.rs`.

### Requirement: MongoDB pagination query builder

`MongoPagination` SHALL provide a two-path pagination strategy:

- **Simple path**: when no fields referenced in `LoadOptions` are present in the
  `join_dict`, `paginate()` SHALL use `find()` + `count_documents()` with the built
  sort, skip, and limit. The count SHALL be skipped (set to -1) when
  `require_total_count` is false.
- **JOIN path**: when any field requires a JOIN, `paginate()` SHALL use an
  aggregation pipeline with `$lookup` stages. For inner joins, `$unwind` without
  `preserveNullAndEmptyArrays`. For outer/LEFT joins, `$unwind` with
  `preserveNullAndEmptyArrays: true`. The pipeline SHALL include a `$group` +
  `$replaceRoot` stage to deduplicate rows from one-to-many relationships.

The count pipeline SHALL use `$lookup` without `$unwind`, applying filter
conditions on the joined array and checking `$ne: []` for inner joins, to
avoid inflating the count.

`build_filter()` SHALL convert a `FilterNode` into a BSON `Document` suitable for
`$match`. String operators (`Contains`, `NotContains`, `StartsWith`, `EndsWith`)
SHALL use `$regex`. `Not` nodes SHALL use `$nor`.

`build_sort()` SHALL convert a slice of `SortDescriptor` into a BSON sort document,
resolving field names through the join dictionary (`virtual_field.target_field` for
joined fields).

See `src/mongo/pagination.rs`.

#### Scenario: Simple pagination without JOINs
- WHEN `MongoPagination::paginate` is called with a `LoadOptions` containing
  only fields not present in the `join_dict`
- THEN `find()` + `count_documents()` SHALL be used
- AND the cursor SHALL be configured with the built sort, skip, and limit

#### Scenario: Pagination with JOINs uses aggregation pipeline
- WHEN a sort or filter field is present in the `join_dict`
- THEN the aggregation pipeline SHALL be used
- AND `$lookup` stages SHALL be generated for each unique `target_entity`
- AND the count SHALL use a separate pipeline without `$unwind`

#### Scenario: Filter build for Contains operator
- WHEN `MongoPagination::build_filter` is called with
  `FilterNode::Leaf { field: "name", operator: Contains, value: String("Mario") }`
- THEN the resulting BSON document SHALL be `{ "name": { "$regex": "Mario" } }`

#### Scenario: NOT filter uses $nor
- WHEN `MongoPagination::build_filter` is called with a `FilterNode::Not`
- THEN the resulting BSON document SHALL use `{ "$nor": [...] }`
