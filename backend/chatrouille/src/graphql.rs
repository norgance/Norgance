/**
 * {
        query: operationsDoc,
        variables: variables,
        operationName: operationName
      }
*/
pub enum GraphQLRequest {
  operationName: String,
  query: String,
  variables: String,
  accessKey: String,
}