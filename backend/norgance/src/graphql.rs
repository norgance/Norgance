use std::sync::Arc;
use juniper::{EmptySubscription, FieldResult, RootNode, Variables};

use crate::db;
use crate::schema;

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
    accessKey: String,
    publicX448: String,
    publicX25519Dalek: String,
    publicEd25519Dalek: String,
    aeadData: String,
}

#[derive(juniper::GraphQLObject, Clone)]
pub struct CitizenPublicKeys {
    publicX448: String,
    publicX25519Dalek: String,
    publicEd25519Dalek: String,
}

/**
 * Context
 **/
pub struct Ctx {
    pub dbPool: Arc<db::DbPool>,
    pub citizenIdentifier: Option<String>,
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

        // TODO validate identifier for fun

        use diesel::prelude::*;
        
        let db = context.dbPool.get()?;

        /*use diesel::dsl::*;
        use schema::citizens;
        let query = select(exists(
            citizens::table.filter(
                citizens::identifier.eq(identifier))));*/

        let query = diesel::sql_query("SELECT NOT EXISTS (SELECT identifier from citizens where identifier = $1) as available")
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
        Ok(available.available)
    }

    /// Technically, the access key would be enough to load the data
    /// But it's very cheap to also check the identifier
    /// so we also ask for the identifier
    fn loadCitizenPersonalData(context: &Ctx, identifier: String, accessKey: String) -> FieldResult<Option<String>> {
        Ok(None)
    }

    /// Returns the public keys of a citizen
    fn loadCitizenPublicKeys(
        context: &Ctx,
        identifier: String,
    ) -> FieldResult<Option<CitizenPublicKeys>> {
        Ok(Some(CitizenPublicKeys {
            publicX448: String::from("x448"),
            publicX25519Dalek: String::from("x25519"),
            publicEd25519Dalek: String::from("ed25519"),
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
    fn registerCitizenShip(context: &Ctx, registration: CitizenRegistration) -> FieldResult<bool> {
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