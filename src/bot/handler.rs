use crate::bot::bot::Bot;
use crate::bot::{commands, interactions};
use crate::settings::CONFIG;
use serenity::all::{GuildId, Interaction, Ready};
use serenity::async_trait;
use serenity::prelude::*;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(CONFIG.discord_server_id);

        match guild_id
            .set_commands(&ctx.http, commands::get_commands())
            .await
        {
            Ok(_) => {
                println!("Successfully registered application commands.");
            }
            Err(e) => {
                eprintln!("Error handling interaction: {:?}", e);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(e) =
            interactions::handle_interaction(&self.database, &self.telnet, &ctx, &interaction).await
        {
            eprintln!("Error handling interaction: {:?}", e);
        }
    }
}
