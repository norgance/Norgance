use juniper::{EmptySubscription, FieldResult, RootNode};
use snafu::{ResultExt, Snafu};
use std::sync::Arc;

use crate::db;
use crate::server::check_password_quality;
use crate::validation;
use crate::vault;

#[derive(Debug, Snafu)]
pub enum NorganceError {
    #[snafu(display("Database connection error"))]
    DatabaseConnectionError { source: r2d2::Error },

    #[snafu(display("Database transaction error"))]
    DatabaseTransactionError { source: diesel::result::Error },

    #[snafu(display("The identifier format is invalid"))]
    InvalidIdentifier,

    #[snafu(display("Error while communicating with the vault: {}", source))]
    VaultError { source: vault::VaultError },
}

/**
 * Types
 **/
/*#[derive(juniper::GraphQLObject, Clone)]
pub struct Citizen {
    identifier: String,
    publicX448: String,
    publicX25519Dalek: String,
    publicEd25519Dalek: String,
    aeadData: String,
}*/

#[derive(juniper::GraphQLInputObject)]
pub struct CitizenRegistration {
    identifier: String,
    access_key: String,
    public_x25519_dalek: String,
    public_ed25519_dalek: String,
    aead_data: String,
}

#[derive(juniper::GraphQLObject, Clone)]
pub struct CitizenPublicKeys {
    public_x25519_dalek: String,
    public_ed25519_dalek: String,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(juniper::GraphQLObject, Clone)]
pub struct CitizenRegistrationResult {
    success: bool,
    valid_identifier: bool,
    valid_access_key: bool,
    valid_public_x25519_dalek: bool,
    valid_public_ed25519_dalek: bool,
    valid_aead_data: bool,
}

#[derive(juniper::GraphQLObject, Clone)]
pub struct NorgancePublicKey {
    public_ed25519_dalek: String,
    creation_time: String,
}

/**
 * Context
 **/
pub struct Ctx {
    pub db_pool: Arc<db::DbPool>,
    pub vault_client: Arc<vault::Client>,
    pub citizen_identifier: Option<String>,
}
impl juniper::Context for Ctx {}

fn db_connection(context: &Ctx) -> Result<db::DbPooledConnection, NorganceError> {
    let db = context.db_pool.get().context(DatabaseConnectionError)?;
    Ok(db)
}

/**
 * Query
 **/
pub struct Query;

#[juniper::graphql_object(
    Context = Ctx,
)]
impl Query {
    /// Returns whether a citizen identifier is available.
    ///
    /// Identifiers are not reserved until the citizenship is created,
    /// therefore if someone is too slow to apply to their citizenship,
    /// they may get an error later even though this call returned true.
    fn isIdentifierAvailable(context: &Ctx, identifier: String) -> FieldResult<bool> {
        if !validation::identifier(&identifier) {
            return Err(NorganceError::InvalidIdentifier.into());
        }

        let db = db_connection(context)?;
        let available = db::is_identifier_available(&db, &identifier)?;

        Ok(available)
    }

    fn loadCitizenPersonalData(context: &Ctx) -> FieldResult<Option<String>> {
        let identifier = match &context.citizen_identifier {
            Some(identifier) => identifier,
            None => return Ok(None),
        };
        let db = db_connection(context)?;

        match db::load_citizen_personal_data(&db, &identifier)? {
            Some(aead_data) => Ok(Some(aead_data)),
            None => Ok(None),
        }
    }

    /// Returns the public keys of a citizen
    fn loadCitizenPublicKeys(
        context: &Ctx,
        identifier: String,
    ) -> FieldResult<Option<CitizenPublicKeys>> {
        let db = db_connection(context)?;

        let public_keys = match db::load_citizen_public_keys(&db, &identifier)? {
            Some(pk) => pk,
            None => return Ok(None),
        };

        // Glue
        let result = CitizenPublicKeys {
            public_x25519_dalek: public_keys.public_x25519_dalek,
            public_ed25519_dalek: public_keys.public_ed25519_dalek,
        };

        Ok(Some(result))
    }

    async fn checkPasswordQuality(
        prefix: String,
    ) -> FieldResult<Vec<check_password_quality::PasswordQuality>> {
        let res = check_password_quality::check_password_quality(prefix).await?;
        Ok(res)
    }

    async fn getNorgancePublicKeys(context: &Ctx) -> FieldResult<Vec<NorgancePublicKey>> {
        let public_keys = context
            .vault_client
            .load_public_keys("tamponner")
            .await
            .context(VaultError)?;

        let mut norgance_public_keys : Vec<NorgancePublicKey> = public_keys
            .iter()
            .map(|public_key| NorgancePublicKey {
                public_ed25519_dalek: public_key.public_key.clone().replace("=", ""),
                creation_time: public_key.creation_time.clone(),
            })
            .collect();

        norgance_public_keys.sort_by(|a,b| a.creation_time.cmp(&b.creation_time));

        Ok(norgance_public_keys)
    }
}

/**
 * Mutation
 **/
pub struct Mutation;

#[juniper::graphql_object(
    Context = Ctx,
)]
impl Mutation {
    fn registerCitizenship(
        context: &Ctx,
        registration: CitizenRegistration,
    ) -> FieldResult<CitizenRegistrationResult> {
        use crate::db::models::{Citizen, NewCitizen};
        use crate::db::schema::citizens;
        use diesel::prelude::*;

        // This is not very nice
        let mut result = CitizenRegistrationResult {
            success: false,
            valid_identifier: validation::identifier(&registration.identifier),
            valid_access_key: validation::curve25519_public_key_base64_no_padding(
                &registration.access_key,
            ),
            valid_aead_data: validation::aead_data_base64_no_padding(&registration.aead_data),
            valid_public_ed25519_dalek: validation::curve25519_public_key_base64_no_padding(
                &registration.public_ed25519_dalek,
            ),
            valid_public_x25519_dalek: validation::curve25519_public_key_base64_no_padding(
                &registration.public_x25519_dalek,
            ),
        };

        if !result.valid_identifier
            || !result.valid_access_key
            || !result.valid_aead_data
            || !result.valid_public_ed25519_dalek
            || !result.valid_public_x25519_dalek
        {
            return Ok(result);
        }

        // Glue
        let new_citizen = NewCitizen {
            identifier: &registration.identifier,
            access_key: &registration.access_key,
            public_x25519_dalek: &registration.public_x25519_dalek,
            public_ed25519_dalek: &registration.public_ed25519_dalek,
            aead_data: &registration.aead_data,
        };

        let db = db_connection(context)?;

        let _citizen: Citizen = diesel::insert_into(citizens::table)
            .values(&new_citizen)
            .get_result(&db)
            .context(DatabaseTransactionError)?;

        result.success = true;

        Ok(result)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Ctx>>;

pub fn new_root_node() -> Arc<Schema> {
    Arc::new(RootNode::new(
        Query,
        Mutation,
        EmptySubscription::<Ctx>::new(),
    ))
}
