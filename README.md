# tracing-glog

`tracing-glog` is a [glog](https://github.com/google/glog)-inspired formatter for [`tracing-subscriber`](https://docs.rs/tracing-subscriber).

`tracing-glog` should be used with `tracing-subscriber`, as it is a formatter
that `tracing-subscriber`'s [`fmt::Subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Subscriber.html) and [`fmt::Layer`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Layer.html) can use to
format events in a glog-inspired fashion.

## Examples

With [`fmt::Subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Subscriber.html):

```rust
use tracing_glog::{Glog, GlogFields};

tracing_subscriber::fmt()
    .event_format(Glog::default())
    .fmt_fields(GlogFields::default())
    .init();
```

With [`tracing_subscriber::fmt::Layer`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Layer.html):

```rust
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, Registry};
use tracing_glog::{Glog, GlogFields};

let fmt = fmt::Layer::default()
    .event_format(Glog::default())
    .fmt_fields(GlogFields::default());

let subscriber = Registry::default().with(fmt);
tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
```