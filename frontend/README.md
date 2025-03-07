# Frontend
The frontend is a touch based contoller with two joysticks (taken from https://github.com/stemkoski/HTML-Joysticks).

The frontend connects to a websocket server at 192.168.1.69:8080, and sends updated control information. The contoller data
has this format: `{"Data":["joystick","0, -0.2679"]}`, where `joystick` is the channel name, and the second part is
the joystick position. The first number is the left contoller, and the second number is the right contoller.

To launch the frontend:
```bash
npm i
npm run dev
```
