const http = require('http');

const requestHandler = (_req, res) => {
  res.writeHead(200, {
    'Content-Type': 'text/plain',
    Connection: 'keep-alive',
  });
  res.end('Hello, World!');
};

const server = http.createServer(requestHandler);

server.keepAliveTimeout = 88888888;
server.headersTimeout = 65000;

server.listen(60000, '0.0.0.0', () => {});
