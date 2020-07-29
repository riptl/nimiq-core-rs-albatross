use futures::future;
use futures::prelude::*;
use ws_stream_wasm::WsMeta;
use url::Url;

use crate::websocket::error::Error;
use crate::websocket::NimiqMessageStream;

/// Connect to a given URL and return a Future that will resolve to a NimiqMessageStream
pub async fn nimiq_connect_async(
    url: Url,
) -> Box<dyn Future<Item = NimiqMessageStream, Error = Error> + Send> {
    Box::new(match WsMeta::connect(url, None).await {
        Ok((_, ws_stream)) => future::result(NimiqMessageStream::new(ws_stream, true)),
        Err(e) => future::err(e.into()),
    })
}
