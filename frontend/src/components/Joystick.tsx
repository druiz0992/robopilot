"use client";

import { useEffect, useRef } from "react";
import "@/styles/joystick.css"; // Adjust path if necessary
import joystickBase from '@/images/joystick-base.png';
import joystickBlue from '@/images/joystick-blue.png';


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

  constructor(stickID: string, onUpdate: (value: { y: number }) => void) {
    this.id = stickID;
    this.stick = document.getElementById(stickID);
    this.dragStart = null;
    this.touchId = null;
    this.active = false;
    this.value = { y: 0 };
    this.onUpdate = onUpdate;

    if (!this.stick) return;

    this.stick.addEventListener("mousedown", this.handleDown.bind(this));
    this.stick.addEventListener("touchstart", this.handleDown.bind(this));
    document.addEventListener("mousemove", this.handleMove.bind(this), { passive: false });
    document.addEventListener("touchmove", this.handleMove.bind(this), { passive: false });
    document.addEventListener("mouseup", this.handleUp.bind(this));
    document.addEventListener("touchend", this.handleUp.bind(this));
  }

  handleDown(event: MouseEvent | TouchEvent) {
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

  handleMove(event: MouseEvent | TouchEvent) {
    if (!this.active || !this.stick || !this.dragStart) return;

    let clientY: number;
    if ("changedTouches" in event) {
      let touch = Array.from(event.changedTouches).find(t => t.identifier === this.touchId);
      if (!touch) return;
      clientY = touch.clientY;
    } else {
      clientY = event.clientY;
    }

    const yDiff = clientY - this.dragStart.y;
    const distance = Math.min(MAX_DISTANCE, Math.abs(yDiff));
    const yPosition = yDiff < 0 ? -distance : distance;

    this.stick.style.transform = `translate3d(0px, ${yPosition}px, 0px)`;

    const distance2 = distance < DEADZONE ? 0 : MAX_DISTANCE / (MAX_DISTANCE - DEADZONE) * (distance - DEADZONE);
    const yPercent = parseFloat((distance2 / MAX_DISTANCE * (yDiff < 0 ? -1 : 1)).toFixed(4));

    this.value = { y: yPercent };
    this.onUpdate(this.value);
  }

  handleUp(event: MouseEvent | TouchEvent) {
    if (!this.active || !this.stick) return;
    if ("changedTouches" in event && this.touchId !== event.changedTouches[0].identifier) return;

    this.stick.style.transition = ".2s";
    this.stick.style.transform = "translate3d(0px, 0px, 0px)";
    this.value = { y: 0 };
    this.touchId = null;
    this.active = false;
    this.onUpdate(this.value);
  }
}

const Joystick = ({ id, onMove }: JoystickProps) => {
  const joystickRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (joystickRef.current) new JoystickController(id, onMove);
  }, [id, onMove]);

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