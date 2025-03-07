import JoystickContainer from "@/components/JoystickContainer";
import { Sensors } from "@/components/Sensors";

export default function Home() {
  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden">
      <div className="h-1/4">
        <Sensors />
      </div>

      {/* JoystickContainer gets a fixed height + bottom padding to avoid cutoff */}
      <div className="h-3/4 flex items-center pb-4">
        <JoystickContainer />
      </div>
    </div>
  );
}
