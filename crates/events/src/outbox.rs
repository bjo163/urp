//! Transactional outbox event contract and append/read repository.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Row, postgres::PgRow, sqlite::SqliteRow};
use thiserror::Error;
use urp_kernel::{
    context::{Actor, ExecutionContext},
    ids::{ServiceId, UserId},
};
use urp_storage::{Database, StorageTransaction};
use uuid::Uuid;

/// Actor identity captured when a domain event is created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventActor {
    /// Human user that initiated the state change.
    User(UserId),
    /// Service identity that initiated the state change.
    Service(ServiceId),
    /// Platform-owned operation without a human/service principal.
    System,
}

impl EventActor {
    fn from_context(actor: &Actor) -> Self {
        match actor {
            Actor::User(id) => Self::User(*id),
            Actor::Service(id) => Self::Service(*id),
            Actor::System => Self::System,
        }
    }

    fn actor_type(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::Service(_) => "service",
            Self::System => "system",
        }
    }

    fn actor_id(&self) -> Option<Uuid> {
        match self {
            Self::User(id) => Some(*id.as_uuid()),
            Self::Service(id) => Some(*id.as_uuid()),
            Self::System => None,
        }
    }

    fn from_parts(actor_type: &str, actor_id: Option<Uuid>) -> Result<Self, OutboxError> {
        match (actor_type, actor_id) {
            ("user", Some(id)) => Ok(Self::User(UserId::from_uuid(id))),
            ("service", Some(id)) => Ok(Self::Service(ServiceId::from_uuid(id))),
            ("system", None) => Ok(Self::System),
            _ => Err(OutboxError::CorruptRecord(
                "stored actor type and actor identifier are inconsistent".to_owned(),
            )),
        }
    }
}

/// Typed, versioned domain event envelope persisted in the outbox.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventEnvelope {
    /// Stable event identifier used for idempotency.
    pub event_id: Uuid,
    /// Namespaced event type, such as `sales.order.confirmed`.
    pub event_type: String,
    /// Positive event schema version.
    pub event_version: u16,
    /// Tenant isolation boundary at the time of the state change.
    pub tenant_id: Uuid,
    /// Namespaced model/aggregate type.
    pub aggregate_type: String,
    /// Stable identifier of the affected aggregate.
    pub aggregate_id: Uuid,
    /// Monotonically increasing aggregate version.
    pub aggregate_version: i64,
    /// Actor captured from the trusted execution context.
    pub actor: EventActor,
    /// UTC instant when the domain state change occurred.
    pub occurred_at: DateTime<Utc>,
    /// Correlation ID linking the event to its originating request/run.
    pub correlation_id: Uuid,
    /// Versioned event payload. Consumers must validate its schema.
    pub payload: Value,
}

impl EventEnvelope {
    /// Creates a validated event from an already trusted execution context.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: &ExecutionContext,
        event_id: Uuid,
        event_type: impl Into<String>,
        event_version: u16,
        aggregate_type: impl Into<String>,
        aggregate_id: Uuid,
        aggregate_version: i64,
        occurred_at: DateTime<Utc>,
        payload: Value,
    ) -> Result<Self, OutboxError> {
        let event_type = event_type.into();
        let aggregate_type = aggregate_type.into();

        if !valid_technical_name(&event_type, true) {
            return Err(OutboxError::InvalidEventType);
        }
        if !valid_technical_name(&aggregate_type, true) {
            return Err(OutboxError::InvalidAggregateType);
        }
        if event_version == 0 || event_version > i16::MAX as u16 {
            return Err(OutboxError::InvalidEventVersion);
        }
        if aggregate_version < 0 {
            return Err(OutboxError::InvalidAggregateVersion);
        }

        Ok(Self {
            event_id,
            event_type,
            event_version,
            tenant_id: *context.tenant_id().as_uuid(),
            aggregate_type,
            aggregate_id,
            aggregate_version,
            actor: EventActor::from_context(context.actor()),
            occurred_at,
            correlation_id: context.correlation_id(),
            payload,
        })
    }
}

fn valid_technical_name(value: &str, allow_dots: bool) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '_'
                || character == '-'
                || (allow_dots && character == '.')
        })
        && !value
            .chars()
            .next()
            .is_some_and(|character| matches!(character, '.' | '-' | '_'))
        && !value
            .chars()
            .last()
            .is_some_and(|character| matches!(character, '.' | '-' | '_'))
        && !value.contains("..")
}

/// Delivery state stored with an outbox record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventStatus {
    /// Event is committed and waiting for a dispatcher.
    Pending,
    /// Event is leased by a dispatcher (claiming is implemented by the worker).
    Claimed,
    /// Event was successfully delivered.
    Delivered,
    /// Event exhausted delivery policy or was marked terminally failed.
    Failed,
}

impl EventStatus {
    fn parse(value: &str) -> Result<Self, OutboxError> {
        match value {
            "pending" => Ok(Self::Pending),
            "claimed" => Ok(Self::Claimed),
            "delivered" => Ok(Self::Delivered),
            "failed" => Ok(Self::Failed),
            _ => Err(OutboxError::CorruptRecord(
                "stored outbox event has an unknown status".to_owned(),
            )),
        }
    }
}

/// Persisted event plus the minimal dispatch state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    /// Immutable event envelope.
    pub event: EventEnvelope,
    /// Current durable delivery state.
    pub status: EventStatus,
    /// Number of delivery attempts recorded so far.
    pub attempt_count: u32,
}

/// Result of appending an event with a stable event ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendOutcome {
    /// A new event row was inserted in the caller's transaction.
    Inserted,
    /// An identical event with this ID already exists.
    AlreadyExists,
}

/// Errors returned by event envelope validation and outbox persistence.
#[derive(Debug, Error)]
pub enum OutboxError {
    /// Event type does not match the required namespaced technical identifier.
    #[error("event type must be a lower-case namespaced technical identifier")]
    InvalidEventType,
    /// Aggregate type does not match the required technical identifier.
    #[error("aggregate type must be a lower-case technical identifier")]
    InvalidAggregateType,
    /// Event schema version must be positive.
    #[error("event schema version must be greater than zero")]
    InvalidEventVersion,
    /// Aggregate version must not be negative.
    #[error("aggregate version must not be negative")]
    InvalidAggregateVersion,
    /// Caller requested an invalid pending-event page size.
    #[error("outbox query limit must be between 1 and 500")]
    InvalidLimit,
    /// An existing event ID was reused with different event contents.
    #[error("event ID already exists with different event contents")]
    EventIdConflict,
    /// Stored data could not be converted into a valid event.
    #[error("outbox contains a corrupt record: {0}")]
    CorruptRecord(String),
    /// JSON serialization or deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Database operation failed.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

const PG_INSERT: &str = r#"
INSERT INTO urp_outbox_events (
    event_id, event_type, event_version, tenant_id,
    aggregate_type, aggregate_id, aggregate_version,
    actor_type, actor_id, occurred_at, correlation_id, payload
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb)
ON CONFLICT (event_id) DO NOTHING
"#;

const SQLITE_INSERT: &str = r#"
INSERT INTO urp_outbox_events (
    event_id, event_type, event_version, tenant_id,
    aggregate_type, aggregate_id, aggregate_version,
    actor_type, actor_id, occurred_at, correlation_id, payload_json
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT (event_id) DO NOTHING
"#;

const PG_FIND_BY_ID: &str = r#"
SELECT event_id, event_type, event_version, tenant_id, aggregate_type,
       aggregate_id, aggregate_version, actor_type, actor_id,
       occurred_at, correlation_id, payload::text AS payload_json,
       status, attempt_count
FROM urp_outbox_events
WHERE event_id = $1
"#;

const SQLITE_FIND_BY_ID: &str = r#"
SELECT event_id, event_type, event_version, tenant_id, aggregate_type,
       aggregate_id, aggregate_version, actor_type, actor_id,
       occurred_at, correlation_id, payload_json, status, attempt_count
FROM urp_outbox_events
WHERE event_id = ?
"#;

const PG_PENDING_FOR_TENANT: &str = r#"
SELECT event_id, event_type, event_version, tenant_id, aggregate_type,
       aggregate_id, aggregate_version, actor_type, actor_id,
       occurred_at, correlation_id, payload::text AS payload_json,
       status, attempt_count
FROM urp_outbox_events
WHERE tenant_id = $1 AND status = 'pending'
ORDER BY created_at, event_id
LIMIT $2
"#;

const SQLITE_PENDING_FOR_TENANT: &str = r#"
SELECT event_id, event_type, event_version, tenant_id, aggregate_type,
       aggregate_id, aggregate_version, actor_type, actor_id,
       occurred_at, correlation_id, payload_json, status, attempt_count
FROM urp_outbox_events
WHERE tenant_id = ? AND status = 'pending'
ORDER BY created_at, event_id
LIMIT ?
"#;

/// Append-only event repository. It never commits the caller's transaction.
pub struct OutboxRepository;

impl OutboxRepository {
    /// Appends an event inside the caller-owned transaction.
    ///
    /// The caller must commit the transaction before the event becomes visible
    /// to readers. A duplicate ID is accepted only when its envelope is identical.
    pub async fn append(
        transaction: &mut StorageTransaction,
        event: &EventEnvelope,
    ) -> Result<AppendOutcome, OutboxError> {
        let payload = serde_json::to_string(&event.payload)?;
        let actor_type = event.actor.actor_type();
        let actor_id = event.actor.actor_id();
        let occurred_at = event.occurred_at.to_rfc3339();

        match transaction {
            StorageTransaction::Postgres(inner) => {
                let result = sqlx::query(PG_INSERT)
                    .bind(event.event_id)
                    .bind(&event.event_type)
                    .bind(
                        i16::try_from(event.event_version)
                            .map_err(|_| OutboxError::InvalidEventVersion)?,
                    )
                    .bind(event.tenant_id)
                    .bind(&event.aggregate_type)
                    .bind(event.aggregate_id)
                    .bind(event.aggregate_version)
                    .bind(actor_type)
                    .bind(actor_id)
                    .bind(event.occurred_at)
                    .bind(event.correlation_id)
                    .bind(payload)
                    .execute(&mut **inner)
                    .await?;

                if result.rows_affected() == 1 {
                    return Ok(AppendOutcome::Inserted);
                }

                let existing = sqlx::query(PG_FIND_BY_ID)
                    .bind(event.event_id)
                    .fetch_optional(&mut **inner)
                    .await?;

                match existing {
                    Some(row) if decode_pg(&row)?.event == *event => {
                        Ok(AppendOutcome::AlreadyExists)
                    }
                    Some(_) => Err(OutboxError::EventIdConflict),
                    None => Err(OutboxError::CorruptRecord(
                        "conflicting event ID could not be loaded".to_owned(),
                    )),
                }
            }
            StorageTransaction::Sqlite(inner) => {
                let result = sqlx::query(SQLITE_INSERT)
                    .bind(event.event_id.to_string())
                    .bind(&event.event_type)
                    .bind(i64::from(event.event_version))
                    .bind(event.tenant_id.to_string())
                    .bind(&event.aggregate_type)
                    .bind(event.aggregate_id.to_string())
                    .bind(event.aggregate_version)
                    .bind(actor_type)
                    .bind(actor_id.map(|value| value.to_string()))
                    .bind(occurred_at)
                    .bind(event.correlation_id.to_string())
                    .bind(payload)
                    .execute(&mut **inner)
                    .await?;

                if result.rows_affected() == 1 {
                    return Ok(AppendOutcome::Inserted);
                }

                let existing = sqlx::query(SQLITE_FIND_BY_ID)
                    .bind(event.event_id.to_string())
                    .fetch_optional(&mut **inner)
                    .await?;

                match existing {
                    Some(row) if decode_sqlite(&row)?.event == *event => {
                        Ok(AppendOutcome::AlreadyExists)
                    }
                    Some(_) => Err(OutboxError::EventIdConflict),
                    None => Err(OutboxError::CorruptRecord(
                        "conflicting event ID could not be loaded".to_owned(),
                    )),
                }
            }
        }
    }

    /// Reads an event by stable ID from committed database state.
    pub async fn find_by_id(
        database: &Database,
        event_id: Uuid,
    ) -> Result<Option<EventRecord>, OutboxError> {
        match database {
            Database::Postgres(pool) => {
                let row = sqlx::query(PG_FIND_BY_ID)
                    .bind(event_id)
                    .fetch_optional(pool)
                    .await?;
                row.as_ref().map(decode_pg).transpose()
            }
            Database::Sqlite(pool) => {
                let row = sqlx::query(SQLITE_FIND_BY_ID)
                    .bind(event_id.to_string())
                    .fetch_optional(pool)
                    .await?;
                row.as_ref().map(decode_sqlite).transpose()
            }
        }
    }

    /// Reads pending events for one tenant in deterministic creation order.
    ///
    /// The limit is deliberately bounded. Claiming, publishing, retries and
    /// delivery-state transitions are implemented by the worker dispatcher.
    pub async fn pending_for_tenant(
        database: &Database,
        tenant_id: Uuid,
        limit: u16,
    ) -> Result<Vec<EventRecord>, OutboxError> {
        if !(1..=500).contains(&limit) {
            return Err(OutboxError::InvalidLimit);
        }

        match database {
            Database::Postgres(pool) => {
                let rows = sqlx::query(PG_PENDING_FOR_TENANT)
                    .bind(tenant_id)
                    .bind(i64::from(limit))
                    .fetch_all(pool)
                    .await?;
                rows.iter().map(decode_pg).collect()
            }
            Database::Sqlite(pool) => {
                let rows = sqlx::query(SQLITE_PENDING_FOR_TENANT)
                    .bind(tenant_id.to_string())
                    .bind(i64::from(limit))
                    .fetch_all(pool)
                    .await?;
                rows.iter().map(decode_sqlite).collect()
            }
        }
    }
}

fn decode_pg(row: &PgRow) -> Result<EventRecord, OutboxError> {
    let event = EventEnvelope {
        event_id: row.try_get("event_id")?,
        event_type: row.try_get("event_type")?,
        event_version: u16::try_from(row.try_get::<i16, _>("event_version")?)
            .map_err(|_| OutboxError::CorruptRecord("invalid event version".to_owned()))?,
        tenant_id: row.try_get("tenant_id")?,
        aggregate_type: row.try_get("aggregate_type")?,
        aggregate_id: row.try_get("aggregate_id")?,
        aggregate_version: row.try_get("aggregate_version")?,
        actor: EventActor::from_parts(
            row.try_get::<String, _>("actor_type")?.as_str(),
            row.try_get("actor_id")?,
        )?,
        occurred_at: row.try_get("occurred_at")?,
        correlation_id: row.try_get("correlation_id")?,
        payload: serde_json::from_str(&row.try_get::<String, _>("payload_json")?)?,
    };
    let attempt_count = u32::try_from(row.try_get::<i32, _>("attempt_count")?)
        .map_err(|_| OutboxError::CorruptRecord("invalid attempt count".to_owned()))?;

    Ok(EventRecord {
        event,
        status: EventStatus::parse(&row.try_get::<String, _>("status")?)?,
        attempt_count,
    })
}

fn decode_sqlite(row: &SqliteRow) -> Result<EventRecord, OutboxError> {
    let parse_uuid = |field: &'static str| -> Result<Uuid, OutboxError> {
        let value: String = row.try_get(field)?;
        Uuid::parse_str(&value)
            .map_err(|_| OutboxError::CorruptRecord(format!("invalid UUID in {field}")))
    };
    let event_version = u16::try_from(row.try_get::<i64, _>("event_version")?)
        .map_err(|_| OutboxError::CorruptRecord("invalid event version".to_owned()))?;
    let occurred_at: String = row.try_get("occurred_at")?;
    let occurred_at = DateTime::parse_from_rfc3339(&occurred_at)
        .map_err(|_| OutboxError::CorruptRecord("invalid occurred_at timestamp".to_owned()))?
        .with_timezone(&Utc);
    let actor_type: String = row.try_get("actor_type")?;
    let actor_id: Option<String> = row.try_get("actor_id")?;
    let actor_id = actor_id
        .map(|value| {
            Uuid::parse_str(&value)
                .map_err(|_| OutboxError::CorruptRecord("invalid actor UUID".to_owned()))
        })
        .transpose()?;
    let event = EventEnvelope {
        event_id: parse_uuid("event_id")?,
        event_type: row.try_get("event_type")?,
        event_version,
        tenant_id: parse_uuid("tenant_id")?,
        aggregate_type: row.try_get("aggregate_type")?,
        aggregate_id: parse_uuid("aggregate_id")?,
        aggregate_version: row.try_get("aggregate_version")?,
        actor: EventActor::from_parts(&actor_type, actor_id)?,
        occurred_at,
        correlation_id: parse_uuid("correlation_id")?,
        payload: serde_json::from_str(&row.try_get::<String, _>("payload_json")?)?,
    };
    let attempt_count = u32::try_from(row.try_get::<i64, _>("attempt_count")?)
        .map_err(|_| OutboxError::CorruptRecord("invalid attempt count".to_owned()))?;

    Ok(EventRecord {
        event,
        status: EventStatus::parse(&row.try_get::<String, _>("status")?)?,
        attempt_count,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        error::Error,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use urp_kernel::{
        context::{Actor, ExecutionContext, Locale, TimeZone as KernelTimeZone},
        ids::{TenantId, UserId},
    };
    use urp_storage::{Database, DatabaseConfig};
    use uuid::Uuid;

    use super::{AppendOutcome, EventEnvelope, EventStatus, OutboxError, OutboxRepository};

    fn context() -> Result<ExecutionContext, Box<dyn Error>> {
        Ok(ExecutionContext::from_trusted_boundary(
            Actor::User(UserId::new()),
            TenantId::new(),
            None,
            None,
            Locale::parse("id-ID")?,
            KernelTimeZone::parse("Asia/Jakarta")?,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )?)
    }

    fn event(
        context: &ExecutionContext,
        event_id: Uuid,
        payload: serde_json::Value,
    ) -> Result<EventEnvelope, Box<dyn Error>> {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 11, 11, 1, 0, 0)
            .single()
            .ok_or("fixed timestamp must be valid")?;
        Ok(EventEnvelope::new(
            context,
            event_id,
            "sales.order.confirmed",
            1,
            "sales.order",
            Uuid::new_v4(),
            7,
            occurred_at,
            payload,
        )?)
    }

    async fn exercise_outbox(database: &Database) -> Result<(), Box<dyn Error>> {
        database.migrate(false).await?;

        let context = context()?;
        let event_id = Uuid::new_v4();
        let envelope = event(&context, event_id, json!({"order_id": "ORD-100"}))?;

        let mut rolled_back = database.begin_transaction().await?;
        assert_eq!(
            OutboxRepository::append(&mut rolled_back, &envelope).await?,
            AppendOutcome::Inserted
        );
        rolled_back.rollback().await?;
        assert!(
            OutboxRepository::find_by_id(database, event_id)
                .await?
                .is_none()
        );

        let mut committed = database.begin_transaction().await?;
        assert_eq!(
            OutboxRepository::append(&mut committed, &envelope).await?,
            AppendOutcome::Inserted
        );
        committed.commit().await?;

        let persisted = OutboxRepository::find_by_id(database, event_id).await?;
        assert!(persisted.is_some());
        let persisted = persisted.ok_or("committed outbox event should be readable")?;
        assert_eq!(persisted.event, envelope);
        assert_eq!(persisted.status, EventStatus::Pending);
        assert_eq!(persisted.attempt_count, 0);

        let mut duplicate = database.begin_transaction().await?;
        assert_eq!(
            OutboxRepository::append(&mut duplicate, &envelope).await?,
            AppendOutcome::AlreadyExists
        );
        duplicate.rollback().await?;

        let conflicting = event(&context, event_id, json!({"order_id": "ORD-CHANGED"}))?;
        let mut conflict_tx = database.begin_transaction().await?;
        assert!(matches!(
            OutboxRepository::append(&mut conflict_tx, &conflicting).await,
            Err(OutboxError::EventIdConflict)
        ));
        conflict_tx.rollback().await?;

        let pending =
            OutboxRepository::pending_for_tenant(database, envelope.tenant_id, 10).await?;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].event.event_id, event_id);
        assert!(matches!(
            OutboxRepository::pending_for_tenant(database, envelope.tenant_id, 0).await,
            Err(OutboxError::InvalidLimit)
        ));

        Ok(())
    }

    fn temporary_database_path() -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let value = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("urp_outbox_{}_{}.db", std::process::id(), value))
    }

    #[tokio::test]
    async fn sqlite_outbox_is_atomic_idempotent_and_queryable() -> Result<(), Box<dyn Error>> {
        let path = temporary_database_path();
        let database = Database::connect(DatabaseConfig {
            url: format!("sqlite://{}", path.display()),
            max_connections: 2,
        })
        .await?;

        exercise_outbox(&database).await?;

        if let Database::Sqlite(pool) = &database {
            pool.close().await;
        }
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}-wal", path.display()));
        let _ = fs::remove_file(format!("{}-shm", path.display()));
        Ok(())
    }

    #[tokio::test]
    async fn postgres_outbox_is_atomic_idempotent_and_queryable() -> Result<(), Box<dyn Error>> {
        let Ok(url) = env::var("URP_OUTBOX_TEST_DATABASE_URL") else {
            return Ok(());
        };
        let database = Database::connect(DatabaseConfig {
            url,
            max_connections: 3,
        })
        .await?;
        exercise_outbox(&database).await?;
        if let Database::Postgres(pool) = &database {
            sqlx::query("DROP TABLE urp_outbox_events")
                .execute(pool)
                .await?;
            sqlx::query("DROP TABLE urp_platform_metadata")
                .execute(pool)
                .await?;
            sqlx::query("DROP TABLE urp_schema_migrations")
                .execute(pool)
                .await?;
            pool.close().await;
        }
        Ok(())
    }

    #[test]
    fn event_names_and_versions_are_validated() {
        assert!(valid_name("sales.order.confirmed", true));
        assert!(!valid_name("Sales.Order", true));
        assert!(!valid_name("sales..order", true));
        assert!(!valid_name("sales.order ", true));
        assert!(!valid_name("", true));
    }

    fn valid_name(value: &str, dots: bool) -> bool {
        super::valid_technical_name(value, dots)
    }
}
