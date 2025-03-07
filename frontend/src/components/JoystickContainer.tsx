"use client";

import { useEffect, useState, useRef } from "react";
import "@/styles/joystick.css"; // Adjust path if necessary
import Joystick from "./Joystick";
import { WS_URL } from "@/config";

const JoystickContainer = () => {
  const wsRef = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [hasMoved, setHasMoved] = useState(false);

  const [joystickData, setJoystickData] = useState({ left: 0, right: 0 });
  const prevJoystickData = useRef({ left: 0, right: 0 });

  const connectWebSocket = () => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) return; // Prevent duplicate connections

    const socket = new WebSocket(WS_URL);

    socket.onopen = () => {
      console.log("WebSocket Connected");
      setIsConnected(true);
    };

    socket.onerror = (error) => {
      console.error("WebSocket Error:", error);
      setIsConnected(false);
    };

    socket.onclose = () => {
      console.log("WebSocket Disconnected");
      setIsConnected(false);
      wsRef.current = null;

      // Optional: Auto-reconnect after 2 seconds
      setTimeout(connectWebSocket, 2000);
    };

    wsRef.current = socket;
  };

  useEffect(() => {
    connectWebSocket();

    return () => {
      wsRef.current?.close();
    };
  }, []);

  const sendJoystickData = (data: { left: number; right: number }) => {
    const ws = wsRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const combinedValues = `${data.left}, ${data.right}`;
    const ws_message = JSON.stringify({ Data: ["joystick", combinedValues] });

    console.log("Sending Joystick Data:", ws_message);
    ws.send(ws_message);
  };

  const updateJoystickData = (id: "left" | "right", value: { y: number }) => {
    setJoystickData((prev) => {
      const newData = { ...prev, [id]: value.y };

      if (newData.left !== prevJoystickData.current.left || newData.right !== prevJoystickData.current.right) {
        sendJoystickData(newData); // Immediately send changed values
        prevJoystickData.current = newData;
        setHasMoved(true);
      }

      return newData;
    });
  };

  useEffect(() => {
    const interval = setInterval(() => {
      if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

      if (hasMoved) {
        sendJoystickData(prevJoystickData.current); // Send last known joystick data
      } else {
        sendJoystickData({ left: 0, right: 0 }); // Send 0,0 if no movement ever occurred
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [hasMoved]);

  return (
    <div>
      <div className="joystickContainer flex w-full justify-between items-center px-8">
        <div className="joystick-left">
          <Joystick id="left" onMove={(value) => updateJoystickData("left", value)} />
        </div>
        <div className="joystick-right">
          <Joystick id="right" onMove={(value) => updateJoystickData("right", value)} />
        </div>
      </div>
      <hr />
    </div>
  );
};

export default JoystickContainer;
