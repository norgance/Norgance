use std::sync::Arc;
use std::sync::RwLock;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Response, Server, StatusCode,
};

mod db;

#[macro_use]
extern crate juniper;
use juniper::{EmptyMutation, EmptySubscription, FieldResult, RootNode, Variables};

#[macro_use]
extern crate diesel_migrations;
embed_migrations!("./migrations");

#[derive(juniper::GraphQLEnum, Clone, Copy)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(juniper::GraphQLObject, Clone)]
/// Citizen public keys
/// Encoded as bytes in base64
struct CitizenPublicKeys {
    public_x448: String,
    public_x25519_dalek: String,
    public_ed25519_dalek: String,
}

#[derive(juniper::GraphQLInputObject)]
/// Citizen public keys
/// Encoded as bytes in base64
struct CitizenPublicKeysInput {
    public_x448: String,
    public_x25519_dalek: String,
    public_ed25519_dalek: String,
}

#[derive(juniper::GraphQLInputObject)]
struct CitizenRegistration {
    identifier: String,
    accessKey: String,
    publicKeys: CitizenPublicKeysInput,
    /// Base64 encrypted data
    personalData: String,
}

// Arbitrary context data.
// struct Ctx(Episode);
struct Ctx {
    //db: &diesel::DbConnection,
    db_pool: Arc<db::DbPool>,
    citizen_identifier: Option<String>,
    favoriteEpisode: RwLock<Episode>,
}

impl juniper::Context for Ctx {}

struct Query;

#[juniper::graphql_object(
    Context = Ctx,
)]
impl Query {
    fn favoriteEpisode(context: &Ctx) -> FieldResult<Episode> {
        println!("{}", match &context.citizen_identifier {
            Some(a) => a,
            None => "zut"
        });
        Ok(*context.favoriteEpisode.read().unwrap())
    }

    /// Returns whether a citizen identifier is available.
    ///
    /// Identifiers are not reserved until the citizenship is created,
    /// therefore if someone is too slow to apply to their citizenship,
    /// they may get an error later even though this call returned true.
    fn isIdentifierAvailable(context: &Ctx, identifier: String) -> FieldResult<bool> {
        Ok(true)
    }

    fn loadCitizenPersonalData(context: &Ctx, accessKey: String) -> FieldResult<Option<String>> {
        Ok(None)
    }

    /// Returns the public keys of a citizen
    fn loadCitizenPublicKeys(
        context: &Ctx,
        identifier: String,
    ) -> FieldResult<Option<CitizenPublicKeys>> {
        Ok(Some(CitizenPublicKeys {
            public_x448: String::from("x448"),
            public_x25519_dalek: String::from("x25519"),
            public_ed25519_dalek: String::from("ed25519"),
        }))
        //Ok(None)
    }
}

struct Mutation;

#[juniper::graphql_object(
    Context = Ctx,
)]
impl Mutation {
    fn registerCitizenShip(context: &Ctx, registration: CitizenRegistration) -> FieldResult<bool> {
        Ok(true)
    }

    fn setFavoriteEpisode(context: &Ctx, episode: Episode) -> FieldResult<bool> {
        {
            let mut fe = context.favoriteEpisode.write().unwrap();
            *fe = episode;
        }
        Ok(true)
    }
}

// A root schema consists of a query, a mutation, and a subscription.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, EmptyMutation<Ctx>, EmptySubscription<Ctx>>;

pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

/*pub fn create_post<'a>(conn: &PgConnection) -> models::PublicKeys {
    use schema::public_keys;
    use uuid::Uuid;

    let identifier = Uuid::new_v4();

    let new_post = models::NewPublicKeys {
        identifier,
        public_x448: String::from("canard"),
        public_x25519_dalek: String::from("canard"),
        public_ed25519_dalek: String::from("canard"),
    };

    diesel::insert_into(public_keys::table)
        .values(&new_post)
        .get_result(conn)
        .expect("Error saving new post")
}*/

#[macro_use]
extern crate diesel;

#[tokio::main]
async fn main() {

    let db_pool = db::create_connection_pool().expect("database");

    {
        let should_run_migrations = std::env::var("DATABASE_MIGRATIONS")
            .unwrap_or(String::from("true"))
            .parse::<lenient_bool::LenientBool>()
            .unwrap_or_default()
            .into();
        if should_run_migrations {
            println!("Running migrations");
            let connection = db_pool.get().expect("pool");
            db::migrate(&connection).expect("migrate");
            println!("OK");
        }
    }

    //use schema::PublicKeys::dsl::*;

    /*let connection = establish_connection();

    connection.transaction::<(), _, _>(|| {
        let post = create_post(&connection);
        println!("\nSaved draft {} with id", post.identifier);
        Err(diesel::result::Error::RollbackTransaction)
    });

    let results = schema::public_keys::table
        .limit(5)
        .load::<models::PublicKeys>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.identifier);
        println!("----------\n");
        println!("{:?}", post.public_x448);
    }

    let canards = diesel::sql_query(r###"
    SELECT identifier, public_x448
    FROM public_keys
    WHERE concat(identifier,'') LIKE '%?%'
    "###)
        .bind::<diesel::sql_types::Text,_>("69")
        .load::<models::Canard>(&connection)
        .expect("stuff");
    println!("Displaying {} posts", canards.len());
    for post in canards {
        println!("{}", post.identifier);
        println!("----------\n");
        println!("{:?}", post.public_x448);
    }

    /*println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("----------\n");
        println!("{}", post.body);
    }*/
    // Create a context object.
    //let ctx = Ctx(Episode::NewHope);*/

    /*// Run the executor.
    let (res, _errors) = juniper::execute(
        "query { favoriteEpisode }",
        None,
        &Schema::new(Query, EmptyMutation::new(), EmptySubscription::new()),
        &Variables::new(),
        &ctx,
    ).await.unwrap();

    // Ensure the value matches.
    assert_eq!(
        res,
        graphql_value!({
            "favoriteEpisode": "NEW_HOPE",
        })
    );*/

    pretty_env_logger::init();

    //let addr = ([127, 0, 0, 1], 3000).into();
    let addr = ([0, 0, 0, 0], 3000).into();

    let database = RwLock::new(Episode::NewHope);
    let arc_db_pool = Arc::new(db_pool);
    
    /*let ctx = Ctx {
        db_pool: arc_db_pool,
        favoriteEpisode: database,
        citizen_identifier: None,
    };*/

    //let pool = db::create_db_pool();

    //let db = Arc::new(ctx);

    let root_node = Arc::new(RootNode::new(
        Query,
        Mutation,
        //EmptyMutation::<Ctx>::new(),
        EmptySubscription::<Ctx>::new(),
    ));

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        //let ctx = db.clone();
        let arc_db_pool = arc_db_pool.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                //let ctx = ctx.clone();
                let arc_db_pool = arc_db_pool.clone();
                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::playground("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            let citizen_identifier = match req.headers().get("citizen") {
                                Some(h) => match h.to_str() {
                                    Ok(h) => Some(String::from(h)),
                                    Err(_) => None,
                                },
                                None => None,
                            };

                            /*let arc_db_pool = arc_db_pool.clone();
                            let db_connection = match (&arc_db_pool).get() {
                                Ok(c) => c,
                                Err(error) => {
                                    eprintln!("Unable to get db connection: {}", error);
                                    let mut response = Response::new(Body::empty());
                                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                    return Ok(response);
                                }
                            };*/

                            /*(&arc_db_pool).commit();

                            db::migrate(&db_connection).expect("migrate");*/
                            
                            let prout = Arc::new(Ctx {
                                db_pool: arc_db_pool.clone(),
                                favoriteEpisode: RwLock::new(Episode::NewHope),
                                citizen_identifier: citizen_identifier,//: Some(String::from("prout")),
                            });

                            //ctx.citizen_identifier = Some(String::from("canard"));
                            juniper_hyper::graphql(root_node, prout, req).await
                        }
                        (&Method::POST, "/lapin") => {
                            let mut response = Response::new(Body::empty());
                            *response.body_mut() = Body::from("{\"available\":true}");
                            Ok(response)
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            Ok(response)
                        }
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}
