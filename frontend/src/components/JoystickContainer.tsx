"use client";

import { useEffect, useState, useRef } from "react";
import "@/styles/joystick.css"; // Adjust path if necessary
import Joystick from "./Joystick";
import { WS_URL } from "@/config";


const socket = new WebSocket(WS_URL);

const JoystickContainer = () => {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false); // Add state for connection status

  const prevJoystickLeftValue = useRef({ y: 0 });
  const prevJoystickRightValue = useRef({ y: 0 });

  useEffect(() => {
    const socket = new WebSocket(WS_URL);

    socket.onopen = () => {
      console.log("WebSocket Connected");
      setIsConnected(true); // Update connection status when connection opens
    };

    socket.onerror = (error) => {
      console.error("WebSocket Error:", error);
      setIsConnected(false); // Update connection status on error
    };

    socket.onclose = () => {
      console.log("WebSocket Disconnected");
      setIsConnected(false); // Update connection status when connection is closed
    };

    setWs(socket);

    return () => {
      socket.close();
    };
  }, []);

  const sendJoystickData = (id: string, value: { y: number }, prevValue: { y: number }, ref: React.MutableRefObject<{ y: number }>) => {
    if (value.y !== prevValue.y && ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ id, value }));
      ref.current = value; // Update the reference after sending data
    }
  };

  return (
    <div>
      <div className="joystickContainer flex w-full justify-between items-center px-8">
        {/* Joystick 1 on the left */}
        <div className="joystick-left">
          <Joystick 
            id="left" 
            onMove={(value) => { 
              sendJoystickData("left", value, prevJoystickLeftValue.current, prevJoystickLeftValue);
            }} 
          />
        </div>
  
        {/* Joystick 2 on the right */}
        <div className="joystick-right">
          <Joystick 
            id="right" 
            onMove={(value) => { 
              sendJoystickData("right", value, prevJoystickRightValue.current, prevJoystickRightValue);
            }} 
          />
        </div>
      </div>
  
      <hr />
    </div>
  );
};

export default JoystickContainer;
