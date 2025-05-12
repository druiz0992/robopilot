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

  const [joystickData, setJoystickData] = useState({ left: 0.0, right: 0.0 });
  const prevJoystickData = useRef({ left: 0.0, right: 0.0 });

  const lastMovementTime = useRef(Date.now());

  const joystickState = useMemo(
    () => ({
      left: joystickData.left,
      right: joystickData.right,
    }),
    [joystickData]
  );

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
      setTimeout(connectWebSocket, 2000); // Auto-reconnect
    };

    wsRef.current = socket;
  }, []);

  useEffect(() => {
    connectWebSocket();
    return () => {
      wsRef.current?.close();
    };
  }, [connectWebSocket]);

  const sendJoystickData = useCallback((data: { left: number; right: number }) => {
    const ws = wsRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const left = data.left.toFixed(2);
    const right = data.right.toFixed(2);
    const combinedValues = `${left}, ${right}`;
    const ws_message = JSON.stringify({ Data: ["joystick", combinedValues] });

    console.log("Sending Joystick Data:", "AAA", ws_message, combinedValues);
    ws.send(ws_message);
  }, []);

  const throttledSendJoystickData = useCallback(
    throttle((data) => sendJoystickData(data), 300),
    [sendJoystickData]
  );

  const debouncedSendJoystickData = useCallback(
    debounce((data) => sendJoystickData(data), 300),
    [sendJoystickData]
  );

  const updateJoystickData = useCallback(
    (id: "left" | "right", value: { y: number }) => {
      setJoystickData((prev) => {
        const newData = { ...prev, [id]: value.y };

        if (
          newData.left !== prevJoystickData.current.left ||
          newData.right !== prevJoystickData.current.right
        ) {
          lastMovementTime.current = Date.now();
          throttledSendJoystickData(newData);
          debouncedSendJoystickData(newData);
          prevJoystickData.current = newData;
        }

        return newData;
      });
    },
    [throttledSendJoystickData, debouncedSendJoystickData]
  );

  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      const secondsSinceLastMove = (now - lastMovementTime.current) / 1000;

      if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

      if (secondsSinceLastMove > 1.0) {
        // Only send if the last data is not already zero
        const { left, right } = prevJoystickData.current;
        //if (left !== 0 || right !== 0) {
        sendJoystickData({ left, right });
        //}
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [sendJoystickData]);

  return (
    <div className="joystickContainer flex justify-between items-center w-full px-1">
      <div className="joystick-left flex-1">
        <Joystick id="left" onMove={(value) => updateJoystickData("left", value)} />
      </div>

      <div className="joystick-right flex-0">
        <Joystick id="right" onMove={(value) => updateJoystickData("right", value)} />
      </div>
    </div>
  );
};

export default JoystickContainer;
