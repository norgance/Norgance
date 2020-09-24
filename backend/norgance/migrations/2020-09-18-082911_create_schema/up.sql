/**
 * The citizen table contains the most important data.
 *
 */
CREATE TABLE citizens (
  identifier TEXT
    PRIMARY KEY
    NOT NULL
    CONSTRAINT valid_identifier_length
      CHECK (char_length(identifier) = 64),
  access_key TEXT
    NOT NULL
    CONSTRAINT valid_access_key_length
      CHECK (char_length(access_key) = 64),
  public_x448 TEXT NOT NULL, 
  public_x25519_dalek TEXT NOT NULL, 
  public_ed25519_dalek TEXT NOT NULL,
  aead_data TEXT NOT NULL
);

CREATE TABLE identity_documents (
  identity_document_hash TEXT
    PRIMARY KEY
    NOT NULL 
    CONSTRAINT valid_hash_length
      CHECK (char_length(identity_document_hash) = 64),
  citizen_identifier TEXT
    NOT NULL
    REFERENCES citizens(identifier)
    ON DELETE CASCADE,
  ed25519_dalek_signature TEXT NOT NULL
);

CREATE TABLE shared_documents (
  identifier TEXT
    PRIMARY KEY
    NOT NULL
    CONSTRAINT valid_identifier_length
      CHECK (char_length(identifier) = 64),
  
  aead_data TEXT NOT NULL,
  data_ed25519_dalek_signature TEXT NOT NULL
);
