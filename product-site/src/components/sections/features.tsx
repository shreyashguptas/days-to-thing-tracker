"use client";

import { useRef } from "react";
import { motion, useScroll, useTransform, useInView } from "framer-motion";
import { Card } from "../ui/card";
import { Badge } from "../ui/badge";

const features = [
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
      </svg>
    ),
    title: "Always Visible",
    description: "No apps to open. No notifications to check. Your tasks are always in view, quietly reminding you what needs attention.",
    color: "coral" as const,
    gradient: "from-[var(--pop-coral)] to-[var(--pop-orange)]",
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    title: "Smart Schedules",
    description: "Set recurring tasks with flexible patterns - daily, weekly, monthly, or yearly. The schedule stays fixed, so you never fall behind.",
    color: "mint" as const,
    gradient: "from-[var(--pop-mint)] to-[var(--pop-cyan)]",
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
      </svg>
    ),
    title: "Color-Coded Urgency",
    description: "Instantly see what's overdue, due today, or coming up. Color-coded indicators help you prioritize at a glance.",
    color: "orange" as const,
    gradient: "from-[var(--pop-orange)] to-[var(--pop-yellow)]",
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
      </svg>
    ),
    title: "Mobile Control",
    description: "Manage tasks from your phone. Add, edit, or complete tasks remotely through the beautiful web interface.",
    color: "blue" as const,
    gradient: "from-[var(--pop-blue)] to-[var(--pop-purple)]",
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
    title: "Instant Response",
    description: "Hardware-accelerated display with microsecond latency. The rotary encoder responds instantly to your every turn.",
    color: "purple" as const,
    gradient: "from-[var(--pop-purple)] to-[var(--pop-pink)]",
  },
  {
    icon: (
      <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z" />
      </svg>
    ),
    title: "Works Offline",
    description: "No internet required. Everything runs locally on the device. Your data stays private and the display works forever.",
    color: "cyan" as const,
    gradient: "from-[var(--pop-cyan)] to-[var(--pop-mint)]",
  },
];

function FeatureCard({ feature, index }: { feature: typeof features[0]; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 50 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 50 }}
      transition={{ duration: 0.6, delay: index * 0.1 }}
    >
      <Card hover className="h-full group">
        <div
          className={`w-14 h-14 rounded-2xl bg-gradient-to-br ${feature.gradient} flex items-center justify-center text-white mb-4 group-hover:scale-110 transition-transform duration-300`}
        >
          {feature.icon}
        </div>
        <h3 className="text-xl font-bold mb-2">{feature.title}</h3>
        <p className="text-[var(--muted)] leading-relaxed">{feature.description}</p>
      </Card>
    </motion.div>
  );
}

export function Features() {
  const containerRef = useRef(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ["start end", "end start"],
  });

  const backgroundY = useTransform(scrollYProgress, [0, 1], ["0%", "30%"]);

  return (
    <section
      id="features"
      ref={containerRef}
      className="relative py-32 overflow-hidden"
    >
      {/* Background decorations */}
      <motion.div
        className="absolute inset-0 pointer-events-none"
        style={{ y: backgroundY }}
      >
        <div className="absolute top-0 right-0 w-[600px] h-[600px] bg-[var(--pop-mint)]/5 rounded-full blur-3xl" />
        <div className="absolute bottom-0 left-0 w-[500px] h-[500px] bg-[var(--pop-blue)]/5 rounded-full blur-3xl" />
      </motion.div>

      <div className="max-w-7xl mx-auto px-6 relative z-10">
        {/* Section header */}
        <div className="text-center mb-20">
          <Badge variant="mint" className="mb-4">Features</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6">
            <span className="gradient-text-cool">Thoughtfully designed</span>
            <br />
            <span>for real life.</span>
          </h2>
          <p className="text-xl text-[var(--muted)] max-w-2xl mx-auto">
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
