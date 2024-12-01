//! GraphQL API definitions.

pub mod contract;
mod mutation;
pub mod placement;
mod query;
pub mod realty;
pub mod report;
pub mod scalar;
mod subscription;
pub mod user;

use crate::define_error;

pub use self::{
    contract::{Contract, ContractValue},
    mutation::Mutation,
    query::Query,
    realty::Realty,
    subscription::Subscription,
    user::User,
};

/// GraphQL schema.
pub type Schema = juniper::RootNode<'static, Query, Mutation, Subscription>;

define_error! {
    enum PrivilegeError {
        #[code = "NOT_EMPLOYER"]
        #[status = FORBIDDEN]
        #[message = "Authenticated `User` must be an employer"]
        Employer,
    }
}

define_error! {
    enum PaginationError {
        #[code = "AMBIGUOUS_PAGINATION_ARGUMENTS"]
        #[status = BAD_REQUEST]
        #[message = "Ambiguous pagination arguments"]
        Ambiguous,
    }
}
