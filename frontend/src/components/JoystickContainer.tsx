"use client";

import { useEffect, useState, useRef, useCallback, useMemo } from "react";
import "@/styles/joystick.css";
import Joystick from "./Joystick";
import { WS_URL } from "@/config";
import throttle from "lodash/throttle";
import debounce from "lodash/debounce";

const JoystickContainer = () => {
  const wsRef = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [hasMoved, setHasMoved] = useState(false);

  const [joystickData, setJoystickData] = useState({ left: 0, right: 0 });
  const prevJoystickData = useRef({ left: 0, right: 0 });

  // Memoized joystick state to avoid unnecessary recomputations
  const joystickState = useMemo(() => ({
    left: joystickData.left,
    right: joystickData.right,
  }), [joystickData]);

  // WebSocket connection setup
  const connectWebSocket = useCallback(() => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) return;

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

      // Auto-reconnect after 2 seconds
      setTimeout(connectWebSocket, 2000);
    };

    wsRef.current = socket;
  }, []);

  useEffect(() => {
    connectWebSocket();
    return () => {
      wsRef.current?.close();
    };
  }, [connectWebSocket]);

  // Send joystick data via WebSocket
  const sendJoystickData = useCallback((data: { left: number; right: number }) => {
    const ws = wsRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const combinedValues = `${data.left}, ${data.right}`;
    const ws_message = JSON.stringify({ Data: ["joystick", combinedValues] });

    console.log("Sending Joystick Data:", ws_message);
    ws.send(ws_message);
  }, []);

  // Throttled version to limit message frequency
  const throttledSendJoystickData = useCallback(
    throttle((data) => sendJoystickData(data), 300), // Adjust timing if needed
    [sendJoystickData]
  );

  // Debounced version to send the latest value if no movement happens for a while
  const debouncedSendJoystickData = useCallback(
    debounce((data) => sendJoystickData(data), 300),
    [sendJoystickData]
  );

  // Joystick movement handler
  const updateJoystickData = useCallback((id: "left" | "right", value: { y: number }) => {
    setJoystickData((prev) => {
      const newData = { ...prev, [id]: value.y };

      if (newData.left !== prevJoystickData.current.left || newData.right !== prevJoystickData.current.right) {
        throttledSendJoystickData(newData); // Send throttled updates
        debouncedSendJoystickData(newData); // Ensure last movement is sent
        prevJoystickData.current = newData;
        setHasMoved(true);
      }

      return newData;
    });
  }, [throttledSendJoystickData, debouncedSendJoystickData]);

  // Periodic sending of latest joystick value if no movement occurs
  useEffect(() => {
    const interval = setInterval(() => {
      if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;
      if (hasMoved) {
        sendJoystickData(prevJoystickData.current); // Send last known joystick data
      } else {
        sendJoystickData({ left: 0, right: 0 }); // Send 0,0 if no movement occurred
      }
    }, 1000); // Adjust this interval if necessary

    return () => clearInterval(interval);
  }, [hasMoved, sendJoystickData]);

  return (
    <div className="joystickContainer flex justify-between items-center w-full px-1">
      {/* Left Joystick */}
      <div className="joystick-left flex-1">
        <Joystick id="left" onMove={(value) => updateJoystickData("left", value)} />
      </div>

      {/* Right Joystick */}
      <div className="joystick-right flex-0">
        <Joystick id="right" onMove={(value) => updateJoystickData("right", value)} />
      </div>
    </div>
  );
};

export default JoystickContainer;
