mod cached_sound;
mod commands;
mod util;

use poise::{
    Framework, FrameworkOptions,
    samples::{register_globally, register_in_guild},
    serenity_prelude::{
        ActivityData, ClientBuilder, Colour, GatewayIntents, GuildId, OnlineStatus,
    },
};
#[cfg(feature = "ai")]
use rig_llama_cpp::Client;
use songbird::{
    SerenityInit,
    driver::Bitrate,
    input::{
        File,
        cached::{Compressed, Memory},
    },
};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;

use crate::{cached_sound::CachedSound, commands::*};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    http_client: reqwest::Client,
    #[cfg(feature = "ai")]
    model: Arc<Client>,
    #[cfg(feature = "ai")]
    model_context: Vec<String>,
    sound_store: Arc<Mutex<HashMap<String, CachedSound>>>,
}

const ACCENT: Colour = Colour(0xffde59);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let http_client = reqwest::Client::new();

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
        use std::fs;

        let mut file_iter = fs::read_dir("./context").await.into_iter();
        while let Some(mut file) = file_iter.next() {
            let entry = file.next_entry().await?.unwrap();
            let entry_type = entry.file_type().await?;

            if entry_type.is_file() {
                model_context.push(fs::read_to_string(entry.path()).await?);
            }
        }
    }

    let mut audio_map = HashMap::new();

    {
        let tikitiki_src = Memory::new(File::new("./audio/tikitiki.ogg").into())
            .await
            .expect("These parameters are well-defined.");
        let _ = tikitiki_src.raw.spawn_loader();
        audio_map.insert("tikitiki".into(), CachedSound::Uncompressed(tikitiki_src));

        // let song_src = Compressed::new(
        //     File::new("./audio/tikitiki.ogg").into(),
        //     Bitrate::Bits(128_000),
        // )
        // .await
        // .expect("These parameters are well-defined.");
        // let _ = song_src.raw.spawn_loader();

        // audio_map.insert("tikitiki".into(), CachedSound::Compressed(song_src));
    }

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                ping(),
                info(),
                join(),
                leave(),
                tikitiki(),
                #[cfg(feature = "ai")]
                chat(),
            ],
            on_error: |err| {
                Box::pin(async move {
                    println!("Error: {}", err);
                })
            },
            ..FrameworkOptions::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                register_globally(ctx, &framework.options().commands).await?;
                register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(
                        env::var("DEV_GUILD")
                            .expect("missing DEV_GUILD")
                            .parse()
                            .expect("DEV_GUILD must be a u64"),
                    ),
                )
                .await?;

                Ok(Data {
                    http_client: http_client.clone(),
                    #[cfg(feature = "ai")]
                    model,
                    #[cfg(feature = "ai")]
                    model_context,
                    sound_store: Arc::new(Mutex::new(audio_map)),
                })
            })
        })
        .build();

    let client = ClientBuilder::new(
        env::var("TOKEN").expect("missing TOKEN"),
        GatewayIntents::privileged(),
    )
    .framework(framework)
    .register_songbird()
    .activity(ActivityData::custom("Watching you sleep"))
    .status(OnlineStatus::Idle)
    .await;
    client.unwrap().start().await.unwrap();

    Ok(())
}
