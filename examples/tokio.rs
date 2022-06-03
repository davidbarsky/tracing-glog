use anyhow::Error;
use tokio::task::JoinSet;
use tracing::{debug, info, instrument, span, Instrument as _, Level};
use tracing_glog::{Glog, GlogFields, GlogUtcTime};

#[instrument]
async fn parent_task(subtasks: usize) -> Result<(), Error> {
    info!("spawning subtasks...");
    let mut set = JoinSet::new();

    for number in 1..=subtasks {
        let span = span!(Level::INFO, "subtask", %number);
        debug!(message = "creating subtask;", number);
        set.spawn(subtask(number).instrument(span));
    }

    // the returnable error would be if one of the subtasks panicked.
    while let Some(task) = set.join_one().await? {
        debug!(%task, "task completed");
    }

    Ok(())
}

async fn subtask(number: usize) -> usize {
    info!(%number, "polling subtask");
    number
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .event_format(
            Glog::default()
                .with_target(false)
                .with_thread_names(false)
                .with_timer(GlogUtcTime::default()),
        )
        .fmt_fields(GlogFields::default())
        .init();
    parent_task(10).await?;
    Ok(())
}
