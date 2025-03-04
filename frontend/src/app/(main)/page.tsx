import JoystickContainer from '@/components/JoystickContainer'
import { Sensors } from '@/components/Sensors'

export default function Home() {
  return (
    <>  
    <div className="min-h-screen flex flex-col overflow-hidden">

      <Sensors />
      <div className="flex-grow" style={{ flexGrow: 0.5 }}></div>
      <JoystickContainer />
      </div>
    </>
  )
}
