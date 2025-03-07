import { Container } from '@/components/Container'

const sensors = [
  {
    name: 'Orientation',
    icon: DeviceCompassIcon,
    disabled: true,
  },
  {
    name: 'Odometry',
    icon: DeviceTiresIcon,
    disabled: true,
  },
  {
    name: 'Distance',
    icon: DeviceDistanceIcon,
    disabled: true,
  },
  {
    name: 'Manual',
    icon: DeviceLockIcon,
    disabled: true,
  },
]

function DeviceDistanceIcon(props: React.ComponentPropsWithoutRef<'svg'>) {
  return(
  <svg viewBox="0 0 32 32" fill="none" aria-hidden="true" {...props}>
    {/* Background Circle */}
    <circle cx="16" cy="16" r="15" stroke="#737373" strokeWidth="2" fillOpacity="0.2" />

    {/* Ruler Shape */}
    <rect x="6" y="12" width="20" height="8" fill="#737373" rx="1" />

    {/* Distance Markings */}
    <path d="M8 12V9" stroke="#171717" strokeWidth="2" />
    <path d="M12 12V10" stroke="#171717" strokeWidth="2" />
    <path d="M16 12V8" stroke="#171717" strokeWidth="2" />
    <path d="M20 12V10" stroke="#171717" strokeWidth="2" />
    <path d="M24 12V9" stroke="#171717" strokeWidth="2" />

    {/* Measurement Indicator (Arrow) */}
    <path d="M6 22L16 26L26 22" stroke="red" strokeWidth="2" strokeLinecap="round" />

    {/* Center Indicator */}
    <circle cx="16" cy="16" r="2" fill="#171717" />
  </svg>
)
}


function DeviceCompassIcon(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
  <svg viewBox="0 0 32 32" fill="none" aria-hidden="true" {...props}>
    {/* Outer circle */}
    <circle cx="16" cy="16" r="15" stroke="#737373" strokeWidth="2" fillOpacity="0.2" />

    {/* Compass needle */}
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M16 5L19 16L16 27L13 16Z"
      fill="red"
    />
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M16 5L13 16L16 27L19 16Z"
      fill="blue"
    />

    {/* Center pivot */}
    <circle cx="16" cy="16" r="2" fill="#171717" />

    {/* Direction Markers */}
    <text x="16" y="4" fontSize="3" textAnchor="middle" fill="#737373">N</text>
    <text x="16" y="30" fontSize="3" textAnchor="middle" fill="#737373">S</text>
    <text x="2" y="17" fontSize="3" textAnchor="middle" fill="#737373">W</text>
    <text x="30" y="17" fontSize="3" textAnchor="middle" fill="#737373">E</text>
  </svg>
  )
}


function DeviceLockIcon(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
    <svg viewBox="0 0 32 32" aria-hidden="true" {...props}>
      <circle cx={16} cy={16} r={16} fill="#A3A3A3" fillOpacity={0.2} />
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M5 4a4 4 0 014-4h14a4 4 0 014 4v10h-2V4a2 2 0 00-2-2h-1.382a1 1 0 00-.894.553l-.448.894a1 1 0 01-.894.553h-6.764a1 1 0 01-.894-.553l-.448-.894A1 1 0 0010.382 2H9a2 2 0 00-2 2v24a2 2 0 002 2h5v2H9a4 4 0 01-4-4V4z"
        fill="#737373"
      />
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M18 19.5a3.5 3.5 0 117 0V22a2 2 0 012 2v6a2 2 0 01-2 2h-7a2 2 0 01-2-2v-6a2 2 0 012-2v-2.5zm2 2.5h3v-2.5a1.5 1.5 0 00-3 0V22z"
        fill="#171717"
      />
    </svg>
  )
}

function DeviceTiresIcon(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
    <svg
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      {...props}
    >
      {/* Outer tire 1 */}
      <circle cx="20" cy="32" r="14" stroke="#171717" strokeWidth="3" fill="#444" />
      <circle cx="20" cy="32" r="8" stroke="#171717" strokeWidth="2" fill="#222" />
      <path d="M20 18v8M20 38v8M10 32h8M22 32h8" stroke="#fff" strokeWidth="2" />

      {/* Outer tire 2 (slightly offset) */}
      <circle cx="44" cy="32" r="14" stroke="#171717" strokeWidth="3" fill="#444" />
      <circle cx="44" cy="32" r="8" stroke="#171717" strokeWidth="2" fill="#222" />
      <path d="M44 18v8M44 38v8M34 32h8M46 32h8" stroke="#fff" strokeWidth="2" />
    </svg>
  );
}

export function Sensors() {
  return (
    <section
      id="sensors"
      aria-label="Robot sensors"
      className="absolute top-0 left-0 w-full bg-white shadow-md py-1"
    >
      <Container>
        <ul
          role="list"
          className="mx-auto flex flex-wrap justify-between items-center px-6 w-full gap-4"

        >
          {sensors.map((feature) => (
            <li
              key={feature.name}
              className="flex flex-col items-center justify-between rounded-2xl border border-gray-20 p-4 flex-1 min-w-0 h-24"
            >
              <feature.icon className="h-12 w-12" />
              <h3 className="mt-1 font-semibold text-gray-900 text-center text-xs sm:text-sm md:text-base lg:text-lg">
                {feature.name}
              </h3>
            </li>
          ))}
        </ul>
      </Container>
    </section>
  )
}
