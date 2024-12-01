//! [`Command`] definition.

pub mod authorize_user_session;
pub mod create_employment_contract;
pub mod create_management_for_rent_contract;
pub mod create_management_for_sale_contract;
pub mod create_realty;
pub mod create_rent_contract;
pub mod create_sale_contract;
pub mod create_user;
pub mod create_user_session;
pub mod deplace_contract;
pub mod place_contract;
pub mod terminate_contract;
pub mod update_user_email;
pub mod update_user_name;
pub mod update_user_password;
pub mod update_user_phone;

/// [`Command`] of the [`Service`].
///
/// [`Service`]: crate::Service
pub use common::Handler as Command;

pub use self::{
    authorize_user_session::AuthorizeUserSession,
    create_employment_contract::CreateEmploymentContract,
    create_management_for_rent_contract::CreateManagementForRentContract,
    create_management_for_sale_contract::CreateManagementForSaleContract,
    create_realty::CreateRealty, create_rent_contract::CreateRentContract,
    create_sale_contract::CreateSaleContract, create_user::CreateUser,
    create_user_session::CreateUserSession, deplace_contract::DeplaceContract,
    place_contract::PlaceContract, terminate_contract::TerminateContract,
    update_user_email::UpdateUserEmail, update_user_name::UpdateUserName,
    update_user_password::UpdateUserPassword,
    update_user_phone::UpdateUserPhone,
};
