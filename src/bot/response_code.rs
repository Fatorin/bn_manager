#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResponseCode {
    AlreadyRegistered,
    InvalidInput,
    InvalidPasswordInput,
    UserIdTaken(String),
    ServerError,
    RegisterSuccess,
    NotRegistered,
    PasswordReset,
    ReportInvalidInput,
    ReportSuccess,
    LinkSuccess,
}

impl ResponseCode {
    pub fn to_i18n_key(&self) -> &'static str {
        match self {
            ResponseCode::AlreadyRegistered => "already_registered",
            ResponseCode::InvalidInput => "invalid_input",
            ResponseCode::InvalidPasswordInput => "invalid_password_input",
            ResponseCode::UserIdTaken(_) => "user_id_taken",
            ResponseCode::ServerError => "server_error",
            ResponseCode::RegisterSuccess => "register_success",
            ResponseCode::NotRegistered => "not_registered",
            ResponseCode::PasswordReset => "password_reset",
            ResponseCode::ReportInvalidInput => "report_invalid_input",
            ResponseCode::ReportSuccess => "report_success",
            ResponseCode::LinkSuccess => "link_success",
        }
    }
}
