var W3CWebSocket = require('websocket').w3cwebsocket;

var args = process.argv.slice(2);

const my_name = args[0]
//const port = args[1]
const addr = args[1]//"ws://localhost:" + port + "/"
console.log(addr)
var client = new W3CWebSocket(addr);

client.onerror = function() {
    console.log('Connection Error');
};

client.onopen = function() {
    console.log('WebSocket client number ' + my_name + ' connected');

    function sendNumber() {
        if (client.readyState === client.OPEN) {
            var number = Math.round(Math.random() * 0xFFFFFF);
            client.send(number.toString());
            setTimeout(sendNumber, 1000);
        }
    }
    //sendNumber();
};

client.onclose = function() {
    console.log('echo-protocol Client Closed');
};

client.onmessage = function(e: any) {
    if (typeof e.data === 'string') {
        console.log('WebSocket client number ' + my_name + " received: '" + e.data + "'");
    } else {
        console.log('WebSocket client number ' + my_name + " received non-string message")
    }
};