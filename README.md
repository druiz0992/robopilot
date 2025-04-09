# robopilot
Demo project with a frontend to control a mobile car using two joysticks (one for the left motor and the other for the right motor).
The backend receives the control information and generates commands for the motors, connected via serial port.

To launch the example, start the the backend in one terminal.
```bash
cd backend
RUST_LOG=info cargo run --example arduino
```

If websocker server fails, ensure that the configured IP address is correct:
```bash
ip a | grep 192.168.1
```

Set `ws_url` to that ip address.

Then start the frontendd in another terminal. You can connect at http://localhost:3000. The idea is tu use a mobile phone as the controller.

```bash
cd frontend
npm i
npm run dev
```
