//! Example using Repl with a custom error type.
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

use crate::irpc::RcanClient;

/// Write "Hello" with given name
async fn hello<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!(
        "Hello, {}",
        args.get_one::<String>("who").unwrap()
    )))
}

async fn say(args: ArgMatches, context: &mut RcanClient) -> Result<Option<String>> {
    let val = args.get_one::<String>("name").unwrap();
    let item = context.info(val).await.expect("bad info");
    Ok(Some(item))
}

async fn list(_args: ArgMatches, context: &mut RcanClient) -> Result<Option<String>> {
    let item = context.list().await.expect("bad list");
    println!("{:#?}",item);
    Ok(None)
}

/// Called after successful command execution, updates prompt with returned Option
// async fn update_prompt<T>(_context: &mut T) -> Result<Option<String>> {
//     Ok(Some("updated".to_string()))
// }

pub async fn make_repl(rcl: RcanClient) -> Result<Repl<RcanClient, reedline_repl_rs::Error>> {
    let repl = Repl::new(rcl)
        .with_name("Rcanner")
        .with_version("v0.1.0")
        .with_command_async(
            Command::new("hello")
                .arg(Arg::new("who").required(true))
                .about("Greetings!"),
            |args, context| Box::pin(hello(args, context)),
        )
        .with_command_async(
            Command::new("say").arg(Arg::new("name").required(true)),
            |args, context| Box::pin(say(args, context)),
        )
        .with_command_async(Command::new("list"), |args, context| {
            Box::pin(list(args, context))
        });
    // .with_on_after_command_async(|context| Box::pin(update_prompt(context)));
    Ok(repl)
}
