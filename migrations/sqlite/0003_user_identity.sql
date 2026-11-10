CREATE TABLE urp_users (
    user_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL
        CHECK (length(trim(display_name)) BETWEEN 1 AND 160),
    status TEXT NOT NULL DEFAULT 'invited'
        CHECK (status IN ('invited', 'active', 'suspended', 'disabled')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    disabled_at TEXT,
    CHECK (status <> 'disabled' OR disabled_at IS NOT NULL)
);

CREATE INDEX urp_users_status_created_idx
    ON urp_users (status, created_at, user_id);

CREATE TABLE urp_auth_identities (
    identity_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES urp_users (user_id) ON DELETE RESTRICT,
    provider_key TEXT NOT NULL
        CHECK (provider_key = lower(trim(provider_key))
            AND length(provider_key) BETWEEN 1 AND 64),
    provider_subject TEXT,
    login_normalized TEXT NOT NULL UNIQUE
        CHECK (login_normalized = lower(trim(login_normalized))
            AND length(login_normalized) BETWEEN 3 AND 254),
    email_normalized TEXT
        CHECK (email_normalized IS NULL OR
            (email_normalized = lower(trim(email_normalized))
                AND length(email_normalized) BETWEEN 3 AND 254)),
    email_verified INTEGER NOT NULL DEFAULT 0
        CHECK (email_verified IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_authenticated_at TEXT,
    CHECK (provider_key = 'local' OR provider_subject IS NOT NULL)
);

CREATE UNIQUE INDEX urp_auth_identities_email_unique_idx
    ON urp_auth_identities (email_normalized)
    WHERE email_normalized IS NOT NULL;

CREATE UNIQUE INDEX urp_auth_identities_provider_subject_unique_idx
    ON urp_auth_identities (provider_key, provider_subject)
    WHERE provider_subject IS NOT NULL;

CREATE INDEX urp_auth_identities_user_idx
    ON urp_auth_identities (user_id, created_at, identity_id);

CREATE TABLE urp_user_credentials (
    user_id TEXT PRIMARY KEY REFERENCES urp_users (user_id) ON DELETE RESTRICT,
    hash_scheme TEXT NOT NULL CHECK (hash_scheme = 'argon2id'),
    hash_version INTEGER NOT NULL CHECK (hash_version > 0),
    encoded_hash TEXT NOT NULL
        CHECK (length(trim(encoded_hash)) BETWEEN 1 AND 1024),
    password_changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at TEXT
);
