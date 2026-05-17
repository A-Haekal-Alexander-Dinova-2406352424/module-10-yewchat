use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, SubmitEvent};
use yew::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct ChatMessage {
    sender: String,
    text: String,
}

fn demo_mode() -> bool {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .is_some_and(|search| search.contains("demo=1"))
}

#[function_component(App)]
fn app() -> Html {
    let username = use_state(|| "Haekal Alexander Dinova".to_string());
    let draft = use_state(String::new);
    let status = use_state(|| "Connecting".to_string());
    let messages = use_state(Vec::<ChatMessage>::new);
    let sender = use_mut_ref(|| None::<UnboundedSender<Message>>);

    {
        let messages = messages.clone();
        let sender = sender.clone();
        let status = status.clone();
        let username = username.clone();

        use_effect_with((), move |_| {
            let socket = WebSocket::open("ws://127.0.0.1:8080");

            match socket {
                Ok(socket) => {
                    status.set("Connected to ws://127.0.0.1:8080".to_string());
                    let (mut write, mut read) = socket.split();
                    let (tx, mut rx) = unbounded::<Message>();
                    *sender.borrow_mut() = Some(tx.clone());

                    spawn_local(async move {
                        while let Some(message) = rx.next().await {
                            if write.send(message).await.is_err() {
                                break;
                            }
                        }
                    });

                    let read_messages = messages.clone();
                    let read_status = status.clone();
                    spawn_local(async move {
                        while let Some(incoming) = read.next().await {
                            match incoming {
                                Ok(Message::Text(text)) => {
                                    let chat = serde_json::from_str::<ChatMessage>(&text)
                                        .unwrap_or_else(|_| ChatMessage {
                                            sender: "Server".to_string(),
                                            text,
                                        });
                                    let mut next_messages = (*read_messages).clone();
                                    next_messages.push(chat);
                                    read_messages.set(next_messages);
                                }
                                Ok(Message::Bytes(_)) => {}
                                Err(error) => {
                                    read_status.set(format!("Disconnected: {error:?}"));
                                    break;
                                }
                            }
                        }
                    });

                    if demo_mode() {
                        let demo_tx = tx.clone();
                        let demo_sender = (*username).clone();
                        spawn_local(async move {
                            TimeoutFuture::new(900).await;
                            let message = ChatMessage {
                                sender: demo_sender,
                                text: "Demo message from the Yew websocket client".to_string(),
                            };
                            if let Ok(payload) = serde_json::to_string(&message) {
                                let _ = demo_tx.unbounded_send(Message::Text(payload));
                            }
                        });
                    }
                }
                Err(error) => {
                    status.set(format!("Connection failed: {error:?}"));
                }
            }

            || ()
        });
    }

    let on_username_input = {
        let username = username.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_draft_input = {
        let draft = draft.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            draft.set(input.value());
        })
    };

    let on_submit = {
        let draft = draft.clone();
        let username = username.clone();
        let sender = sender.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let text = (*draft).trim().to_string();

            if text.is_empty() {
                return;
            }

            let message = ChatMessage {
                sender: (*username).clone(),
                text,
            };

            if let (Some(tx), Ok(payload)) =
                (sender.borrow().as_ref(), serde_json::to_string(&message))
            {
                let _ = tx.unbounded_send(Message::Text(payload));
            }

            draft.set(String::new());
        })
    };

    html! {
        <main class="app-shell">
            <section class="chat-panel">
                <header class="topbar">
                    <img class="brand-mark" src="/assets/chat-mark.png" alt="YewChat mark" />
                    <div>
                        <h1>{"YewChat"}</h1>
                        <p>{"Haekal Alexander Dinova"}</p>
                    </div>
                    <span class="status">{(*status).clone()}</span>
                </header>

                <section class="messages" aria-live="polite">
                    {
                        if messages.is_empty() {
                            html! { <p class="empty">{"No messages yet."}</p> }
                        } else {
                            html! {
                                <>
                                    { for messages.iter().map(|message| html! {
                                        <article class="message">
                                            <strong>{&message.sender}</strong>
                                            <span>{&message.text}</span>
                                        </article>
                                    }) }
                                </>
                            }
                        }
                    }
                </section>

                <form class="composer" onsubmit={on_submit}>
                    <input
                        aria-label="Name"
                        class="name-input"
                        value={(*username).clone()}
                        oninput={on_username_input}
                    />
                    <input
                        aria-label="Message"
                        class="message-input"
                        placeholder="Write a message"
                        value={(*draft).clone()}
                        oninput={on_draft_input}
                    />
                    <button type="submit">{"Send"}</button>
                </form>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
