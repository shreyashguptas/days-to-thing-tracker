"use client";

import { Navigation } from "@/components/sections/navigation";
import { Hero } from "@/components/sections/hero";
import { Features } from "@/components/sections/features";
import { HowItWorks } from "@/components/sections/how-it-works";
import { UseCases } from "@/components/sections/use-cases";
import { Colors } from "@/components/sections/colors";
import { Specs } from "@/components/sections/specs";
import { CTA } from "@/components/sections/cta";
import { Footer } from "@/components/sections/footer";

export default function Home() {
  return (
    <>
      <Navigation />
      <main>
        <Hero />
        <Features />
        <HowItWorks />
        <UseCases />
        <Colors />
        <Specs />
        <CTA />
      </main>
      <Footer />
    </>
  );
}
