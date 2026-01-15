"use client";

import { useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Button } from "../ui/button";

export function CTA() {
  const containerRef = useRef(null);
  const isInView = useInView(containerRef, { once: true, margin: "-100px" });

  return (
    <section
      ref={containerRef}
      className="relative py-32 bg-[var(--neutral-900)]"
    >
      <div className="max-w-3xl mx-auto px-6 text-center">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
          transition={{ duration: 0.8 }}
        >
          {/* Main headline */}
          <h2 className="text-4xl md:text-5xl lg:text-6xl mb-6">
            <span className="font-serif italic text-[var(--accent)]">Ready to never</span>
            <br />
            <span className="font-semibold text-white">forget again?</span>
          </h2>

          <p className="text-lg text-[var(--neutral-400)] mb-10 max-w-xl mx-auto">
            Pre-order now and be among the first to receive DaysTracker.
          </p>

          {/* Pricing */}
          <motion.div
            className="inline-flex items-center gap-4 mb-10"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={isInView ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.95 }}
            transition={{ duration: 0.6, delay: 0.2 }}
          >
            <span className="text-[var(--neutral-500)] line-through text-2xl">$149</span>
            <span className="text-white text-5xl font-bold">$99</span>
            <span className="text-[var(--accent)] text-sm font-medium px-3 py-1 bg-[var(--accent)]/10 rounded-full">
              Save $50
            </span>
          </motion.div>

          {/* CTA buttons */}
          <motion.div
            className="flex flex-col sm:flex-row gap-4 justify-center items-center"
            initial={{ opacity: 0, y: 15 }}
            animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 15 }}
            transition={{ duration: 0.6, delay: 0.4 }}
          >
            <Button size="lg">
              Pre-Order Now
            </Button>
            <Button
              variant="outline"
              size="lg"
              className="border-[var(--neutral-700)] text-white hover:bg-[var(--neutral-800)]"
            >
              Learn More
            </Button>
          </motion.div>

          {/* Trust badges */}
          <motion.div
            className="mt-12 flex flex-wrap justify-center gap-8"
            initial={{ opacity: 0 }}
            animate={isInView ? { opacity: 1 } : { opacity: 0 }}
            transition={{ duration: 0.6, delay: 0.6 }}
          >
            <div className="flex items-center gap-2 text-[var(--neutral-500)]">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
              <span className="text-sm">30-Day Money Back</span>
            </div>
            <div className="flex items-center gap-2 text-[var(--neutral-500)]">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span className="text-sm">Ships Q2 2026</span>
            </div>
            <div className="flex items-center gap-2 text-[var(--neutral-500)]">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
              </svg>
              <span className="text-sm">Secure Checkout</span>
            </div>
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}
