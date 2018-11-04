use client::conn::ClientConnData;
use client::conn::ClientStream;
use client::conn::ClientStreamData;
use client::conn::ClientToWriteMessage;
use common::client_or_server::ClientOrServer;
use common::types::Types;
use req_resp::RequestOrResponse;

pub struct ClientTypes;

impl Types for ClientTypes {
    type HttpStreamData = ClientStream;
    type HttpStreamSpecific = ClientStreamData;
    type ConnSpecific = ClientConnData;
    type ToWriteMessage = ClientToWriteMessage;

    const CLIENT_OR_SERVER: ClientOrServer = ClientOrServer::Client;

    const OUT_REQUEST_OR_RESPONSE: RequestOrResponse = RequestOrResponse::Request;
}