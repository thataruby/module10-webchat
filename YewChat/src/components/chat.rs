use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    Typing,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
    time: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
    Typing,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
    is_online: bool,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    typing_user: Option<String>,
}

const USER_COLORS: [&str; 12] = [
    "bg-pink-200",
    "bg-yellow-200",
    "bg-green-200",
    "bg-blue-200",
    "bg-purple-200",
    "bg-orange-200",
    "bg-red-200",
    "bg-indigo-200",
    "bg-teal-200",
    "bg-lime-200",
    "bg-amber-200",
    "bg-cyan-200",
];

fn color_for_user(name: &str) -> &'static str {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let index = (hasher.finish() as usize) % USER_COLORS.len();
    USER_COLORS[index]
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            typing_user: None,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let username = user.username.borrow().clone();

        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        let current_names: Vec<String> = users_from_message.iter().cloned().collect();
                        self.users = current_names
                            .iter()
                            .map(|u| UserProfile {
                                name: u.clone(),
                                avatar: "https://i.pinimg.com/736x/c2/68/bd/c268bd1d2b621ec2e5646dafbf48dd6b.jpg".to_string(),
                                is_online: true,
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        self.typing_user = None;
                        return true;
                    }
                    MsgTypes::Typing => {
                        let typing_from = msg.data.unwrap_or_default();
                        if typing_from != username {
                            self.typing_user = Some(typing_from);
                            return true;
                        }
                        return false;
                    }
                    _ => return false,
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    self.typing_user = None;
                    let _ = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap());
                    input.set_value("");
                };
                false
            }
            Msg::Typing => {
                let message = WebSocketMessage {
                    message_type: MsgTypes::Typing,
                    data: Some(username),
                    data_array: None,
                };
                let _ = self
                    .wss
                    .tx
                    .clone()
                    .try_send(serde_json::to_string(&message).unwrap());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let oninput = ctx.link().callback(|_| Msg::Typing);

        html! {
            <div class="flex w-screen">
                <div class="flex-none w-56 h-screen bg-violet-100">
                    <div class="text-xl p-3">{"🫂 Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white rounded-lg p-2">
                                    <div>
                                        <div class="relative">
                                            <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                            {
                                                if u.is_online {
                                                    html! {
                                                        <span class="absolute bottom-0 right-0 block h-3 w-3 rounded-full ring-2 ring-white bg-green-500"></span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {"Hiiii!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300">
                        <div class="text-xl p-3">{"💬 Chat!"}</div>
                    </div>
                    <div class="w-full grow overflow-auto border-b-2 border-gray-300">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                let timestamp_html = if let Some(timestamp) = m.time {
                                    if let Some(dt) = chrono::NaiveDateTime::from_timestamp_opt(timestamp / 1000, 0) {
                                        html! {
                                            <div class="text-[10px] text-gray-400 mt-1">
                                                {format!("Sent at {}", dt.format("%Y-%m-%d %H:%M:%S"))}
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                };       
                                let bubble_class = color_for_user(&m.from);

                                html!{
                                    <div class={classes!("flex", "items-end", "w-3/6", bubble_class, "m-8", "rounded-tl-lg", "rounded-tr-lg", "rounded-br-lg")}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3">
                                            <div class="text-sm">
                                                {m.from.clone()}
                                            </div>
                                            <div class="text-xs text-gray-500">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                                {timestamp_html}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                        {
                            if let Some(name) = &self.typing_user {
                                html! {
                                    <div class="text-xs text-gray-500 px-4 pb-4">
                                        {format!("{} is typing...", name)}
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                    <div class="w-full h-14 flex px-3 items-center">
                        <input
                            ref={self.chat_input.clone()}
                            oninput={oninput}
                            type="text"
                            placeholder="Message"
                            class="block w-full py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:text-gray-700"
                            name="message"
                            required=true
                        />
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center color-white">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}

