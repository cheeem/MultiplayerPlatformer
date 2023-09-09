use axum::{ extract::{ ws::{ Message, WebSocket, WebSocketUpgrade }, State, }, response::IntoResponse, routing::get, Router, };
use futures::{ sink::SinkExt, stream::StreamExt, };
use std::{ net::TcpListener, sync::{ Arc, Mutex, MutexGuard }, str::Chars, time::Duration, thread::sleep, };
use tokio::{ sync::broadcast, task::JoinHandle };
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt, };
use serde::Serialize;
use serde_json;
use uuid::Uuid;

struct Platformer {
    players: Mutex<Vec<Player>>,
    tx: broadcast::Sender<String>,
}

#[derive(Serialize, Debug)]
struct Player {
    // static, for now
    #[serde(skip_serializing)]
    id: Uuid,
    rgb: String,
    w: f64,
    h: f64,
    // dynamic
    #[serde(skip_serializing)]
    x_velocity: f64,
    #[serde(skip_serializing)]
    y_velocity: f64,
    x: f64,
    y: f64,
}

enum Platform {
    Base(DimensionPosition),
    Jumpthrough(DimensionPosition),
}

#[derive(Serialize, Debug)]
struct DimensionPosition {
    w: f64,
    h: f64,
    x: f64,
    y: f64,
}

enum PlayerEvent {
    Jump,
    StartLeft,
    StartRight,
    StopLeft,
    StopRight,
}

impl Platformer {

    pub fn join(&self) -> Option<Uuid> {

        let mut players: MutexGuard<'_, Vec<Player>> = self.players.lock().unwrap();
        
        let id: Uuid = Uuid::new_v4();

        players.push(Player { 
            id,
            rgb: "red".to_owned(),
            w: 10.0,
            h: 10.0,
            x: 20.0,
            y: 20.0,
            x_velocity: 0.0,
            y_velocity: 0.0,
        });

        Some(id)

    }

    pub fn frame(&self) {

        let players: &mut [Player] = &mut *self.players.lock().unwrap();

        if players.len() == 0 {
            return;
        }

        for player in players.iter_mut() {

            player.y_velocity += GRAVITY;

            player.x += player.x_velocity;
            player.y += player.y_velocity;

        }

        for player in players.iter_mut() {

            for platform in PLATFORMS {

                match platform {
                    Platform::Base(platform) => {

                        let is_colliding: bool =
                            player.x < platform.x + platform.w &&
                            player.x + player.w > platform.x &&
                            player.y < platform.y + platform.h &&
                            player.y + player.h > platform.y;

                        if is_colliding {
                            player.y_velocity -= GRAVITY;
                            player.rgb = "green".to_owned();
                        }

                    }
                    Platform::Jumpthrough(_platform) => {

                    }
                }

            }

        }
        
        let json: String = serde_json::to_string(players).unwrap();

        let _ = self.tx.send(json);

    }

    pub fn stdin(&self, id: Uuid, input: &str) {

        let mut chars: Chars<'_> = input.chars();

        let players: &mut [Player] = &mut *self.players.lock().unwrap();

        for player in players.iter_mut() {
            if player.id == id {
                PlayerEvent::from_char(chars.next().unwrap()).unwrap().execute(player);
                break;
            }
        }

    }

}

impl PlayerEvent {

    pub fn execute(&self, player: &mut Player) {
        match self {
            PlayerEvent::Jump => {
                player.y_velocity -= 0.05;
            },
            PlayerEvent::StartLeft => {
                player.x_velocity = 0.05;
            },
            PlayerEvent::StartRight => {
                player.x_velocity = 0.05;
            },
            PlayerEvent::StopLeft => {

            },
            PlayerEvent::StopRight => {

            }
        };
    }

    pub fn from_char(char: char) -> Option<Self> {
        match char {
            'j' => Some(PlayerEvent::Jump),
            'l' => Some(PlayerEvent::StartLeft),
            'r' => Some(PlayerEvent::StartRight),
            'a' => Some(PlayerEvent::StopLeft),
            'd' => Some(PlayerEvent::StopRight),
            _ => return None,
        }
    }

}

const GRAVITY: f64 = 0.004;
const FRAME_MS: u64 = 1;
const PLATFORMS: [Platform; 1] = [
    Platform::Base(DimensionPosition {
        w: 400.0,
        h: 15.0,
        x: 0.0,
        y: 70.0,
    }),
];

#[tokio::main]
async fn main() {
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "server=trace".into()),)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let players: Mutex<Vec<Player>> = Mutex::new(Vec::new());

    let (tx, _rx) = broadcast::channel(100);

    let platformer: Arc<Platformer> = Arc::new(Platformer { players, tx });

    let renderer: Arc<Platformer> = platformer.clone();

    let frame_delay: Duration = Duration::from_millis(FRAME_MS);

    let app: Router = Router::new()
        .route("/ws/", get(websocket_handler))
        .with_state(platformer);

    //let listener: TcpListener = TcpListener::bind("23.152.226.72:3000").unwrap();
    let listener: TcpListener = TcpListener::bind("localhost:3000").unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    let handle: JoinHandle<()> = tokio::spawn(async move {
        //throwing away error for now
        loop {
            sleep(frame_delay);
            let _ = renderer.frame();
        }
    });

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();

    let _ = handle.await;

}

async fn websocket_handler(ws: WebSocketUpgrade, State(platformer): State<Arc<Platformer>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, platformer))
}

async fn websocket(stream: WebSocket, platformer: Arc<Platformer>) {

    let (mut sender, mut receiver) = stream.split();

    let id: Option<Uuid> = platformer.join();

    if id.is_none() {
        return;
    }

    let id: Uuid = id.unwrap();

    let mut rx: tokio::sync::broadcast::Receiver<String> = platformer.tx.subscribe();

    tracing::debug!("Player {} Joined", id);

    let mut send_task: JoinHandle<()> = tokio::spawn(async move {
        while let Ok(json) = rx.recv().await { // maybe receive players and seralize on each?
            println!("received: {}", json.len());
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let reader: Arc<Platformer> = platformer.clone();

    let mut recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(Ok(Message::Text(input))) = receiver.next().await {
            reader.stdin(id, &input);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    tracing::debug!("Player {} Left.", id);

    let players: &mut MutexGuard<'_, Vec<Player>> = &mut platformer.players.lock().unwrap();

    let idx: usize = players
        .iter()
        .position(|player| player.id == id)
        .unwrap();

    players.remove(idx);

}