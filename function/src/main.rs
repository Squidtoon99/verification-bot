#![feature(async_closure)]
use std::env;

use deadpool_redis::redis::AsyncCommands;
use lambda_http::{service_fn, Body, Error, IntoResponse, Request, RequestExt, Response};
use lazy_static::lazy_static;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::Id,
};
use zephyrus::{
    command::Command,
    group::ParentType,
    prelude::*,
    twilight_exports::{
        ApplicationCommand, CommandDataOption, CommandOption, CommandOptionType,
        CommandOptionValue, InteractionResponseData,
    },
};

mod commands;
mod context;
mod error;
mod verification;
use context::Context;
pub use error::Error as CustomError;
// pub use structs::*;
use verification::verify_signature;

lazy_static! {
    #[derive(Debug)]
    static ref PUBLIC_KEY: String = std::env::var("PUBLIC_KEY").unwrap();
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
// / - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(
    event: Request,
    framework: Arc<Framework<Context>>,
) -> Result<impl IntoResponse, Error> {
    // Extract some useful information from the request
    if let (Body::Text(body), Some(interaction)) =
        (event.body(), event.payload::<Interaction>().unwrap())
    {
        // Verify the signature
        let signature = event
            .headers()
            .get("x-signature-ed25519")
            .unwrap()
            .to_str()
            .unwrap();
        let timestamp = event
            .headers()
            .get("x-signature-timestamp")
            .unwrap()
            .to_str()
            .unwrap();
        if let Err(why) = verify_signature(&PUBLIC_KEY, signature, timestamp, body) {
            // Build invalid response

            return Ok(Response::builder()
                .status(401)
                .body(format!("{}", why))
                .map_err(Box::new)?);
        } else {
            Ok(actual_handler(interaction, framework)
                .await
                .expect("Couldn't process cmd"))
        }
    } else {
        panic!("Invalid request");
    }
}

fn get_next(interaction: &mut Vec<CommandDataOption>) -> Option<CommandDataOption> {
    if interaction.len() > 0
        && (interaction[0].value.kind() == CommandOptionType::SubCommand
            || interaction[0].value.kind() == CommandOptionType::SubCommandGroup)
    {
        Some(interaction.remove(0))
    } else {
        None
    }
}

fn get_command<'a>(
    s: &'a Framework<Context>,
    interaction: &mut ApplicationCommand,
) -> Option<&'a Command<Context>> {
    if let Some(next) = get_next(&mut interaction.data.options) {
        let group = s.groups.get(&*interaction.data.name)?;
        match next.value.kind() {
            CommandOptionType::SubCommand => {
                let subcommands = group.kind.as_simple()?;
                let options = match next.value {
                    CommandOptionValue::SubCommand(s) => s,
                    _ => unreachable!(),
                };
                interaction.data.options = options;
                subcommands.get(&*next.name)
            }
            CommandOptionType::SubCommandGroup => {
                let mut options = match next.value {
                    CommandOptionValue::SubCommandGroup(s) => s,
                    _ => unreachable!(),
                };
                let subcommand = get_next(&mut options)?;
                let subgroups = group.kind.as_group()?;
                let group = subgroups.get(&*next.name)?;
                let options = match subcommand.value {
                    CommandOptionValue::SubCommand(s) => s,
                    _ => unreachable!(),
                };
                interaction.data.options = options;
                group.subcommands.get(&*subcommand.name)
            }
            _ => None,
        }
    } else {
        s.commands.get(&*interaction.data.name)
    }
}

async fn actual_handler<'a>(
    interaction: Interaction,
    framework: Arc<Framework<Context>>,
) -> Result<Response<String>, CustomError> {
    let resp: InteractionResponse = match interaction {
        Interaction::Ping(_) => InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None,
        },

        Interaction::ApplicationCommand(mut command) => {
            if let Some(cmd) = get_command(&framework, &mut command) {
                let command = *command;
                let http_client = &framework.http_client;
                let interaction_client = http_client.inner().interaction(framework.application_id);
                let context = SlashContext {
                    http_client,
                    interaction_client,
                    data: &framework.data,
                    interaction: command,
                    application_id: framework.application_id,
                };

                let execute = if let Some(before) = &framework.before {
                    (before.0)(&context, cmd.name).await
                } else {
                    true
                };

                if execute {
                    let result = (cmd.fun)(&context).await;

                    match result {
                        Ok(inner) => inner,
                        Err(why) => InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(InteractionResponseData {
                                content: Some(format!("{}", why)),
                                ..Default::default()
                            }),
                        },
                    }
                } else {
                    InteractionResponse {
                        kind: InteractionResponseType::ChannelMessageWithSource,
                        data: Some(InteractionResponseData {
                            content: Some("Command is disabled".to_string()),
                            ..Default::default()
                        }),
                    }
                }
            } else {
                InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        content: Some("Command not found".to_string()),
                        ..Default::default()
                    }),
                }
            }
        }
        _ => unreachable!(),
    };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json;charset=UTF-8")
        .body(serde_json::to_string(&resp)?)
        .unwrap())
}

#[command]
#[description = "Says hello"]
async fn hello(ctx: &SlashContext<Context>) -> CommandResult {
    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(String::from("Hello world")),
            ..Default::default()
        }),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());

    let context = Context::new(redis_url);

    let http_client = Arc::new(Client::builder().token(token.clone()).build());

    let framework = Arc::new(
        Framework::builder(
            http_client,
            Id::new(std::env::var("APPLICATION_ID").unwrap().parse().unwrap()),
            context.clone(),
        )
        .group(|g| {
            g.name("verification")
                .description("Configuration for member verification")
                .add_command(commands::verification::typ)
                .add_command(commands::verification::role)
        })
        .group(|g| {
            g.name("logging")
                .description("Configuration for how the bot will log member events")
                .add_command(commands::logging::channel)
        })
        .build(),
    );
    {
        let mut conn = context.redis.get().await.expect("Redis connection failed");
        let mut options: Vec<CommandOption> = Vec::new();

        for (_, cmd) in &framework.commands {
            for i in &cmd.fun_arguments {
                options.push(i.as_option());
            }
        }

        for (_, group) in &framework.groups {
            match &group.kind {
                ParentType::Simple(data) => {
                    for (_, cmd) in data {
                        for i in &cmd.fun_arguments {
                            options.push(i.as_option());
                        }
                    }
                }
                _ => {}
            }
        }
        options.sort_by_cached_key(|o| serde_json::to_string(o).unwrap());
        let data = serde_json::to_string(&options)?;
        let mut update = false;
        if let Some(val) = conn
            .get::<_, Option<String>>("commands")
            .await
            .expect("Redis get failed")
        {
            if val != data {
                update = true;
            }
        } else {
            update = true;
        }

        if update {
            conn.set::<_, _, ()>("commands", data)
                .await
                .expect("Redis set failed");
            match framework
                .register_guild_commands(Id::new(639078486434381835))
                .await
            {
                Ok(_) => {
                    println!("Registered guild commands");
                }
                Err(why) => {
                    eprintln!("Failed to register guild commands: {}", why);
                }
            }
        }
    }

    let f_ref = &framework;
    lambda_http::run(service_fn(|request| async {
        function_handler(request, Arc::clone(f_ref)).await
    }))
    .await?;
    Ok(())
}
