#[macro_use]
extern crate tracing;

use lavalink_rs::{model::events, prelude::*};
use poise::serenity_prelude::{self as serenity, ActivityData, Colour, OnlineStatus};
#[cfg(feature = "ai")]
use rig_llama_cpp::Client;
use songbird::SerenityInit;
use std::env;
#[cfg(feature = "ai")]
use std::sync::Arc;

pub mod commands;
pub mod music_advanced;
pub mod music_basic;
pub mod music_events;
pub mod util;

pub struct Data {
    #[cfg(feature = "ai")]
    model: Arc<Client>,
    #[cfg(feature = "ai")]
    model_context: Vec<String>,
    lavalink: LavalinkClient,
}

const ACCENT: Colour = Colour(0xffde59);

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Test
#[poise::command(prefix_command)]
async fn test(ctx: Context<'_>) -> Result<(), Error> {
    let lava_client = ctx.data().lavalink.clone();

    let node = lava_client.get_node_by_index(0).unwrap();
    dbg!(node.http.info().await?);

    ctx.say("all good!").await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;

    unsafe {
        env::set_var("RUST_LOG", "info,lavalink_rs=trace");
    }

    tracing_subscriber::fmt::init();

    #[cfg(feature = "ai")]
    let model = Arc::from(
        Client::builder("./models/Qwen3.5-9B-The-Defiant-Fable-Uncnr-Heretic-NEO-MAX-Q4_K_M.gguf")
            .n_ctx(8192)
            .build()?,
    );

    #[cfg(feature = "ai")]
    let mut model_context = vec![];

    #[cfg(feature = "ai")]
    {
        use tokio::fs;

        let mut file_iter = fs::read_dir("./context").await.into_iter();
        while let Some(mut file) = file_iter.next() {
            let entry = file.next_entry().await?.unwrap();
            let entry_type = entry.file_type().await?;

            if entry_type.is_file() {
                model_context.push(fs::read_to_string(entry.path()).await?);
            }
        }
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                music_basic::play(),
                music_basic::join(),
                music_basic::leave(),
                music_advanced::queue(),
                music_advanced::skip(),
                music_advanced::pause(),
                music_advanced::resume(),
                music_advanced::stop(),
                music_advanced::seek(),
                music_advanced::clear(),
                music_advanced::remove(),
                music_advanced::swap(),
                crate::test(),
                commands::ping(),
                commands::info(),
                commands::chat(),
            ],
            on_error: |err| {
                Box::pin(async move {
                    println!("Error: {}", err);
                })
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    serenity::GuildId::new(
                        env::var("DEV_GUILD")
                            .expect("missing DEV_GUILD")
                            .parse()
                            .expect("DEV_GUILD must be a u64"),
                    ),
                )
                .await?;

                let events = events::Events {
                    raw: Some(music_events::raw_event),
                    ready: Some(music_events::ready_event),
                    track_start: Some(music_events::track_start),
                    track_end: Some(music_events::track_end),
                    ..Default::default()
                };

                let node_local = NodeBuilder {
                    hostname: "localhost:2333".to_string(),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: env::var("LAVALINK_PASSWORD").expect("missing LAVALINK_PASSWORD"),
                    user_id: ctx.cache.current_user().id.into(),
                    session_id: None,
                };

                let client = LavalinkClient::new(
                    events,
                    vec![node_local],
                    NodeDistributionStrategy::round_robin(),
                )
                .await;

                Ok(Data {
                    #[cfg(feature = "ai")]
                    model,
                    #[cfg(feature = "ai")]
                    model_context,
                    lavalink: client,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(
        env::var("TOKEN").expect("missing TOKEN"),
        serenity::GatewayIntents::all(),
    )
    .register_songbird()
    .framework(framework)
    .activity(ActivityData::custom("Watching you sleep"))
    .status(OnlineStatus::Idle)
    .await?;

    client.start().await?;

    Ok(())
}
