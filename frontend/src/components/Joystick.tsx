"use client";

import { useEffect, useRef, useCallback } from "react";
import "@/styles/joystick.css";
import joystickBase from "@/images/joystick-base.png";
import joystickBlue from "@/images/joystick-blue.png";

const MAX_DISTANCE = 64;
const DEADZONE = 8;

interface JoystickProps {
  id: string;
  onMove: (value: { y: number }) => void;
}

class JoystickController {
  id: string;
  stick: HTMLElement | null;
  dragStart: { y: number } | null;
  touchId: number | null;
  active: boolean;
  value: { y: number };
  onUpdate: (value: { y: number }) => void;

  handleDown: (event: MouseEvent | TouchEvent) => void;
  handleMove: (event: MouseEvent | TouchEvent) => void;
  handleUp: (event: MouseEvent | TouchEvent) => void;

  constructor(stickID: string, onUpdate: (value: { y: number }) => void) {
    this.id = stickID;
    this.stick = document.getElementById(stickID);
    this.dragStart = null;
    this.touchId = null;
    this.active = false;
    this.value = { y: 0 };
    this.onUpdate = onUpdate;

    this.handleDown = this._handleDown.bind(this);
    this.handleMove = this._handleMove.bind(this);
    this.handleUp = this._handleUp.bind(this);

    if (!this.stick) return;

    this.stick.addEventListener("mousedown", this.handleDown);
    this.stick.addEventListener("touchstart", this.handleDown);
    document.addEventListener("mousemove", this.handleMove, { passive: false });
    document.addEventListener("touchmove", this.handleMove, { passive: false });
    document.addEventListener("mouseup", this.handleUp);
    document.addEventListener("touchend", this.handleUp);
  }

  _handleDown(event: MouseEvent | TouchEvent) {
    this.active = true;
    if (this.stick) this.stick.style.transition = "0s";

    event.preventDefault();
    if ("changedTouches" in event) {
      this.dragStart = { y: event.changedTouches[0].clientY };
      this.touchId = event.changedTouches[0].identifier;
    } else {
      this.dragStart = { y: event.clientY };
    }
  }

  _handleMove(event: MouseEvent | TouchEvent) {
    if (!this.active || !this.stick || !this.dragStart) return;

    let clientY: number;
    if ("changedTouches" in event) {
      const touch = Array.from(event.changedTouches).find(
        (t) => t.identifier === this.touchId
      );
      if (!touch) return;
      clientY = touch.clientY;
    } else {
      clientY = event.clientY;
    }

    const yDiff = clientY - this.dragStart.y;
    const distance = Math.min(MAX_DISTANCE, Math.abs(yDiff));
    const yPosition = yDiff < 0 ? -distance : distance;

    this.stick.style.transform = `translate3d(0px, ${yPosition}px, 0px)`;

    const distance2 =
      distance < DEADZONE
        ? 0
        : (MAX_DISTANCE / (MAX_DISTANCE - DEADZONE)) * (distance - DEADZONE);
    const yPercent = parseFloat(
      ((distance2 / MAX_DISTANCE) * (yDiff < 0 ? 1 : -1)).toFixed(4)
    );

    this.value = { y: yPercent };
    this.onUpdate(this.value);
  }

  _handleUp(event: MouseEvent | TouchEvent) {
    if (!this.active || !this.stick) return;
    if (
      "changedTouches" in event &&
      this.touchId !== event.changedTouches[0].identifier
    )
      return;

    this.stick.style.transition = ".2s";
    this.stick.style.transform = "translate3d(0px, 0px, 0px)";
    this.value = { y: 0 };
    this.touchId = null;
    this.active = false;
    this.onUpdate(this.value);
  }

  cleanup() {
    if (!this.stick) return;
    this.stick.removeEventListener("mousedown", this.handleDown);
    this.stick.removeEventListener("touchstart", this.handleDown);
    document.removeEventListener("mousemove", this.handleMove);
    document.removeEventListener("touchmove", this.handleMove);
    document.removeEventListener("mouseup", this.handleUp);
    document.removeEventListener("touchend", this.handleUp);
  }
}

const Joystick = ({ id, onMove }: JoystickProps) => {
  const joystickRef = useRef<HTMLDivElement>(null);

  const memoizedOnMove = useCallback(onMove, []);

  useEffect(() => {
    const controller = new JoystickController(id, memoizedOnMove);
    return () => controller.cleanup();
  }, [id, memoizedOnMove]);

  return (
    <div className="joystick">
      <img src={joystickBase.src} alt="Joystick Base" />
      <div id={id} ref={joystickRef} className="stick">
        <img src={joystickBlue.src} alt="Joystick" />
      </div>
    </div>
  );
};

export default Joystick;
