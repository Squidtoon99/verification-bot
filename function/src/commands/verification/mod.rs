use deadpool_redis::redis::AsyncCommands;
use twilight_model::{id::Id, guild::{Role}};
use twilight_util::builder::{InteractionResponseDataBuilder, embed::{EmbedBuilder, EmbedFooterBuilder}};
use zephyrus::{
    prelude::*,
    twilight_exports::{InteractionResponse, InteractionResponseType, RoleMarker},
};

#[command]
#[description = "Set the verification role"]
async fn role(
    ctx: &SlashContext<crate::Context>,
    #[description = "The role to assign users once they have completed verification"] role: Option<Id<RoleMarker>>,
) -> CommandResult {
    dbg!(&role);
    let mut conn = ctx.data.redis.get().await?;


    let default_role = if let Some(role) = &role {
        let data: String = conn.get(format!("role:{}:{}", ctx.interaction.guild_id.unwrap().get(), role.get())).await?;
        let rle : Role = serde_json::from_str(&data).unwrap();
        rle.name == "@everyone"
    } else {
        false
    };
    
    let current: Option<u64> = conn
        .hget(
            format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
            "verification:role",
        )
        .await?;
    
    let desc = match (current, role.map(|o|o.get())) {
        (None, None) => {
            format!("The verification role will be given to users once they have completed verification.\nYou can set any role that is currently below the bot's top role.\n\nYou can set the type of verification with `/verification type`.\nTo setup the verification gate run `/verification setup`.")
        }
        (Some(a), Some(b)) if a == b => format!("The verification role is already <@&{}>", a),
        (None, Some(_)) if default_role => format!("There is no verification role set, specify the role argument to set one."),
        (_, Some(_)) if default_role => {
            let _: () = conn
                .hdel(
                    format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
                    "verification:role"
                )
                .await?;
            String::from("Removed the verification role")
        }
        (_, Some(b)) => {
            let _: () = conn
                .hset(
                    format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
                    "verification:role",
                    b,
                )
                .await?;
            format!("Set the verification role to <@&{}>", b)
        }
        (Some(_), None) => String::from("Removed the verification role"),
    };

    let mut embed = EmbedBuilder::new().description(desc);

    if !conn
        .hexists::<_, _, bool>(
            format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
            "verification:message",
        )
        .await? {
            embed = embed.footer(EmbedFooterBuilder::new(":warning: No verification message set. Without one, users will not recieve this role. Set one up with /verification setup."))
        }

    Ok(
        InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new().embeds(vec![
                embed.build()
            ]).build()),
        }
    )
}

#[derive(Parse, Debug, Clone, Eq, PartialEq)]
enum VerificationType {
    None,
    Captcha,
    Audio,
}

impl From<String> for VerificationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "None" => VerificationType::None,
            "Captcha" => VerificationType::Captcha,
            "Audio" => VerificationType::Audio,
            _ => VerificationType::None,
        }
    }
}

impl ToString for VerificationType {
    fn to_string(&self) -> String {
        match self {
            VerificationType::None => String::from("None"),
            VerificationType::Captcha => String::from("Captcha"),
            VerificationType::Audio => String::from("Audio"),
        }
    }
}

#[command("type")]
#[description = "Set the type of verification to use when a user joins the server"]
async fn typ(
    ctx: &SlashContext<crate::Context>,
    #[description = "the type of join gate"] choice: VerificationType,
) -> CommandResult {
    
    let mut conn = ctx.data.redis.get().await?;

    let current : VerificationType = conn
        .hget::<_, _, Option<String>>(
            format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
            "verification:type",
        )
        .await?
        .map(|o| VerificationType::from(o))
        .unwrap_or(VerificationType::None);
    
    let desc = match (current, choice) {
        (a, b) if a == b => format!("The verification type is already `{}`", a.to_string()),
        (_, b) => {
            let _: () = conn
                .hset(
                    format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
                    "type",
                    b.to_string(),
                )
                .await?;
            format!("Set the verification type to `{}`", b.to_string())
        }
    };
    Ok(
        InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new().embeds(vec![
                EmbedBuilder::new().description(desc).build(),
            ]).build()),
        }
    )
}
