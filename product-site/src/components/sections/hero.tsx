"use client";

import { useEffect, useRef, useState } from "react";
import { motion, useScroll, useTransform } from "framer-motion";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

export function Hero() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ["start start", "end start"],
  });

  const y = useTransform(scrollYProgress, [0, 1], ["0%", "50%"]);
  const opacity = useTransform(scrollYProgress, [0, 0.5], [1, 0]);
  const scale = useTransform(scrollYProgress, [0, 0.5], [1, 0.8]);

  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) return null;

  return (
    <section
      ref={containerRef}
      className="relative min-h-[200vh] overflow-hidden"
    >
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-b from-[var(--subtle-cream)] via-white to-white dark:from-[var(--dark-bg)] dark:via-[var(--dark-bg)] dark:to-[var(--dark-bg)]" />

      {/* Decorative elements */}
      <div className="absolute top-20 left-10 w-72 h-72 bg-[var(--pop-coral)]/10 rounded-full blur-3xl" />
      <div className="absolute top-40 right-10 w-96 h-96 bg-[var(--pop-orange)]/10 rounded-full blur-3xl" />
      <div className="absolute bottom-40 left-1/3 w-80 h-80 bg-[var(--pop-yellow)]/10 rounded-full blur-3xl" />

      {/* Sticky container */}
      <motion.div
        className="sticky top-0 min-h-screen flex flex-col items-center justify-center px-6 pt-24"
        style={{ opacity }}
      >
        <motion.div
          className="max-w-5xl mx-auto text-center"
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: "easeOut" }}
        >
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
          >
            <Badge variant="coral" className="mb-6">
              Now Available for Pre-Order
            </Badge>
          </motion.div>

          {/* Headline */}
          <motion.h1
            className="text-5xl md:text-7xl lg:text-8xl font-bold tracking-tight mb-6"
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.8 }}
          >
            <span className="gradient-text">Never forget</span>
            <br />
            <span className="text-[var(--foreground)]">what matters.</span>
          </motion.h1>

          {/* Subheadline */}
          <motion.p
            className="text-xl md:text-2xl text-[var(--muted)] max-w-2xl mx-auto mb-10"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5, duration: 0.8 }}
          >
            A beautiful ambient display that quietly tracks your recurring tasks.
            Always visible. Always on. Always keeping you on track.
          </motion.p>

          {/* CTA Buttons */}
          <motion.div
            className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-16"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7, duration: 0.8 }}
          >
            <Button size="lg">
              Pre-Order for $99
            </Button>
            <Button variant="outline" size="lg">
              Watch Demo
            </Button>
          </motion.div>
        </motion.div>

        {/* Product Image */}
        <motion.div
          className="relative w-full max-w-3xl mx-auto"
          style={{ y, scale }}
          initial={{ opacity: 0, y: 60 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.9, duration: 1 }}
        >
          {/* Device mockup */}
          <div className="relative aspect-[16/10] rounded-3xl overflow-hidden glow-coral">
            {/* Device frame */}
            <div className="absolute inset-0 bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-800 dark:to-gray-900 rounded-3xl p-4">
              {/* Device body - white case */}
              <div className="relative w-full h-full bg-white dark:bg-gray-100 rounded-2xl flex items-center justify-center shadow-2xl">
                {/* Screen bezel area */}
                <div className="absolute left-8 top-1/2 -translate-y-1/2 w-[55%] aspect-[4/3]">
                  {/* Screen frame with screws */}
                  <div className="absolute -top-2 -left-2 w-3 h-3 rounded-full bg-gray-400 shadow-inner" />
                  <div className="absolute -top-2 -right-2 w-3 h-3 rounded-full bg-gray-400 shadow-inner" />
                  <div className="absolute -bottom-2 -left-2 w-3 h-3 rounded-full bg-gray-400 shadow-inner" />
                  <div className="absolute -bottom-2 -right-2 w-3 h-3 rounded-full bg-gray-400 shadow-inner" />

                  {/* Screen */}
                  <div className="w-full h-full bg-gray-900 rounded-lg overflow-hidden border-4 border-gray-300">
                    {/* Screen content */}
                    <div className="w-full h-full bg-gradient-to-br from-gray-900 to-gray-800 p-3 flex flex-col">
                      {/* Dashboard header */}
                      <div className="text-[var(--pop-coral)] text-xs font-bold mb-2">DASHBOARD</div>

                      {/* Stats grid */}
                      <div className="grid grid-cols-2 gap-2 flex-1">
                        <div className="bg-[var(--pop-coral)]/20 rounded-lg p-2 flex flex-col justify-center">
                          <div className="text-[var(--pop-coral)] text-2xl font-bold">2</div>
                          <div className="text-gray-400 text-[10px]">Overdue</div>
                        </div>
                        <div className="bg-[var(--pop-orange)]/20 rounded-lg p-2 flex flex-col justify-center">
                          <div className="text-[var(--pop-orange)] text-2xl font-bold">1</div>
                          <div className="text-gray-400 text-[10px]">Today</div>
                        </div>
                        <div className="bg-[var(--pop-mint)]/20 rounded-lg p-2 flex flex-col justify-center">
                          <div className="text-[var(--pop-mint)] text-2xl font-bold">4</div>
                          <div className="text-gray-400 text-[10px]">This Week</div>
                        </div>
                        <div className="bg-[var(--pop-blue)]/20 rounded-lg p-2 flex flex-col justify-center">
                          <div className="text-[var(--pop-blue)] text-2xl font-bold">12</div>
                          <div className="text-gray-400 text-[10px]">Total</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Rotary encoder */}
                <div className="absolute right-12 top-1/2 -translate-y-1/2">
                  <div className="w-16 h-16 rounded-full bg-gradient-to-br from-gray-700 to-gray-900 shadow-xl flex items-center justify-center">
                    <div className="w-12 h-12 rounded-full bg-gradient-to-br from-gray-600 to-gray-800 flex items-center justify-center">
                      <div className="w-1 h-4 bg-gray-400 rounded-full" />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Floating elements */}
          <motion.div
            className="absolute -top-8 -right-8 bg-white dark:bg-[var(--dark-card)] rounded-2xl p-4 shadow-xl"
            animate={{ y: [0, -10, 0] }}
            transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
          >
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-[var(--pop-coral)]/10 flex items-center justify-center">
                <span className="text-[var(--pop-coral)] text-lg">!</span>
              </div>
              <div>
                <div className="text-sm font-semibold">Water Plants</div>
                <div className="text-xs text-[var(--pop-coral)]">2 days overdue</div>
              </div>
            </div>
          </motion.div>

          <motion.div
            className="absolute -bottom-4 -left-8 bg-white dark:bg-[var(--dark-card)] rounded-2xl p-4 shadow-xl"
            animate={{ y: [0, 10, 0] }}
            transition={{ duration: 5, repeat: Infinity, ease: "easeInOut", delay: 1 }}
          >
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-[var(--pop-mint)]/10 flex items-center justify-center">
                <svg className="w-5 h-5 text-[var(--pop-mint)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <div>
                <div className="text-sm font-semibold">Change Filter</div>
                <div className="text-xs text-[var(--pop-mint)]">Completed!</div>
              </div>
            </div>
          </motion.div>
        </motion.div>
      </motion.div>

      {/* Scroll indicator */}
      <motion.div
        className="absolute bottom-8 left-1/2 -translate-x-1/2"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.5 }}
      >
        <motion.div
          className="w-6 h-10 border-2 border-[var(--muted)] rounded-full flex justify-center"
          animate={{ y: [0, 8, 0] }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          <motion.div
            className="w-1.5 h-3 bg-[var(--pop-coral)] rounded-full mt-2"
            animate={{ opacity: [1, 0.3, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          />
        </motion.div>
      </motion.div>
    </section>
  );
}
