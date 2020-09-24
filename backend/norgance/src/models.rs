use super::schema::citizens;

#[derive(diesel::Queryable)]
pub struct Citizen {
    pub identifier: String,
    pub access_key: String,
    pub public_x448: String,
    pub public_x25519_dalek: String,
    pub public_ed25519_dalek: String,
    pub aead_data: String,
}

#[derive(Insertable)]
#[table_name="citizens"]
pub struct NewCitizen<'a> {
    pub identifier: &'a str,
    pub access_key: &'a str,
    pub public_x448: &'a str,
    pub public_x25519_dalek: &'a str,
    pub public_ed25519_dalek: &'a str,
    pub aead_data: &'a str,
}


use super::schema::identity_documents;

#[derive(diesel::Queryable)]
pub struct IdentityDocument {
    pub identity_document_hash: String,
    pub citizen_identifier: String,
    pub ed25519_dalek_signature: String,
}

#[derive(Insertable)]
#[table_name="identity_documents"]
pub struct NewIdentityDocument<'a> {
    pub identity_document_hash: &'a str,
    pub citizen_identifier: &'a str,
    pub ed25519_dalek_signature: &'a str,
}

use super::schema::shared_documents;

#[derive(diesel::Queryable)]
pub struct SharedDocument {
    pub identifier: String,
    pub aead_data: String,
    pub data_ed25519_dalek_signature: String,
}

#[derive(Insertable)]
#[table_name="shared_documents"]
pub struct NewSharedDocument<'a> {
    pub identifier: &'a str,
    pub aead_data: &'a str,
    pub data_ed25519_dalek_signature: &'a str,
}
