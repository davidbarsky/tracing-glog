# tracing-glog

`tracing-glog` is a [glog](https://github.com/google/glog)-inspired formatter for [`tracing-subscriber`](https://docs.rs/tracing-subscriber).

`tracing-glog` should be used with `tracing-subscriber`, as it is a formatter
that `tracing-subscriber`'s [`fmt::Subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Subscriber.html) and [`fmt::Layer`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Layer.html) can use to
format events in a glog-inspired fashion. Here's an example:

<pre><font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.724801 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>34</b>] preparing to shave yaks, <b>number_of_yaks</b>: 3
<font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.724948 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>75</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}] shaving yaks
<font color="#A2734C">W</font><font color="#8D8F8A">1201 01:13:04.725071 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>56</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}, <b>shave</b>{<i><b>yak</b></i>: 3}] could not locate yak
<font color="#C01C28">E</font><font color="#8D8F8A">1201 01:13:04.725135 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>85</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}] failed to shave yak, <b>yak</b>: 3, <b>error</b>: out of cash
<font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.725195 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>38</b>] yak shaving completed, <b>all_yaks_shaved</b>: false
</pre>

## Examples

With [`fmt::Subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Subscriber.html):

```
use tracing_glog::{Glog, GlogFields};

tracing_subscriber::fmt()
    .event_format(Glog::default())
    .fmt_fields(GlogFields::default())
    .init();
```

With [`tracing_subscriber::fmt::Layer`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Layer.html):

```
use tracing_glog::{Glog, GlogFields};
use tracing_subscriber::{fmt, Registry};
use tracing_subscriber::prelude::*;

let fmt = fmt::Layer::default()
    .event_format(Glog::default())
    .fmt_fields(GlogFields::default());

let subscriber = Registry::default().with(fmt);
tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
```