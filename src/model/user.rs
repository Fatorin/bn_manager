use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct AddUserReq {
    pub username: String,
    pub valid_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddUserResp {
    pub username: String,
    pub password: String,
}