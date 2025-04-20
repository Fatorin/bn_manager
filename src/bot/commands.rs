use crate::i18n;
use serenity::all::*;

const COMMAND_REGISTER: &'static str = "register";
const COMMAND_FIND_ACCOUNT: &'static str = "find_account";
const COMMAND_LINK_ACCOUNT: &'static str = "link_account";
const COMMAND_CHANGE_PASSWORD: &'static str = "chpass";
const COMMAND_REPORT: &'static str = "report";

#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    Register,
    FindAccount,
    LinkAccount,
    ChangePassword,
    Report,
}

impl CommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandType::Register => COMMAND_REGISTER,
            CommandType::FindAccount => COMMAND_FIND_ACCOUNT,
            CommandType::LinkAccount => COMMAND_LINK_ACCOUNT,
            CommandType::ChangePassword => COMMAND_CHANGE_PASSWORD,
            CommandType::Report => COMMAND_REPORT,
        }
    }
}

impl From<CommandType> for String {
    fn from(cmd: CommandType) -> Self {
        cmd.as_str().to_string()
    }
}

impl std::str::FromStr for CommandType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            COMMAND_REGISTER => Ok(CommandType::Register),
            COMMAND_FIND_ACCOUNT => Ok(CommandType::FindAccount),
            COMMAND_LINK_ACCOUNT => Ok(CommandType::LinkAccount),
            COMMAND_CHANGE_PASSWORD => Ok(CommandType::ChangePassword),
            COMMAND_REPORT => Ok(CommandType::Report),
            _ => Err("unknown command".to_string()),
        }
    }
}

pub fn get_commands() -> Vec<CreateCommand> {
    vec![
        register(),
        find_account(),
        link_account(),
        change_password(),
        report(),
    ]
}

fn register() -> CreateCommand {
    CreateCommand::new(CommandType::Register)
        .description("Register Account")
        .description_localized(i18n::LANG_ZH_TW, "註冊新帳號")
        .description_localized(i18n::LANG_ZH_CN, "注册新账号")
        .description_localized(i18n::LANG_KO_KR, "새 계정을 등록하다")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "username", "UserName")
                .description_localized(i18n::LANG_ZH_TW, "使用者名稱")
                .description_localized(i18n::LANG_ZH_CN, "用戶名")
                .description_localized(i18n::LANG_KO_KR, "사용자 이름")
                .required(true),
        )
}

fn find_account() -> CreateCommand {
    CreateCommand::new(CommandType::FindAccount)
        .description("Find your account")
        .description_localized(i18n::LANG_ZH_TW, "尋找帳號")
        .description_localized(i18n::LANG_ZH_CN, "寻找帐号")
        .description_localized(i18n::LANG_KO_KR, "계정 찾기")
}

fn link_account() -> CreateCommand {
    CreateCommand::new(CommandType::LinkAccount)
        .description("Link your account")
        .description_localized(i18n::LANG_ZH_TW, "連結帳號")
        .description_localized(i18n::LANG_ZH_CN, "链接账号")
        .description_localized(i18n::LANG_KO_KR, "계정 연결")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "username", "UserName")
                .description_localized(i18n::LANG_ZH_TW, "使用者名稱")
                .description_localized(i18n::LANG_ZH_CN, "用戶名")
                .description_localized(i18n::LANG_KO_KR, "사용자 이름")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "password", "Password")
                .description_localized(i18n::LANG_ZH_TW, "密碼")
                .description_localized(i18n::LANG_ZH_CN, "密码")
                .description_localized(i18n::LANG_KO_KR, "비밀번호")
                .required(true),
        )
}

fn change_password() -> CreateCommand {
    CreateCommand::new(CommandType::ChangePassword)
        .description("Change password")
        .description_localized(i18n::LANG_ZH_TW, "變更密碼")
        .description_localized(i18n::LANG_ZH_CN, "变更密码")
        .description_localized(i18n::LANG_KO_KR, "비밀번호 변경")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "password", "Password")
                .description_localized(i18n::LANG_ZH_TW, "密碼")
                .description_localized(i18n::LANG_ZH_CN, "密码")
                .description_localized(i18n::LANG_KO_KR, "비밀번호")
                .required(true),
        )
}

fn report() -> CreateCommand {
    CreateCommand::new(CommandType::Report)
        .description("Report Player")
        .description_localized(i18n::LANG_ZH_TW, "檢舉玩家")
        .description_localized(i18n::LANG_ZH_CN, "举报玩家")
        .description_localized(i18n::LANG_KO_KR, "플레이어 신고")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "username", "UserName")
                .description_localized(i18n::LANG_ZH_TW, "使用者名稱")
                .description_localized(i18n::LANG_ZH_CN, "用戶名")
                .description_localized(i18n::LANG_KO_KR, "사용자 이름")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Attachment,
                "attachment",
                "Replay or screenshot",
            )
            .description_localized(i18n::LANG_ZH_TW, "重播檔案或截圖")
            .description_localized(i18n::LANG_ZH_CN, "回放文件或截屏")
            .description_localized(i18n::LANG_KO_KR, "사용자 스크린샷")
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "reason", "Reason for report")
                .description_localized(i18n::LANG_ZH_TW, "檢舉原因")
                .description_localized(i18n::LANG_ZH_CN, "举报原因")
                .description_localized(i18n::LANG_KO_KR, "신고 이유")
                .required(true)
                .add_string_choice_localized(
                    "Game Leaving",
                    "game_leaving",
                    [
                        (i18n::LANG_ZH_TW, "遊戲中離"),
                        (i18n::LANG_ZH_CN, "游戏中离"),
                        (i18n::LANG_KO_KR, "게임 이탈"),
                    ],
                )
                .add_string_choice_localized(
                    "Misconduct",
                    "misconduct",
                    [
                        (i18n::LANG_ZH_TW, "惡意行為"),
                        (i18n::LANG_ZH_CN, "恶意行为"),
                        (i18n::LANG_KO_KR, "부정행위"),
                    ],
                ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "comment",
                "Additional comments (optional)",
            )
            .description_localized(i18n::LANG_ZH_TW, "附加說明（選填）")
            .description_localized(i18n::LANG_ZH_CN, "附加说明（选填）")
            .description_localized(i18n::LANG_KO_KR, "추가 설명 (선택 사항)")
            .required(false),
        )
}
