use juniper::{EmptySubscription, FieldResult, RootNode};
use snafu::{ResultExt, Snafu};
use std::sync::Arc;

use crate::db;
use crate::validation;

#[derive(Debug, Snafu)]
pub enum NorganceError {
    #[snafu(display("Database connection error"))]
    DatabaseConnectionError { source: r2d2::Error },

    #[snafu(display("Database transaction error"))]
    DatabaseTransactionError { source: diesel::result::Error },
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
    public_x448: String,
    public_x25519_dalek: String,
    public_ed25519_dalek: String,
    aead_data: String,
}

#[derive(juniper::GraphQLObject, Clone)]
pub struct CitizenPublicKeys {
    public_x448: String,
    public_x25519_dalek: String,
    public_ed25519_dalek: String,
}

#[derive(juniper::GraphQLObject, Clone)]
pub struct CitizenRegistrationResult {
    success: bool,
    valid_identifier: bool,
    valid_access_key: bool,
    valid_public_x448: bool,
    valid_public_x25519_dalek: bool,
    valid_public_ed25519_dalek: bool,
    valid_aead_data: bool,
}

/**
 * Context
 **/
pub struct Ctx {
    pub db_pool: Arc<db::DbPool>,
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
        if !validation::validate_identifier(&identifier) {
            return Ok(false);
        }

        let db = context.db_pool.get().context(DatabaseConnectionError)?;
        let available = db::is_identifier_available(&db, &identifier)?;

        Ok(available)
    }

    /// Technically, the access key would be enough to load the data
    /// But it's very cheap to also check the identifier
    /// so we also ask for the identifier
    fn loadCitizenPersonalData(
        _context: &Ctx,
        _identifier: String,
        _access_key: String,
    ) -> FieldResult<Option<String>> {
        Ok(None)
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
            public_x448: public_keys.public_x448,
            public_x25519_dalek: public_keys.public_x25519_dalek,
            public_ed25519_dalek: public_keys.public_ed25519_dalek,
        };

        Ok(Some(result))
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
    fn registerCitizenShip(
        context: &Ctx,
        registration: CitizenRegistration,
    ) -> FieldResult<CitizenRegistrationResult> {
        // This is not very nice
        let mut result = CitizenRegistrationResult {
            success: false,
            valid_identifier: validation::validate_identifier(&registration.identifier),
            valid_access_key: validation::validate_key(&registration.access_key),
            valid_aead_data: validation::validate_base64(&registration.aead_data),
            valid_public_ed25519_dalek: validation::validate_base64(
                &registration.public_ed25519_dalek,
            ),
            valid_public_x25519_dalek: validation::validate_base64(
                &registration.public_x25519_dalek,
            ),
            valid_public_x448: validation::validate_base64(&registration.public_x448),
        };

        if !result.valid_identifier
            || !result.valid_access_key
            || !result.valid_aead_data
            || !result.valid_public_ed25519_dalek
            || !result.valid_public_x25519_dalek
            || !result.valid_public_x448
        {
            return Ok(result);
        }

        use crate::models::{Citizen, NewCitizen};
        use crate::schema::citizens;
        use diesel::prelude::*;

        // Glue
        let new_citizen = NewCitizen {
            identifier: &registration.identifier,
            access_key: &registration.access_key,
            public_x448: &registration.public_x448,
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
