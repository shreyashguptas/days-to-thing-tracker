"use client";

import { useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Card } from "../ui/card";
import { Badge } from "../ui/badge";

const features = [
  {
    number: "01",
    title: "Always Visible",
    description: "No apps to open. No notifications to check. Your tasks are always in view, quietly reminding you what needs attention.",
  },
  {
    number: "02",
    title: "Smart Schedules",
    description: "Set recurring tasks with flexible patterns - daily, weekly, monthly, or yearly. The schedule stays fixed, so you never fall behind.",
  },
  {
    number: "03",
    title: "Color-Coded Urgency",
    description: "Instantly see what's overdue, due today, or coming up. Color-coded indicators help you prioritize at a glance.",
  },
  {
    number: "04",
    title: "Mobile Control",
    description: "Manage tasks from your phone. Add, edit, or complete tasks remotely through the beautiful web interface.",
  },
  {
    number: "05",
    title: "Instant Response",
    description: "Hardware-accelerated display with microsecond latency. The rotary encoder responds instantly to your every turn.",
  },
  {
    number: "06",
    title: "Works Offline",
    description: "No internet required. Everything runs locally on the device. Your data stays private and the display works forever.",
  },
];

function FeatureCard({ feature, index }: { feature: typeof features[0]; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 40 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 40 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
    >
      <Card hover className="h-full">
        <div className="text-4xl font-serif text-[var(--accent)] mb-4">
          {feature.number}
        </div>
        <h3 className="text-xl font-semibold mb-3 text-[var(--foreground)]">
          {feature.title}
        </h3>
        <p className="text-[var(--muted)] leading-relaxed">
          {feature.description}
        </p>
      </Card>
    </motion.div>
  );
}

export function Features() {
  const containerRef = useRef(null);

  return (
    <section
      id="features"
      ref={containerRef}
      className="relative py-32 bg-[var(--background)]"
    >
      <div className="max-w-6xl mx-auto px-6">
        {/* Section header */}
        <div className="text-center mb-20">
          <Badge className="mb-4">Features</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl mb-6">
            <span className="font-serif italic text-[var(--accent)]">Thoughtfully designed</span>
            <br />
            <span className="font-semibold">for real life.</span>
          </h2>
          <p className="text-lg text-[var(--muted)] max-w-xl mx-auto">
            Every feature is crafted to help you stay on top of life&apos;s recurring tasks
            without adding complexity.
          </p>
        </div>

        {/* Features grid */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <FeatureCard key={feature.title} feature={feature} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}
