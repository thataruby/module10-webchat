# Advanced Programming Module 10 - Webchat
### Athazahra Nabila Ruby - 2306173113

> Original code
 
![Login](images/login.png)
![Chat](images/chat.png)

> Be creative!
 
![Login](images/login_after.png)
In the login page, I added a welcome message and a footer by modifying the html on login.rs
![Chat](images/chat_after.png)
In the chat page, I added:
- Typing Indicator
 
I implemented a "typing..." indicator that appears when another user is typing. This is done by sending a typing WebSocket message whenever the input field changes (oninput). The message includes the username of the one typing. When the message is received, the UI displays <username> is typing... at the bottom of the message area if it's not the current user.

- Online Indicator
 
I added a small green circle below each user's avatar in the user list to show they’re online. This is done by rendering a green dot using Tailwind's bg-green-400 and rounded-full classes. All users in the WebSocket connection are considered online, and the list is updated every 5 seconds via the server's users message.

- Timestamps
 
I added a timestamp on each message bubble to show when that message was sent. This is done by including a time field in each message from the server, which contains the exact moment the message was created, then I used the chrono crate to convert that number into a readable date and time format like “2025-05-23 12:29:10”. 

- Different Color Bubbles
 
I added different background colors to the message bubbles so it’s easier to tell who the sender is. This is done by taking each sender’s username and hashing it into a number, then using that number to pick a background color from a list of Tailwind colors.

- Others
 
For fun, I set everyone's profile picture to my favorite actress, Jodie Comer!

> Bonus: Rust Websocket server for YewChat!
 
To complete the bonus, I added a Rust-based server adapted from Tutorial 2 (named tutorial_2 server here) and used that instead of the original JavaScript WebSocket server. The Rust server was modified to handle structured JSON messages, matching the format used by the YewChat frontend (messageType, data, dataArray). I reused the existing server.rs logic and added serde for parsing and generating JSON, and I tracked connected users in a shared map using Arc<Mutex<...>>. Messages were serialized and broadcasted to all clients using tokio-websockets.

This was a successful change because the frontend was able to register users, show real-time updates, typing notifications, and broadcast messages—all without any changes to the frontend logic other than switching the WebSocket URL to port 2000. The server correctly mimics the behavior of the original JS server while leveraging Rust’s strong type safety and concurrency model.

Personally, I prefer the Rust version because I feel like it is easier to mantain and is less error prone for real- time concurrent applications. It also integrates smoothly with the async ecosystem provided by tokio, which made it easy to handle multiple clients and broadcasts efficiently.