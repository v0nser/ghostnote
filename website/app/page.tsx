import { Footer } from "@/components/footer";
import { Header } from "@/components/header";
import { LiveNotifications } from "@/components/live-notifications";
import { Capabilities } from "@/components/sections/capabilities";
import { Contribute } from "@/components/sections/contribute";
import { Cta } from "@/components/sections/cta";
import { Demo } from "@/components/sections/demo";
import { Faq } from "@/components/sections/faq";
import { Features } from "@/components/sections/features";
import { Hero } from "@/components/sections/hero";
import { HowItWorks } from "@/components/sections/how-it-works";
import { OpenSource } from "@/components/sections/open-source";
import { Testimonials } from "@/components/sections/testimonials";

export default function HomePage() {
  return (
    <div id="top">
      <Header />
      <main>
        <Hero />
        <Demo />
        <Capabilities />
        <Features />
        <HowItWorks />
        <OpenSource />
        <Contribute />
        <Testimonials />
        <Faq />
        <Cta />
      </main>
      <Footer />
      <LiveNotifications />
    </div>
  );
}
