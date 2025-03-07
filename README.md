# robopilot
Demo project with a frontend to control a mobile car using two joysticks (one for the left motor and the other for the right motor).
The backend receives the contol information and generates commands for the motors, connected via serial port.

To launch the example, start the the backend in one terminal.
```bash
cd backend
cargo run --example arduino
```

Then start the frontendd in another terminal. You can connect at http://localhost:3000. The idea is tu use a mobile phone as the controller.

```bash
cd frontend
npm i
npm run dev
```
