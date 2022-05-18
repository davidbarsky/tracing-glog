use structopt::StructOpt;
use thiserror::Error;
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_glog::{Glog, GlogFields};

/// To run with ANSI colors, run:
/// ```bash
/// cargo run --example yak-shave -- --with-ansi
/// ```
///
/// To run without ANSI colors, run:
/// ```bash
/// cargo run --example yak-shave
/// ```

#[derive(Debug, structopt::StructOpt)]
struct Args {
    /// Whether to run this example with or without ANSI colors.
    #[structopt(short, long)]
    with_ansi: bool,
}

fn main() {
    let args = Args::from_args();

    tracing_subscriber::fmt()
        .with_ansi(args.with_ansi)
        .event_format(Glog::new(chrono::Local))
        .fmt_fields(GlogFields::default())
        .init();

    let number_of_yaks = 3;
    // this creates a new event, outside of any spans.
    tracing::info!(number_of_yaks, "preparing to shave yaks");

    let _span = span!(Level::INFO, "yak shaving", count = number_of_yaks);
    let number_shaved = shave_all(number_of_yaks);
    tracing::info!(
        all_yaks_shaved = number_shaved == number_of_yaks,
        "yak shaving completed"
    );
}

// the `#[tracing::instrument]` attribute creates and enters a span
// every time the instrumented function is called. The span is named after the
// the function or method. Paramaters passed to the function are recorded as fields.
#[tracing::instrument]
fn shave(yak: usize) -> Result<(), OutOfSpaceError> {
    // this creates an event at the TRACE log level with two fields:
    // - `excitement`, with the key "excitement" and the value "yay!"
    // - `message`, with the key "message" and the value "hello! I'm gonna shave a yak."
    //
    // unlike other fields, `message`'s shorthand initialization is just the string itself.
    trace!(excitement = "yay!", "hello! I'm gonna shave a yak");
    if yak == 3 {
        warn!("could not locate yak");
        return Err(OutOfSpaceError::OutOfCash);
    } else {
        trace!("yak shaved successfully");
    }
    Ok(())
}

fn shave_all(yaks: usize) -> usize {
    // Constructs a new span named "shaving_yaks" at the INFO level,
    // and a field whose key is "yaks". This is equivalent to writing:
    //
    // let span = span!(Level::INFO, "shaving_yaks", yaks = yaks);
    //
    // local variables (`yaks`) can be used as field values
    // without an assignment, similar to struct initializers.
    let span = span!(Level::INFO, "shaving_yaks", yaks);
    let _enter = span.enter();

    info!("shaving yaks");

    let mut yaks_shaved = 0;
    for yak in 1..=yaks {
        let res = shave(yak);
        debug!(target: "yak_events", yak, shaved = res.is_ok());

        if let Err(ref error) = res {
            // Like spans, events can also use the field initialization shorthand.
            // In this instance, `yak` is the field being initalized.
            error!(
                yak,
                error = error as &dyn std::error::Error,
                "failed to shave yak"
            );
        } else {
            yaks_shaved += 1;
        }
        trace!(yaks_shaved);
    }

    yaks_shaved
}

// Error types
// Usually you would pick one error handling library to use, but they can be mixed freely
#[derive(Debug, Error)]
enum OutOfSpaceError {
    #[error("out of cash")]
    OutOfCash,
}
