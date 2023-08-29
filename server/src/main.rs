use axum::{ extract::{ ws::{ Message, WebSocket, WebSocketUpgrade }, State, }, response::IntoResponse, routing::get, Router, };
use futures::{ sink::SinkExt, stream::StreamExt, };
use std::{ net::TcpListener, sync::{ Arc, Mutex, MutexGuard }, str::Chars, time::Duration, thread::sleep, };
use tokio::{ sync::broadcast, task::JoinHandle };
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt, };
use serde::Serialize;
use serde_json;
use anyhow;
use uuid::Uuid;

struct Platformer {
    players: Mutex<Vec<Player>>,
    tx: broadcast::Sender<String>,
}

#[derive(Serialize, Debug)]
struct Player {
    //static, for now
    #[serde(skip_serializing)]
    id: Uuid,
    rgb: String,
    width: f64,
    height: f64,
    //dynamic
    #[serde(skip_serializing)]
    x_velocity: f64,
    #[serde(skip_serializing)]
    y_velocity: f64,
    x_min: f64,
    y_min: f64,
    #[serde(skip_serializing)]
    x_max: f64,
    #[serde(skip_serializing)]
    y_max: f64,
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
            width: 10.0,
            height: 10.0,
            x_velocity: 0.0,
            y_velocity: 0.0,
            x_min: 50.0,
            y_min: 50.0,
            x_max: 60.0,
            y_max: 60.0,
        });

        Some(id)

    }

    pub fn frame(&self) -> anyhow::Result<usize> {

        let players: &mut Vec<Player> = &mut *self.players.lock().unwrap();

        for player in players.iter_mut() {

            player.y_velocity += GRAVITY;

            player.x_min += player.x_velocity;
            //player.x_max += player.x_velocity;
            player.y_min += player.y_velocity;
            //player.y_max += player.y_velocity;

        }

        // check for collisions 
        
        let json: String = serde_json::to_string(players).unwrap();

        Ok(self.tx.send(json)?)

    }

    pub fn stdin(&self, id: Uuid, input: &str) {

        let mut chars: Chars<'_> = input.chars();

        let players: &mut MutexGuard<'_, Vec<Player>> = &mut self.players.lock().unwrap();

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
                player.y_velocity += -1.5;
            },
            PlayerEvent::StartLeft => {
                player.x_velocity -= 1.0;
            },
            PlayerEvent::StartRight => {
                player.x_velocity += 1.0;
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

static GRAVITY: f64 = 0.0;
static FRAME_MS: u64 = 10;

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