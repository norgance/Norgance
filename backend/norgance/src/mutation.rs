use juniper::{EmptyMutation, EmptySubscription, FieldResult, RootNode, Variables};

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