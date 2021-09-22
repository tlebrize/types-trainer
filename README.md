# types-trainer

Online multiplayer game about knowing type matchups in pokemon.
Made in rust with Tokyo-Tungstenite and Raylib.

# Usage

## Server
If you want to play over the network you will need something like ngrok.

`cargo run --bin server $host:$port`

## Clients
You need two clients connected to the same server before the game starts.

`cargo run --bin client ws://$host:$port`
