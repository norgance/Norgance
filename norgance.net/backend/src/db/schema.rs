table! {
    citizens (identifier) {
        identifier -> Text,
        access_key -> Text,
        public_x25519_dalek -> Text,
        public_ed25519_dalek -> Text,
        aead_data -> Text,
    }
}

table! {
    identity_documents (identity_document_hash) {
        identity_document_hash -> Text,
        citizen_identifier -> Text,
        ed25519_dalek_signature -> Text,
    }
}

table! {
    shared_documents (identifier) {
        identifier -> Text,
        aead_data -> Text,
        data_ed25519_dalek_signature -> Text,
    }
}

joinable!(identity_documents -> citizens (citizen_identifier));

allow_tables_to_appear_in_same_query!(
    citizens,
    identity_documents,
    shared_documents,
);
