use juniper::{EmptySubscription, FieldResult, RootNode};
use std::sync::Arc;

use crate::db;
use crate::schema;
use crate::validation;

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

/**
 * Context
 **/
pub struct Ctx {
    pub db_pool: Arc<db::DbPool>,
    pub citizen_identifier: Option<String>,
}
impl juniper::Context for Ctx {}

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

        // We have a name conflict
        let prout = identifier;
        {
            use diesel::dsl::*;
            use diesel::prelude::*;
            use schema::citizens::dsl::*;
            let db = context.db_pool.get()?;

            let query = select(not(exists(
                citizens
                    .filter(identifier.eq(prout))
                    .select(identifier),
            )));

            println!("{}", diesel::debug_query::<diesel::pg::Pg, _>(&query));

            return Ok(query.get_result(&db)?);
        }

        /*let query = diesel::sql_query("SELECT NOT EXISTS (SELECT identifier from citizens where identifier = $1) as available")
                .bind::<diesel::sql_types::Text,_>(identifier);
        println!("{}", diesel::debug_query::<diesel::pg::Pg,_>(&query));

        use diesel::sql_types::Bool;
        #[derive(QueryableByName)]
        struct Exists {
            #[sql_type="Bool"]
            available: bool,
        }

        let mut availables = query.load::<Exists>(&db)?;
        let available = availables.pop().expect("No result");
        Ok(available.available)*/
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
        _context: &Ctx,
        _identifier: String,
    ) -> FieldResult<Option<CitizenPublicKeys>> {
        Ok(Some(CitizenPublicKeys {
            public_x448: String::from("x448"),
            public_x25519_dalek: String::from("x25519"),
            public_ed25519_dalek: String::from("ed25519"),
        }))
        //Ok(None)
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
    fn registerCitizenShip(_context: &Ctx, _registration: CitizenRegistration) -> FieldResult<bool> {
        Ok(true)
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
