use std::{collections::LinkedList, env, fs::File, io::Read, path::Path, process::exit, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use tailcall::tailcall;
use once_cell::sync::Lazy;
use tokio::{spawn, sync::RwLock};
use poise::serenity_prelude as serenity;

static TO_BE_QUEUED: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));
static VIEWABLE_QUEUE: Lazy<Arc<RwLock<LinkedList<String>>>> = Lazy::new(|| Arc::new(RwLock::new(LinkedList::new())));
static SOFT_BLOCK: AtomicBool = AtomicBool::new(false);

mod commands;

// Types used by all command functions
struct Data {} // Stores user data
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tailcall]
async fn run_queue(queue: &mut LinkedList<String>, stock: i32) {
    let mut tbq = TO_BE_QUEUED.write().await;
    while !tbq.is_empty() {
        let id = tbq.pop().unwrap();
        if queue.iter().any(|member| member.to_string() == id.to_string()) {
            // pass
        } else {
            queue.push_back(id);
        }
    }
    *VIEWABLE_QUEUE.write().await = queue.clone();
    if stock == 0 {
        println!("Stock Empty");
        std::thread::sleep(std::time::Duration::from_secs(5));
        run_queue(queue, stock);
    } else if queue.is_empty() {
        println!("Queue Empty");
        std::thread::sleep(std::time::Duration::from_secs(5));
        run_queue(queue, stock);
    } else {
        if !SOFT_BLOCK.load(Ordering::SeqCst) {
            let dequed_member = queue.pop_front().unwrap();
        
            println!("Dequed: {}", dequed_member);
            run_queue(queue, stock - 1)
        } else {
            run_queue(queue, stock)
        }
    }
}

#[tokio::main]
async fn main() {
    let mut token = String::new();
    let path_str = env::current_dir().expect("FS Error");
    let p = Path::new(&format!("{}/.config", path_str.display())).to_owned();
    if Path::exists(&p) {
        
        if let Ok(mut config_file) = File::open(p) {
            if let Err(_) = config_file.read_to_string(&mut token) {
                panic!("FS Error");
            }
        } else {
            panic!("FS Error");
        }
    } else {
        panic!("The config file does not exist.");
    }
    let intents = serenity::GatewayIntents::non_privileged();
    let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
        commands: vec![commands::request(), commands::get_queue()],
        ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            ctx.set_activity(Some(serenity::ActivityData::playing("with memory safe code".to_string())));
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {})
        })
    })
    .build();
    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;
    spawn(async move {
        run_queue(&mut LinkedList::new(), 15).await;
    });
    spawn(async move {
        let res = client.unwrap().start().await;
        
        if let Err(err_code) = res {
            println!("err: {}", err_code);
        }
    });
    loop{}
}