/**
 * The citizen table contains the most important data.
 *
 * All binary data is encoded as base64 string, without padding.
 */
CREATE TABLE citizens (
  identifier TEXT
    PRIMARY KEY
    NOT NULL
    CONSTRAINT valid_identifier
      CHECK (identifier ~ '^[a-zA-Z0-9+/]{64}$'),
  access_key TEXT
    NOT NULL
    CONSTRAINT valid_access_key
      CHECK (access_key ~ '^[a-zA-Z0-9+/]{43}$'
        AND access_key <> identifier),
  public_x25519_dalek TEXT
    NOT NULL
    CONSTRAINT valid_public_x25519_dalek
      CHECK (public_x25519_dalek ~ '^[a-zA-Z0-9+/]{43}$'),
  public_ed25519_dalek TEXT
    NOT NULL
    CONSTRAINT valid_public_ed25519_dalek
      CHECK (public_ed25519_dalek ~ '^[a-zA-Z0-9+/]{43}$'),
  aead_data TEXT
    NOT NULL
    CONSTRAINT valid_aead_data
      CHECK (aead_data ~ '^[a-zA-Z0-9+/]{55,}$')
);

CREATE TABLE identity_documents (
  identity_document_hash TEXT
    PRIMARY KEY
    NOT NULL 
    CONSTRAINT valid_identity_document_hash
      CHECK (identity_document_hash ~ '^[a-zA-Z0-9+/]{64}$'),
  citizen_identifier TEXT
    NOT NULL
    REFERENCES citizens(identifier)
    ON DELETE CASCADE,
  ed25519_dalek_signature TEXT
    NOT NULL
    CONSTRAINT valid_ed25519_dalek_signature
      CHECK (ed25519_dalek_signature ~ '^[a-zA-Z0-9+/]{86}$')
);

CREATE TABLE shared_documents (
  identifier TEXT
    PRIMARY KEY
    NOT NULL
    CONSTRAINT valid_identifier
      CHECK (identifier ~ '^[a-zA-Z0-9+/]{64}$'),
  
  aead_data TEXT
    NOT NULL
    CONSTRAINT valid_aead_data
      CHECK (aead_data ~ '^[a-zA-Z0-9+/]{55,}$'),
  
  data_ed25519_dalek_signature TEXT
    NOT NULL
    CONSTRAINT valid_data_ed25519_dalek_signature
      CHECK (data_ed25519_dalek_signature ~ '^[a-zA-Z0-9+/]{86}$')
);
