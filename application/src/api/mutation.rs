//! GraphQL [`Mutation`]s definitions.

use common::{DateTime, Money, Percent};
use juniper::graphql_object;
use service::{command, query, Command as _};

use crate::{api, define_error, AsError, Context, Error, Session};

/// Root of all GraphQL mutations.
#[derive(Clone, Copy, Debug)]
pub struct Mutation;

impl Mutation {
    /// Name of the [`tracing::Span`] for the mutations.
    const SPAN_NAME: &'static str = "GraphQL mutation";
}

#[graphql_object(context = Context)]
impl Mutation {
    /// Creates a new `User` with the provided credentials and contact info.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `LOGIN_OCCUPIED` - provided `UserLogin` is occupied by another `User`;
    /// - `NO_CONTACT_INFO` - either `UserEmail` or `UserPhone` must be
    ///                       provided.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "createUser",
            email = ?email,
            login = %login,
            name = %name,
            otel.name = Self::SPAN_NAME,
            phone = ?phone,
        ),
    )]
    pub async fn create_user(
        name: api::user::Name,
        login: api::user::Login,
        password: api::user::Password,
        email: Option<api::user::Email>,
        phone: Option<api::user::Phone>,
        ctx: &Context,
    ) -> Result<api::user::session::CreateResult, Error> {
        let user = ctx
            .service()
            .execute(command::CreateUser {
                name: name.into(),
                login: login.into(),
                password: secrecy::SecretBox::init_with(move || {
                    password.into()
                }),
                email: email.map(Into::into),
                phone: phone.map(Into::into),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?;
        let output = ctx
            .service()
            .execute(command::CreateUserSession::ByUserId(user.id))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?;

        ctx.set_current_session(Session {
            user_id: output.user.id.into(),
            token: output.token.clone(),
            expires_at: output.expires_at.coerce(),
        })
        .await;

        Ok(output.into())
    }

    /// Creates a new `UserSession` with the provided credentials.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `WRONG_CREDENTIALS` - provided credentials does not match any `User`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "createUserSession",
            login = %login,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn create_user_session(
        login: api::user::Login,
        password: api::user::Password,
        ctx: &Context,
    ) -> Result<api::user::session::CreateResult, Error> {
        let output = ctx
            .service()
            .execute(command::CreateUserSession::ByCredentials {
                login: login.into(),
                password: secrecy::SecretBox::init_with(move || {
                    password.into()
                }),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?;

        ctx.set_current_session(Session {
            user_id: output.user.id.into(),
            token: output.token.clone(),
            expires_at: output.expires_at.coerce(),
        })
        .await;

        Ok(output.into())
    }

    /// Updates the `User`'s name to the provided one.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "updateUserName",
            name = %name,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn update_user_name(
        name: api::user::Name,
        ctx: &Context,
    ) -> Result<api::User, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::UpdateUserName {
                user_id: my_id.into(),
                name: name.into(),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Updates the `User`'s password to the provided one.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `WRONG_PASSWORD` - provided `old_password` does not match the current
    ///                      `User` password.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "updateUserPassword",
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn update_user_password(
        new_password: api::user::Password,
        old_password: api::user::Password,
        ctx: &Context,
    ) -> Result<api::User, Error> {
        let my_id = ctx.current_session().await?.user_id;

        // TODO: Execute in constant time to avoid timing attacks.
        //       https://en.wikipedia.org/wiki/Timing_attack
        ctx.service()
            .execute(command::UpdateUserPassword {
                user_id: my_id.into(),
                new_password: new_password.into(),
                old_password: old_password.into(),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Updates the `User`'s email to the provided one.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "updateUserEmail",
            email = ?email,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn update_user_email(
        email: Option<api::user::Email>,
        ctx: &Context,
    ) -> Result<api::User, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::UpdateUserEmail {
                user_id: my_id.into(),
                address: email.map(Into::into),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Updates the `User`'s phone to the provided one.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "updateUserPhone",
            phone = ?phone,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn update_user_phone(
        phone: Option<api::user::Phone>,
        ctx: &Context,
    ) -> Result<api::User, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::UpdateUserPhone {
                user_id: my_id.into(),
                number: phone.map(Into::into),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Creates a new `Realty` with the provided details.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            apartment_num = ?apartment_num,
            building_name = %building_name,
            city = %city,
            country = %country,
            floor = ?floor,
            gql.name = "createRealty",
            num_floors = %num_floors,
            otel.name = Self::SPAN_NAME,
            room_num = ?room_num,
            state = ?state,
            street = %street,
            zip_code = ?zip_code,
        ),
    )]
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    pub async fn create_realty(
        country: api::realty::Country,
        state: Option<api::realty::State>,
        city: api::realty::City,
        street: api::realty::Street,
        zip_code: Option<api::realty::ZipCode>,
        building_name: api::realty::BuildingName,
        num_floors: i32,
        floor: Option<i32>,
        apartment_num: Option<api::realty::ApartmentNum>,
        room_num: Option<api::realty::RoomNum>,
        ctx: &Context,
    ) -> Result<api::Realty, Error> {
        let num_floors = num_floors.try_into().map_err(AsError::into_error)?;
        let floor = floor
            .map(TryInto::try_into)
            .transpose()
            .map_err(AsError::into_error)?;

        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Err(api::PrivilegeError::Employer.into());
        }

        ctx.service()
            .execute(command::CreateRealty {
                country: country.into(),
                state: state.map(Into::into),
                city: city.into(),
                street: street.into(),
                zip_code: zip_code.map(Into::into),
                building_name: building_name.into(),
                num_floors,
                floor,
                apartment_num: apartment_num.map(Into::into),
                room_num: room_num.map(Into::into),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Creates a new `EmploymentContract` with the provided details.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `USER_EMPLOYED` - the `User` with the provided ID is already employed;
    /// - `USER_NOT_EXISTS` - the `User` with the provided ID does not exist;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            base_salary = %base_salary,
            description = %description,
            expires_at = ?expires_at.as_ref().map(DateTime::to_rfc3339),
            gql.name = "createEmploymentContract",
            name = %name,
            otel.name = Self::SPAN_NAME,
            user_id = %user_id,
        ),
    )]
    pub async fn create_employment_contract(
        user_id: api::user::Id,
        name: api::contract::Name,
        description: api::contract::Description,
        expires_at: Option<DateTime>,
        base_salary: Money,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::CreateEmploymentContract {
                user_id: user_id.into(),
                initiator_id: my_id.into(),
                name: name.into(),
                description: description.into(),
                expires_at: expires_at.map(DateTime::coerce),
                base_salary,
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Creates a new `ManagementForRentContract` with the provided details.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `REALTY_MANAGED` - the `Realty` with the provided ID is already
    ///                      managed for rent;
    /// - `REALTY_NOT_EXISTS` - the `Realty` with the provided ID does not
    ///                         exist;
    /// - `USER_NOT_EXISTS` - the `User` with the provided ID does not exist;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            description = %description,
            expected_deposit = ?expected_deposit
                .as_ref()
                .map(ToString::to_string),
            expected_price = expected_price.to_string(),
            expires_at = ?expires_at.as_ref().map(DateTime::to_rfc3339),
            gql.name = "createManagementForRentContract",
            landlord_id = %landlord_id,
            make_placement = ?make_placement,
            monthly_fee = ?monthly_fee.as_ref().map(ToString::to_string),
            name = %name,
            one_time_fee = ?one_time_fee.as_ref().map(ToString::to_string),
            otel.name = Self::SPAN_NAME,
            percent_fee = ?percent_fee.as_ref().map(ToString::to_string),
            realty_id = %realty_id,
        ),
    )]
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    pub async fn create_management_for_rent_contract(
        realty_id: api::realty::Id,
        landlord_id: api::user::Id,
        name: api::contract::Name,
        description: api::contract::Description,
        expires_at: Option<DateTime>,
        expected_price: Money,
        expected_deposit: Option<Money>,
        one_time_fee: Option<Money>,
        monthly_fee: Option<Money>,
        percent_fee: Option<Percent>,
        make_placement: Option<bool>,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;
        let make_placement = make_placement.unwrap_or_default();

        ctx.service()
            .execute(command::CreateManagementForRentContract {
                realty_id: realty_id.into(),
                landlord_id: landlord_id.into(),
                employer_id: my_id.into(),
                name: name.into(),
                description: description.into(),
                expires_at: expires_at.map(DateTime::coerce),
                expected_price,
                expected_deposit,
                one_time_fee,
                monthly_fee,
                percent_fee,
                make_placement,
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Creates a new `ManagementForSaleContract` with the provided details.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `REALTY_MANAGED` - the `Realty` with the provided ID is already
    ///                      managed for sale;
    /// - `REALTY_NOT_EXISTS` - the `Realty` with the provided ID does not
    ///                         exist;
    /// - `USER_NOT_EXISTS` - the `User` with the provided ID does not exist;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            description = %description,
            expected_deposit = ?expected_deposit
                .as_ref()
                .map(ToString::to_string),
            expected_price = expected_price.to_string(),
            expires_at = ?expires_at.as_ref().map(DateTime::to_rfc3339),
            gql.name = "createManagementForSaleContract",
            landlord_id = %landlord_id,
            make_placement = ?make_placement,
            monthly_fee = ?monthly_fee.as_ref().map(ToString::to_string),
            name = %name,
            one_time_fee = ?one_time_fee.as_ref().map(ToString::to_string),
            otel.name = Self::SPAN_NAME,
            percent_fee = ?percent_fee.as_ref().map(ToString::to_string),
            realty_id = %realty_id,
        ),
    )]
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    pub async fn create_management_for_sale_contract(
        realty_id: api::realty::Id,
        landlord_id: api::user::Id,
        name: api::contract::Name,
        description: api::contract::Description,
        expires_at: Option<DateTime>,
        expected_price: Money,
        expected_deposit: Option<Money>,
        one_time_fee: Option<Money>,
        monthly_fee: Option<Money>,
        percent_fee: Option<Percent>,
        make_placement: Option<bool>,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;
        let make_placement = make_placement.unwrap_or_default();

        ctx.service()
            .execute(command::CreateManagementForSaleContract {
                realty_id: realty_id.into(),
                landlord_id: landlord_id.into(),
                employer_id: my_id.into(),
                name: name.into(),
                description: description.into(),
                expires_at: expires_at.map(DateTime::coerce),
                expected_price,
                expected_deposit,
                one_time_fee,
                monthly_fee,
                percent_fee,
                make_placement,
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Creates a new `RentContract` with the provided details.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `REALTY_NOT_MANAGED` - the `Realty` with the provided ID is not
    ///                          managed for rent;
    /// - `USER_NOT_EXISTS` - the `User` with the provided ID does not
    ///                       exist;
    /// - `USER_NOT_MANAGER` - the current `User` is not a manager of the
    ///                        `Realty`;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            deposit = ?deposit.as_ref().map(ToString::to_string),
            description = %description,
            expires_at = ?expires_at.as_ref().map(DateTime::to_rfc3339),
            gql.name = "createRentContract",
            name = %name,
            otel.name = Self::SPAN_NAME,
            price = price.to_string(),
            purchaser_id = %purchaser_id,
            realty_id = %realty_id,
        ),
    )]
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    pub async fn create_rent_contract(
        realty_id: api::realty::Id,
        purchaser_id: api::user::Id,
        name: api::contract::Name,
        description: api::contract::Description,
        expires_at: Option<DateTime>,
        price: Money,
        deposit: Option<Money>,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::CreateRentContract {
                realty_id: realty_id.into(),
                employer_id: my_id.into(),
                purchaser_id: purchaser_id.into(),
                name: name.into(),
                description: description.into(),
                expires_at: expires_at.map(DateTime::coerce),
                price,
                deposit,
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Creates a new `SaleContract` with the provided details.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `REALTY_MANAGED_FOR_RENTED` - the `Realty` with the provided ID is
    ///                                 managed for rent;
    /// - `REALTY_RENTED` - the `Realty` with the provided ID is rented;
    /// - `REALTY_NOT_MANAGED` - the `Realty` with the provided ID is not
    ///                          managed for sale;
    /// - `USER_NOT_EXISTS` - the `User` with the provided ID does not exist;
    /// - `USER_NOT_MANAGER` - the current `User` is not a manager of the
    ///                        `Realty`;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            deposit = ?deposit.as_ref().map(ToString::to_string),
            description = %description,
            expires_at = ?expires_at.as_ref().map(DateTime::to_rfc3339),
            gql.name = "createSaleContract",
            name = %name,
            otel.name = Self::SPAN_NAME,
            price = price.to_string(),
            purchaser_id = %purchaser_id,
            realty_id = %realty_id,
        ),
    )]
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    pub async fn create_sale_contract(
        realty_id: api::realty::Id,
        purchaser_id: api::user::Id,
        name: api::contract::Name,
        description: api::contract::Description,
        expires_at: Option<DateTime>,
        price: Money,
        deposit: Option<Money>,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::CreateSaleContract {
                realty_id: realty_id.into(),
                employer_id: my_id.into(),
                purchaser_id: purchaser_id.into(),
                name: name.into(),
                description: description.into(),
                expires_at: expires_at.map(DateTime::coerce),
                price,
                deposit,
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Terminates the `Contract` with the provided ID.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `CONTRACT_NOT_EXISTS` - the `Contract` with the provided ID does not
    ///                           exist or terminated already.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "terminateContract",
            id = %id,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn terminate_contract(
        id: api::contract::Id,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;

        ctx.service()
            .execute(command::TerminateContract {
                contract_id: id.into(),
                initiator_id: my_id.into(),
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Toggles the placement of the `Contract` with the provided ID.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `CONTRACT_ALREADY_PLACED` - the `Contract` with the provided ID is
    ///                               already placed (if `isPlaced` is `true`);
    /// - `CONTRACT_NOT_PLACED` - the `Contract` with the provided ID is not
    ///                           placed (if `isPlaced` is `false`);
    /// - `CONTRACT_NOT_EXISTS` - the `Contract` with the provided ID does not
    ///                           exist;
    /// - `UNSUPPORTED_CONTRACT` - the `Contract` with the provided ID is not
    ///                            supported for placement;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "toggleContractPlacement",
            id = %id,
            is_placed = %is_placed,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn toggle_contract_placement(
        id: api::contract::Id,
        is_placed: bool,
        ctx: &Context,
    ) -> Result<api::ContractValue, Error> {
        let my_id = ctx.current_session().await?.user_id;

        if is_placed {
            ctx.service()
                .execute(command::PlaceContract {
                    contract_id: id.into(),
                    initiator_id: my_id.into(),
                })
                .await
                .map_err(AsError::into_error)
                .map_err(ctx.error())
                .map(Into::into)
        } else {
            ctx.service()
                .execute(command::DeplaceContract {
                    contract_id: id.into(),
                    initiator_id: my_id.into(),
                })
                .await
                .map_err(AsError::into_error)
                .map_err(ctx.error())
                .map(Into::into)
        }
    }
}

impl AsError for command::create_user::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "LOGIN_OCCUPIED"]
                #[status = CONFLICT]
                #[message = "`UserLogin` is occupied by another \
                             `User`"]
                LoginOccupied,

                #[code = "NO_CONTACT_INFO"]
                #[status = BAD_REQUEST]
                #[message = "Either `UserEmail` or `UserPhone` must be \
                             provided"]
                NoContactInfo,
            }
        }

        match self {
            Self::Db(e) => e.try_as_error(),
            Self::LoginOccupied(_) => Some(Error::LoginOccupied.into()),
            Self::NoContactInfo => Some(Error::NoContactInfo.into()),
        }
    }
}

impl AsError for command::create_user_session::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "WRONG_CREDENTIALS"]
                #[status = FORBIDDEN]
                #[message = "Provided credentials does not match any `User`"]
                WrongCredentials,
            }
        }

        match self {
            Self::Db(e) => e.try_as_error(),
            Self::JsonWebTokenEncodeError(_) => None,
            Self::UserNotExists(_) | Self::WrongCredentials => {
                Some(Error::WrongCredentials.into())
            }
        }
    }
}

impl AsError for command::update_user_name::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        match self {
            Self::Db(e) => e.try_as_error(),
            Self::UserNotExists(_) => None,
        }
    }
}

impl AsError for command::update_user_password::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "WRONG_PASSWORD"]
                #[status = CONFLICT]
                #[message = "Provided `old_password` does not match the \
                             current `User` password"]
                WrongPassword,
            }
        }

        match self {
            Self::Db(e) => e.try_as_error(),
            Self::UserNotExists(_) => None,
            Self::WrongPassword => Some(Error::WrongPassword.into()),
        }
    }
}

impl AsError for command::update_user_email::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        match self {
            Self::Db(e) => e.try_as_error(),
            Self::UserNotExists(_) => None,
        }
    }
}

impl AsError for command::update_user_phone::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        match self {
            Self::Db(e) => e.try_as_error(),
            Self::UserNotExists(_) => None,
        }
    }
}

impl AsError for command::create_employment_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "USER_EMPLOYED"]
                #[status = CONFLICT]
                #[message = "`User` with the provided ID is already employed"]
                UserAlreadyEmployed,

                #[code = "USER_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`User` with the provided ID is not exists"]
                UserNotExists,
            }
        }

        Some(match self {
            Self::Db(e) => return e.try_as_error(),
            Self::UserAlreadyEmployed(_) => Error::UserAlreadyEmployed.into(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => Error::UserNotExists.into(),
        })
    }
}

impl AsError for command::create_management_for_rent_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "REALTY_MANAGED"]
                #[status = CONFLICT]
                #[message = "`Realty` with the provided ID is already managed \
                             for rent"]
                RealtyAlreadyManaged,

                #[code = "REALTY_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`Realty` with the provided ID is not exists"]
                RealtyNotExists,

                #[code = "USER_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`User` with the provided ID is not exists"]
                UserNotExists,
            }
        }

        Some(match self {
            Self::Db(e) => return e.try_as_error(),
            Self::RealtyAlreadyManaged(_) => Error::RealtyAlreadyManaged.into(),
            Self::RealtyNotExists(_) => Error::RealtyNotExists.into(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => Error::UserNotExists.into(),
        })
    }
}

impl AsError for command::create_management_for_sale_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "REALTY_MANAGED"]
                #[status = CONFLICT]
                #[message = "`Realty` with the provided ID is already managed \
                             for sale"]
                RealtyAlreadyManaged,

                #[code = "REALTY_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`Realty` with the provided ID is not exists"]
                RealtyNotExists,

                #[code = "USER_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`User` with the provided ID is not exists"]
                UserNotExists,
            }
        }

        Some(match self {
            Self::Db(e) => return e.try_as_error(),
            Self::RealtyAlreadyManaged(_) => Error::RealtyAlreadyManaged.into(),
            Self::RealtyNotExists(_) => Error::RealtyNotExists.into(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => Error::UserNotExists.into(),
        })
    }
}

impl AsError for command::create_rent_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "REALTY_NOT_MANAGED"]
                #[status = FORBIDDEN]
                #[message = "`Realty` with the provided ID is not managed \
                             for rent"]
                RealtyNotManaged,

                #[code = "USER_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`User` with the provided ID is not exists"]
                UserNotExists,

                #[code = "USER_NOT_MANAGER"]
                #[status = FORBIDDEN]
                #[message = "Authenticated `User` is not manager of the \
                             `Realty`"]
                UserNotManager,
            }
        }

        Some(match self {
            Self::Db(e) => return e.try_as_error(),
            Self::RealtyNotManaged(_) => Error::RealtyNotManaged.into(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => Error::UserNotExists.into(),
            Self::UserNotManager(_) => Error::UserNotManager.into(),
        })
    }
}

impl AsError for command::create_sale_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "REALTY_MANAGED_FOR_RENTED"]
                #[status = CONFLICT]
                #[message = "`Realty` with the provided ID is managed rent"]
                RealtyManagedForRent,

                #[code = "REALTY_RENTED"]
                #[status = CONFLICT]
                #[message = "`Realty` with the provided ID is rented"]
                RealtyRented,

                #[code = "REALTY_NOT_MANAGED"]
                #[status = FORBIDDEN]
                #[message = "`Realty` with the provided ID is not managed \
                             for sale"]
                RealtyNotManaged,

                #[code = "USER_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`User` with the provided ID is not exists"]
                UserNotExists,

                #[code = "USER_NOT_MANAGER"]
                #[status = FORBIDDEN]
                #[message = "Authenticated `User` is not manager of the \
                             `Realty`"]
                UserNotManager,
            }
        }

        Some(match self {
            Self::Db(e) => return e.try_as_error(),
            Self::RealtyManagedForRent(_) => Error::RealtyManagedForRent.into(),
            Self::RealtyNotManaged(_) => Error::RealtyNotManaged.into(),
            Self::RealtyRented(_) => Error::RealtyRented.into(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => Error::UserNotExists.into(),
            Self::UserNotManager(_) => Error::UserNotManager.into(),
        })
    }
}

impl AsError for command::terminate_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "CONTRACT_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`Contract` with the provided ID is not exists or \
                             terminated already"]
                ContractNotExists,
            }
        }

        Some(match self {
            Self::ContractAlreadyTerminated(_) | Self::ContractNotExists(_) => {
                Error::ContractNotExists.into()
            }
            Self::Db(e) => return e.try_as_error(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => return None,
        })
    }
}

impl AsError for command::place_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "CONTRACT_ALREADY_PLACED"]
                #[status = CONFLICT]
                #[message = "`Contract` with the provided ID is already placed"]
                ContractAlreadyPlaced,

                #[code = "CONTRACT_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`Contract` with the provided ID is not exists"]
                ContractNotExists,

                #[code = "UNSUPPORTED_CONTRACT"]
                #[status = BAD_REQUEST]
                #[message = "Placement of `Contract` with the provided ID is \
                             not supported"]
                UnsupportedContract,
            }
        }

        Some(match self {
            Self::ContractAlreadyPlaced(_) => {
                Error::ContractAlreadyPlaced.into()
            }
            Self::ContractNotExists(_) => Error::ContractNotExists.into(),
            Self::Db(e) => return e.try_as_error(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => return None,
            Self::UnsupportedContract(_) => Error::UnsupportedContract.into(),
        })
    }
}

impl AsError for command::deplace_contract::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        define_error! {
            enum Error {
                #[code = "CONTRACT_NOT_PLACED"]
                #[status = CONFLICT]
                #[message = "`Contract` with the provided ID is not placed"]
                ContractNotPlaced,

                #[code = "CONTRACT_NOT_EXISTS"]
                #[status = NOT_FOUND]
                #[message = "`Contract` with the provided ID is not exists"]
                ContractNotExists,

                #[code = "UNSUPPORTED_CONTRACT"]
                #[status = BAD_REQUEST]
                #[message = "Placement of `Contract` with the provided ID is \
                             not supported"]
                UnsupportedContract,
            }
        }

        Some(match self {
            Self::ContractNotPlaced(_) => Error::ContractNotPlaced.into(),
            Self::ContractNotExists(_) => Error::ContractNotExists.into(),
            Self::Db(e) => return e.try_as_error(),
            Self::UserNotEmployer(_) => api::PrivilegeError::Employer.into(),
            Self::UserNotExists(_) => return None,
            Self::UnsupportedContract(_) => Error::UnsupportedContract.into(),
        })
    }
}
