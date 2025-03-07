use crate::i18n;
use serenity::all::*;

pub fn get_commands() -> Vec<CreateCommand> {
    vec![register(), find_account(), forget_password(), report()]
}

fn register() -> CreateCommand {
    CreateCommand::new("register")
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
    CreateCommand::new("find_account")
        .description("Register a new account")
        .description_localized(i18n::LANG_ZH_TW, "尋找帳號")
        .description_localized(i18n::LANG_ZH_CN, "寻找帐号")
        .description_localized(i18n::LANG_KO_KR, "계정 찾기")
}

fn forget_password() -> CreateCommand {
    CreateCommand::new("forget_password")
        .description("Forget Password")
        .description_localized(i18n::LANG_ZH_TW, "忘記密碼")
        .description_localized(i18n::LANG_ZH_CN, "忘记密码")
        .description_localized(i18n::LANG_KO_KR, "비밀번호를 잊으셨나요")
}

fn report() -> CreateCommand {
    CreateCommand::new("report")
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
