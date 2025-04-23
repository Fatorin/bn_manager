use crate::bot::commands::CommandType;
use crate::bot::query::{create_user, get_user_by_discord_id};
use crate::bot::response_code::ResponseCode;
use crate::i18n::I18N;
use crate::model::user::User;
use crate::settings::CONFIG;
use crate::telnet::{ApiResult, Command};
use crate::{telnet, util};
use rand::Rng;
use regex::Regex;
use serenity::all::{
    ChannelId, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, Interaction,
    Timestamp,
};
use serenity::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub async fn handle_interaction(
    db: &sqlx::sqlite::SqlitePool,
    client: &telnet::ApiClient,
    ctx: &Context,
    interaction: &Interaction,
) -> serenity::Result<(), Error> {
    match interaction {
        Interaction::Command(command) => match CommandType::from_str(&command.data.name) {
            Ok(cmd) => match cmd {
                CommandType::Register => handle_register(db, ctx, interaction).await?,
                CommandType::FindAccount => handle_find_account(db, ctx, interaction).await?,
                CommandType::LinkAccount => handle_link_account(db, ctx, interaction).await?,
                CommandType::ChangePassword => {
                    handle_change_password(db, client, ctx, interaction).await?
                }
                CommandType::Report => handle_report(ctx, interaction).await?,
            },
            Err(err) => eprintln!("unknown interaction, ex:{:?}", err),
        },
        _ => eprintln!("unknown interaction"),
    }

    Ok(())
}

async fn handle_register(
    db: &sqlx::sqlite::SqlitePool,
    ctx: &Context,
    interaction: &Interaction,
) -> serenity::Result<(), Error> {
    if let Interaction::Command(command) = interaction {
        let locale = command.locale.as_str();
        let discord_id = command.user.id.to_string();

        if let Err(message) = check_exist_user(db, &discord_id, locale).await {
            command_send_message(ctx, command, message).await?;
            return Ok(());
        }

        let options = &command.data.options;

        let username = options
            .iter()
            .find(|opt| opt.name == "username")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or_default();

        let password = match write_user_data(&username) {
            Ok(password) => password,
            Err(err) => {
                println!("create user failed, ex:{:?}", err);
                command_send_message(ctx, command, I18N.get(err.to_i18n_key(), locale)).await?;
                return Ok(());
            }
        };

        if let Err(err) = create_user(db, &discord_id, username).await {
            println!("create db user failed, ex:{}", err);
            command_send_message(
                ctx,
                command,
                I18N.get(ResponseCode::ServerError.to_i18n_key(), locale),
            )
            .await?;
            return Ok(());
        }

        let args = vec![("username", username), ("password", &password)];
        let message =
            I18N.get_with_args(ResponseCode::RegisterSuccess.to_i18n_key(), locale, &args);
        command_send_message(ctx, command, message).await?;
    }

    Ok(())
}

async fn handle_find_account(
    db: &sqlx::sqlite::SqlitePool,
    ctx: &Context,
    interaction: &Interaction,
) -> serenity::Result<(), Error> {
    if let Interaction::Command(command) = interaction {
        let locale = command.locale.as_str();
        let discord_id = command.user.id.to_string();
        let user_result = find_user(db, &discord_id).await;

        return match user_result {
            Ok(user) => {
                command_send_message(
                    ctx,
                    command,
                    I18N.get_with_arg(
                        ResponseCode::AlreadyRegistered.to_i18n_key(),
                        locale,
                        "username",
                        &user.username,
                    ),
                )
                .await?;
                Ok(())
            }
            Err(err) => {
                command_send_message(ctx, command, I18N.get(err.to_i18n_key(), locale)).await?;
                Ok(())
            }
        };
    }

    Ok(())
}

async fn handle_link_account(
    db: &sqlx::sqlite::SqlitePool,
    ctx: &Context,
    interaction: &Interaction,
) -> serenity::Result<(), Error> {
    if let Interaction::Command(command) = interaction {
        let locale = command.locale.as_str();
        let discord_id = command.user.id.to_string();

        if let Err(message) = check_exist_user(db, &discord_id, locale).await {
            command_send_message(ctx, command, message).await?;
            return Ok(());
        }

        let options = &command.data.options;

        let username = options
            .iter()
            .find(|opt| opt.name == "username")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or_default();

        let password = options
            .iter()
            .find(|opt| opt.name == "password")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or_default();

        if username.is_empty() || password.is_empty() {
            command_send_message(
                ctx,
                command,
                I18N.get(ResponseCode::InvalidInput.to_i18n_key(), locale),
            )
            .await?;
            return Ok(());
        }

        if let Err(err) =
            util::file::verify_user_credentials(&CONFIG.user_data_path, &username, &password)
        {
            command_send_message(ctx, command, I18N.get(err.to_i18n_key(), locale)).await?;
            return Ok(());
        }

        if let Err(err) = create_user(db, &discord_id, username).await {
            println!("create db user to link failed, ex:{}", err);
            command_send_message(
                ctx,
                command,
                I18N.get(ResponseCode::ServerError.to_i18n_key(), locale),
            )
            .await?;
            return Ok(());
        }

        let message = I18N.get(ResponseCode::LinkSuccess.to_i18n_key(), locale);
        command_send_message(ctx, command, message).await?;
    }

    Ok(())
}

async fn handle_change_password(
    db: &sqlx::sqlite::SqlitePool,
    telnet: &telnet::ApiClient,
    ctx: &Context,
    interaction: &Interaction,
) -> serenity::Result<(), Error> {
    if let Interaction::Command(command) = interaction {
        let locale = command.locale.as_str();
        let discord_id = command.user.id.to_string();

        let options = &command.data.options;

        let password = options
            .iter()
            .find(|opt| opt.name == "password")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or_default();

        let username = match change_password(db, telnet, &discord_id, password).await {
            Ok(username) => username,
            Err(err) => {
                let message = I18N.get(err.to_i18n_key(), locale);
                command_send_message(ctx, command, message).await?;
                return Ok(());
            }
        };

        let args = vec![("username", username.as_str()), ("password", password)];
        let message = I18N.get_with_args(ResponseCode::PasswordReset.to_i18n_key(), locale, &args);
        command_send_message(ctx, command, message).await?;
    }
    Ok(())
}

async fn handle_report(ctx: &Context, interaction: &Interaction) -> serenity::Result<(), Error> {
    if let Interaction::Command(command) = interaction {
        let locale = command.locale.as_str();

        let options = &command.data.options;

        let username = options
            .iter()
            .find(|opt| opt.name == "username")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or_default();

        let reason_code = options
            .iter()
            .find(|opt| opt.name == "reason")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("unknown");

        let reason = match reason_code {
            "game_leaving" => "Game Leaving",
            "misconduct" => "Misconduct",
            _ => reason_code,
        };

        let comment = options
            .iter()
            .find(|opt| opt.name == "comment")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("");

        let attachment = options
            .iter()
            .find(|opt| opt.name == "attachment")
            .and_then(|opt| {
                if let CommandDataOptionValue::Attachment(attachment_id) = &opt.value {
                    Some(attachment_id)
                } else {
                    None
                }
            })
            .and_then(|id| command.data.resolved.attachments.get(id));

        if username.is_empty() || reason_code.is_empty() || attachment.is_none() {
            command_send_message(
                ctx,
                command,
                I18N.get(ResponseCode::ReportInvalidInput.to_i18n_key(), locale),
            )
            .await?;
            return Ok(());
        }

        if let Some(attachment_data) = attachment {
            let filename = &attachment_data.filename;
            let is_png = filename.to_lowercase().ends_with(".png");
            let is_img = filename.to_lowercase().ends_with(".img");
            let is_w3g = filename.to_lowercase().ends_with(".w3g");

            let is_supported_file = is_png || is_img || is_w3g;

            if !is_supported_file {
                command_send_message(
                    ctx,
                    command,
                    I18N.get(ResponseCode::ReportInvalidInput.to_i18n_key(), locale),
                )
                .await?;
                return Ok(());
            }
        }

        command_send_message(
            ctx,
            command,
            I18N.get(ResponseCode::ReportSuccess.to_i18n_key(), locale),
        )
        .await?;

        let mut embed = CreateEmbed::new()
            .title("User Report")
            .color(0xff0000) // 紅色
            .field("Reported Player", username, false)
            .field("Reason", reason, false)
            .field("Reporter", &command.user.name, false)
            .timestamp(Timestamp::now());

        if !comment.is_empty() {
            embed = embed.field("Comment", comment, false);
        }

        if let Some(attachment_data) = attachment {
            embed = embed.field(
                "Attachment",
                format!("[Click Download]({})", attachment_data.url),
                false,
            );

            if attachment_data
                .content_type
                .as_ref()
                .map_or(false, |ct| ct.starts_with("image/"))
            {
                embed = embed.image(&attachment_data.url);
            }
        }

        let message = CreateMessage::new().embed(embed);

        let report_channel_id = ChannelId::new(CONFIG.discord_report_channel_id);

        report_channel_id.send_message(&ctx.http, message).await?;
    }

    Ok(())
}

async fn command_send_message(
    ctx: &Context,
    command: &CommandInteraction,
    message: String,
) -> serenity::Result<()> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(message)
                    .ephemeral(true),
            ),
        )
        .await
}

fn create_random_password() -> Option<String> {
    let mut rng = rand::rng();
    let random_number: u32 = rng.random_range(0..100_000_000);
    let password = format!("fate{:08}", random_number);
    let _pwd_hash = match pvpgn_hash_rs::get_hash_string(&password) {
        Ok(hash) => hash,
        Err(_) => return None,
    };

    Some(password)
}

fn write_user_data(username: &str) -> Result<String, ResponseCode> {
    if !check_username_valid(&username) {
        return Err(ResponseCode::InvalidInput);
    }

    let mut template_data: String = match fs::read_to_string("./template/user_template_data.dat") {
        Ok(data) => data,
        Err(_) => {
            println!("user not template data");
            return Err(ResponseCode::ServerError);
        }
    };

    let uid_offset = CONFIG.uid_offset;

    let file_path = Path::new(&CONFIG.user_data_path).join(username);

    let mut uid: usize = 1 + uid_offset as usize;

    let index = match util::file::check_exist(&CONFIG.user_data_path, username) {
        Ok(count) => count,
        Err(err) => return Err(err),
    };

    uid += index;

    let password = match create_random_password() {
        Some(password) => password,
        None => {
            println!("create random password failed");
            return Err(ResponseCode::ServerError);
        }
    };

    let pwd_hash = match pvpgn_hash_rs::get_hash_string(&password) {
        Ok(pwd) => pwd,
        Err(_) => {
            println!("can't create hash password");
            return Err(ResponseCode::ServerError);
        }
    };

    template_data = template_data.replace("{{ userid }}", &uid.to_string());
    template_data = template_data.replace("{{ username }}", username);
    template_data = template_data.replace("{{ password }}", &pwd_hash);

    match fs::write(file_path, template_data.as_bytes()) {
        Ok(()) => Ok(password),
        Err(err) => {
            println!("can't write file, ex:{}", err);
            Err(ResponseCode::ServerError)
        }
    }
}

async fn check_exist_user(
    db: &sqlx::sqlite::SqlitePool,
    discord_id: &str,
    locale: &str,
) -> Result<(), String> {
    match find_user(db, &discord_id).await {
        Ok(user) => {
            return Err(I18N.get_with_arg(
                ResponseCode::AlreadyRegistered.to_i18n_key(),
                locale,
                "username",
                &user.username,
            ))
        }
        Err(err) => {
            if err != ResponseCode::NotRegistered {
                return Err(I18N.get(ResponseCode::ServerError.to_i18n_key(), locale));
            }
        }
    }

    Ok(())
}

async fn find_user(db: &sqlx::sqlite::SqlitePool, discord_id: &str) -> Result<User, ResponseCode> {
    let user_result = get_user_by_discord_id(db, &discord_id).await;
    match user_result {
        Ok(maybe_user) => match maybe_user {
            Some(user) => Ok(user),
            None => Err(ResponseCode::NotRegistered),
        },
        Err(err) => {
            println!("An error occurred while querying the user, ex:{:?}", err);
            Err(ResponseCode::ServerError)
        }
    }
}

async fn change_password(
    db: &sqlx::sqlite::SqlitePool,
    client: &telnet::ApiClient,
    discord_id: &str,
    password: &str,
) -> Result<String, ResponseCode> {
    let user_result = find_user(db, discord_id).await;

    let user = match user_result {
        Ok(user) => user,
        Err(err) => return Err(err),
    };

    if !check_password_valid(password) {
        return Err(ResponseCode::InvalidPasswordInput);
    }

    let resp = client
        .send_command(Command::ChangePassword(
            user.username.to_string(),
            password.to_string(),
        ))
        .await?;

    match resp {
        ApiResult::Success(_) => Ok(user.username),
        _ => Err(ResponseCode::ServerError),
    }
}

fn check_username_valid(user_id: &str) -> bool {
    if user_id.is_empty() {
        return false;
    }

    let regex = Regex::new(r"^[a-zA-Z0-9\[\]\-_.]{4,20}$").unwrap();
    regex.is_match(user_id)
}

fn check_password_valid(password: &str) -> bool {
    if password.is_empty() {
        return false;
    }

    let regex = Regex::new(r"^[a-zA-Z0-9]{4,20}$").unwrap();
    regex.is_match(password)
}
