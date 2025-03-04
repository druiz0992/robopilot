import { Footer } from '@/components/Footer'
import { Header } from '@/components/Header'

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <>
      <main className="flex-auto">{children}</main>
    </>
  )
}
