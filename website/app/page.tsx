import { Footer } from "@/components/footer";
import { Header } from "@/components/header";
import { LiveNotifications } from "@/components/live-notifications";
import { Contribute } from "@/components/sections/contribute";
import { Cta } from "@/components/sections/cta";
import { Demo } from "@/components/sections/demo";
import { EarlyBird } from "@/components/sections/early-bird";
import { Faq } from "@/components/sections/faq";
import { Features } from "@/components/sections/features";
import { Hero } from "@/components/sections/hero";
import { HowItWorks } from "@/components/sections/how-it-works";
import { OpenSource } from "@/components/sections/open-source";
import { Pricing } from "@/components/sections/pricing";
import { Testimonials } from "@/components/sections/testimonials";

export default function HomePage() {
  return (
    <div id="top">
      <Header />
      <main>
        <Hero />
        <EarlyBird />
        <Demo />
        <Features />
        <HowItWorks />
        <OpenSource />
        <Pricing />
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
