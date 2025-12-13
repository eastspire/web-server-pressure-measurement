const { Worker, isMainThread } = require('worker_threads');
const http = require('http');
const THREAD_COUNT = 128;
const URL = 'http://127.0.0.1:60000/';

function sendRequest() {
  return new Promise((resolve) => {
    const req = http.get(URL, (res) => {
      res.on('data', () => {});
      res.on('end', resolve);
    });
    req.on('error', resolve);
    req.end();
  });
}

function workerThread() {
  (async () => {
    while (true) {
      await sendRequest();
    }
  })();
}

if (isMainThread) {
  for (let i = 0; i < THREAD_COUNT; i++) {
    new Worker(__filename);
  }
} else {
  workerThread();
}
