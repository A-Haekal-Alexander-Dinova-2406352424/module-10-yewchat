const WebSocket = require("ws");

const port = 8080;
const server = new WebSocket.Server({ host: "127.0.0.1", port });

function normalizeMessage(raw) {
  try {
    const parsed = JSON.parse(raw.toString());
    return JSON.stringify({
      sender: parsed.sender || "Anonymous",
      text: parsed.text || "",
    });
  } catch (_error) {
    return JSON.stringify({
      sender: "Server",
      text: raw.toString(),
    });
  }
}

server.on("connection", (socket, request) => {
  const address = `${request.socket.remoteAddress}:${request.socket.remotePort}`;
  console.log(`New YewChat client from ${address}`);

  socket.on("message", (raw) => {
    const payload = normalizeMessage(raw);
    console.log(`Broadcasting ${payload}`);

    for (const client of server.clients) {
      if (client.readyState === WebSocket.OPEN) {
        client.send(payload);
      }
    }
  });
});

server.on("listening", () => {
  console.log(`Haekal Alexander Dinova JavaScript websocket server listening on ws://127.0.0.1:${port}`);
});
