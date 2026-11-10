CREATE TABLE urp_outbox_events (
    event_id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    event_version SMALLINT NOT NULL CHECK (event_version > 0),
    tenant_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    aggregate_version BIGINT NOT NULL CHECK (aggregate_version >= 0),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user', 'service', 'system')),
    actor_id UUID,
    occurred_at TIMESTAMPTZ NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'claimed', 'delivered', 'failed')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    claimed_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    last_error TEXT
);

CREATE INDEX urp_outbox_pending_created_idx
    ON urp_outbox_events (status, created_at, event_id);

CREATE INDEX urp_outbox_tenant_created_idx
    ON urp_outbox_events (tenant_id, created_at DESC);
