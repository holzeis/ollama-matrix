use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::message::{
    MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
};
use matrix_sdk::{Client, Room, RoomState, config::SyncSettings};
use ollama_rs::Ollama;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use std::time::Duration;
use anyhow::Context;
use tokio::time::sleep;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let username = std::env::var("OLLAMA_USERNAME")
        .context("missing environment variable `OLLAMA_USERNAME`")?;
    let password = std::env::var("OLLAMA_PASSWORD")
        .context("missing environment variable `OLLAMA_PASSWORD`")?;
    let ollama_url = std::env::var("OLLAMA_URL")
        .context("missing environment variable `OLLAMA_URL`")?;
    let ollama_model = std::env::var("OLLAMA_MODEL")
        .context("missing environment variable `OLLAMA_MODEL`")?;
    let homeserver_url = std::env::var("HOMESERVER_URL")
        .context("missing environment variable `HOMESERVER_URL`")?;

    // Note that when encryption is enabled, you should use a persistent store to be
    // able to restore the session with a working encryption setup.
    // See the `persist_session` example.
    let client = Client::builder()
        // We use the convenient client builder to set our custom homeserver URL on it.
        .homeserver_url(homeserver_url)
        .build()
        .await?;

    // Then let's log that client in
    let user_id = client
        .matrix_auth()
        .login_username(username.clone(), &password)
        .initial_device_display_name("Ollama")
        .await?.user_id;


    println!("logged in as {username}");

    client.add_event_handler(on_stripped_state_member);

    let sync_token = client.sync_once(SyncSettings::default()).await?.next_batch;

    let ollama = Ollama::new(ollama_url, 11434);
    let mut history = vec![];

    client.add_event_handler(
        |event: OriginalSyncRoomMessageEvent, room: Room| async move {
            // First, we need to unpack the message: We only want messages from rooms we are
            // still in and that are regular text messages - ignoring everything else.
            if room.state() != RoomState::Joined {
                return;
            }
            let MessageType::Text(text_content) = event.content.msgtype else {
                return;
            };

            if event.sender.eq(&user_id) {
                // ignore messages from ourselves
                return;
            }

            // here comes the actual "logic": when the bot see's a `!party` in the message,
            // it responds
            if text_content.body.contains("!party") {
                let content = RoomMessageEventContent::text_plain("🎉🎊🥳 let's PARTY!! 🥳🎊🎉");

                println!("sending");

                // send our message to the room we found the "!party" command in
                room.send(content).await.unwrap();

                println!("message sent");
            } else {
                let response = ollama
                    .send_chat_messages_with_history(
                        &mut history,
                        ChatMessageRequest::new(
                            ollama_model,
                            vec![ChatMessage::new(
                                MessageRole::User,
                                text_content.clone().body,
                            )],
                        ),
                    )
                    .await
                    .unwrap();

                let content = RoomMessageEventContent::text_plain(response.message.content);
                println!("sending");

                // send our message to the room we found the "!party" command in
                room.send(content).await.unwrap();

                println!("message sent");
            }
        },
    );

    let settings = SyncSettings::default().token(sync_token);

    client.sync(settings).await?;

    Ok(())
}

async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().unwrap() {
        // the invite we've seen isn't for us, but for someone else. ignore
        return;
    }

    // The event handlers are called before the next sync begins, but
    // methods that change the state of a room (joining, leaving a room)
    // wait for the sync to return the new room state so we need to spawn
    // a new task for them.
    tokio::spawn(async move {
        println!("Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.join().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            eprintln!(
                "Failed to join room {} ({err:?}), retrying in {delay}s",
                room.room_id()
            );

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                eprintln!("Can't join room {} ({err:?})", room.room_id());
                break;
            }
        }
        println!("Successfully joined room {}", room.room_id());
    });
}
