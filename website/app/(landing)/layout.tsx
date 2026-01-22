import '../../styles/globals.css'
import Navbar from './components/Navbar'
import Footer from './components/Footer'

export default function LandingLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <>
      <Navbar />
      {children}
      <Footer />
    </>
  )
}
