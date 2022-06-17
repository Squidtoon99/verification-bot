use deadpool_redis::redis::AsyncCommands;
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;
use zephyrus::{
    prelude::*,
    twilight_exports::{ChannelMarker, InteractionResponse, InteractionResponseType},
};

#[command]
#[description = "The channel to log joins to"]
async fn channel(
    ctx: &SlashContext<crate::Context>,
    #[description = "To remove, set this value to nothing"] chn: Option<Id<ChannelMarker>>,
) -> CommandResult {
    dbg!(&chn);
    let mut conn = ctx.data.redis.get().await?;

    let current: Option<u64> = conn
        .hget(
            format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
            "logging:channel",
        )
        .await?;

    let desc = match (current, chn.map(|o| o.get())) {
        (None, None) => {
            format!("There is no logging channel set, specify the channel argument to set one.")
        }
        (Some(a), Some(b)) if a == b => format!("The logging channel is already {}", a),
        (_, Some(b)) => {
            let _: () = conn
                .hset(
                    format!("config:{}", ctx.interaction.guild_id.unwrap().get()),
                    "channel",
                    b,
                )
                .await?;
            format!("Set the logging channel to <#{}>", b)
        }
        (Some(_), _) => String::from("Removed the logging channel"),
    };

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .embeds(vec![EmbedBuilder::new().description(desc).build()])
                .build(),
        ),
    })
}
