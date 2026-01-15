"use client";

import { useRef } from "react";
import { motion, useScroll, useTransform } from "framer-motion";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

export function Hero() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ["start start", "end start"],
  });

  const y = useTransform(scrollYProgress, [0, 1], ["0%", "40%"]);
  const opacity = useTransform(scrollYProgress, [0, 0.5], [1, 0]);
  const scale = useTransform(scrollYProgress, [0, 0.5], [1, 0.95]);

  return (
    <section
      ref={containerRef}
      className="relative min-h-[180vh] overflow-hidden bg-[var(--neutral-50)] dark:bg-[var(--neutral-900)]"
    >
      {/* Subtle background texture */}
      <div className="absolute inset-0 grain" />

      {/* Sticky container */}
      <motion.div
        className="sticky top-0 min-h-screen flex flex-col items-center justify-center px-6 pt-24"
        style={{ opacity }}
      >
        <motion.div
          className="max-w-4xl mx-auto text-center"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: "easeOut" }}
        >
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
          >
            <Badge className="mb-8">Now Available for Pre-Order</Badge>
          </motion.div>

          {/* Headline - editorial typography */}
          <motion.h1
            className="mb-6"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.8 }}
          >
            <span className="block text-5xl md:text-6xl lg:text-7xl font-serif italic text-[var(--accent)]">
              Never forget
            </span>
            <span className="block text-5xl md:text-6xl lg:text-7xl font-semibold text-[var(--foreground)] mt-2">
              what matters.
            </span>
          </motion.h1>

          {/* Subheadline */}
          <motion.p
            className="text-lg md:text-xl text-[var(--muted)] max-w-xl mx-auto mb-10 leading-relaxed"
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5, duration: 0.8 }}
          >
            A beautiful ambient display that quietly tracks your recurring tasks.
            Always visible. Always on. Always keeping you on track.
          </motion.p>

          {/* CTA Buttons */}
          <motion.div
            className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-20"
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7, duration: 0.8 }}
          >
            <Button size="lg">Pre-Order for $99</Button>
            <Button variant="outline" size="lg">
              Watch Demo
            </Button>
          </motion.div>
        </motion.div>

        {/* Product Image - simplified device mockup */}
        <motion.div
          className="relative w-full max-w-2xl mx-auto"
          style={{ y, scale }}
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.9, duration: 1 }}
        >
          {/* Device frame */}
          <div className="relative aspect-[16/10] rounded-2xl overflow-hidden shadow-floating">
            {/* Device body - clean white */}
            <div className="absolute inset-0 bg-white dark:bg-[var(--neutral-100)] rounded-2xl p-6 flex items-center">
              {/* Screen area */}
              <div className="relative w-[60%] aspect-[4/3] bg-[var(--neutral-900)] rounded-xl overflow-hidden">
                {/* Screen content */}
                <div className="absolute inset-0 p-4 flex flex-col">
                  <div className="text-[var(--accent)] text-[10px] font-medium tracking-wider mb-3">
                    DASHBOARD
                  </div>

                  {/* Stats grid */}
                  <div className="grid grid-cols-2 gap-2 flex-1">
                    <div className="bg-[var(--status-overdue)]/15 rounded-lg p-3 flex flex-col justify-center">
                      <div className="text-[var(--status-overdue)] text-2xl font-bold">2</div>
                      <div className="text-[var(--neutral-400)] text-[9px] mt-0.5">Overdue</div>
                    </div>
                    <div className="bg-[var(--status-today)]/15 rounded-lg p-3 flex flex-col justify-center">
                      <div className="text-[var(--status-today)] text-2xl font-bold">1</div>
                      <div className="text-[var(--neutral-400)] text-[9px] mt-0.5">Today</div>
                    </div>
                    <div className="bg-[var(--status-upcoming)]/15 rounded-lg p-3 flex flex-col justify-center">
                      <div className="text-[var(--status-upcoming)] text-2xl font-bold">4</div>
                      <div className="text-[var(--neutral-400)] text-[9px] mt-0.5">This Week</div>
                    </div>
                    <div className="bg-[var(--status-later)]/15 rounded-lg p-3 flex flex-col justify-center">
                      <div className="text-[var(--status-later)] text-2xl font-bold">12</div>
                      <div className="text-[var(--neutral-400)] text-[9px] mt-0.5">Total</div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Rotary encoder */}
              <div className="absolute right-8 top-1/2 -translate-y-1/2">
                <div className="w-14 h-14 rounded-full bg-[var(--neutral-800)] flex items-center justify-center shadow-lg">
                  <div className="w-10 h-10 rounded-full bg-[var(--neutral-700)] flex items-center justify-center">
                    <div className="w-0.5 h-3 bg-[var(--neutral-400)] rounded-full" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Floating notification card - top right */}
          <motion.div
            className="absolute -top-4 -right-4 md:-right-8 bg-white dark:bg-[var(--neutral-800)] rounded-xl p-3 shadow-elevated"
            animate={{ y: [0, -6, 0] }}
            transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
          >
            <div className="flex items-center gap-2.5">
              <div className="w-8 h-8 rounded-full bg-[var(--status-overdue)]/10 flex items-center justify-center">
                <span className="text-[var(--status-overdue)] text-sm font-medium">!</span>
              </div>
              <div>
                <div className="text-sm font-medium text-[var(--foreground)]">Water Plants</div>
                <div className="text-xs text-[var(--status-overdue)]">2 days overdue</div>
              </div>
            </div>
          </motion.div>

          {/* Floating success card - bottom left */}
          <motion.div
            className="absolute -bottom-2 -left-4 md:-left-8 bg-white dark:bg-[var(--neutral-800)] rounded-xl p-3 shadow-elevated"
            animate={{ y: [0, 6, 0] }}
            transition={{ duration: 5, repeat: Infinity, ease: "easeInOut", delay: 1 }}
          >
            <div className="flex items-center gap-2.5">
              <div className="w-8 h-8 rounded-full bg-[var(--status-upcoming)]/10 flex items-center justify-center">
                <svg className="w-4 h-4 text-[var(--status-upcoming)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <div>
                <div className="text-sm font-medium text-[var(--foreground)]">Change Filter</div>
                <div className="text-xs text-[var(--status-upcoming)]">Completed</div>
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
          className="w-5 h-8 border border-[var(--neutral-300)] rounded-full flex justify-center"
          animate={{ y: [0, 6, 0] }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          <motion.div
            className="w-1 h-2 bg-[var(--accent)] rounded-full mt-1.5"
            animate={{ opacity: [1, 0.4, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          />
        </motion.div>
      </motion.div>
    </section>
  );
}
