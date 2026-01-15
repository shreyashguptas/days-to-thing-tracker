"use client";

import { useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Badge } from "../ui/badge";

const specs = [
  {
    category: "Display",
    items: [
      { label: "Screen Size", value: '1.8" TFT LCD' },
      { label: "Resolution", value: "160 x 128 pixels" },
      { label: "Color Depth", value: "65K colors" },
      { label: "Backlight", value: "LED with auto-timeout" },
    ],
  },
  {
    category: "Controls",
    items: [
      { label: "Input", value: "Rotary encoder with push" },
      { label: "Latency", value: "<10 microseconds" },
      { label: "Actions", value: "Scroll, select, long-press" },
      { label: "Feedback", value: "Tactile clicks" },
    ],
  },
  {
    category: "Connectivity",
    items: [
      { label: "WiFi", value: "2.4GHz 802.11 b/g/n" },
      { label: "Web Interface", value: "Responsive mobile UI" },
      { label: "API", value: "REST API on port 8080" },
      { label: "QR Code", value: "Quick device pairing" },
    ],
  },
  {
    category: "Hardware",
    items: [
      { label: "Processor", value: "Raspberry Pi Zero 2 W" },
      { label: "Storage", value: "8GB+ microSD" },
      { label: "Power", value: "5V 2A USB-C" },
      { label: "Consumption", value: "<3W typical" },
    ],
  },
  {
    category: "Physical",
    items: [
      { label: "Dimensions", value: '4.5" x 2.5" x 1.2"' },
      { label: "Weight", value: "120g / 4.2oz" },
      { label: "Materials", value: "ABS plastic shell" },
      { label: "Mounting", value: "Desk or wall mount" },
    ],
  },
  {
    category: "Software",
    items: [
      { label: "OS", value: "Raspberry Pi OS" },
      { label: "Updates", value: "OTA via WiFi" },
      { label: "Data", value: "Local SQLite database" },
      { label: "Privacy", value: "100% offline capable" },
    ],
  },
];

function SpecCard({ spec, index }: { spec: typeof specs[0]; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 25 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 25 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
      className="bg-white dark:bg-[var(--neutral-800)] rounded-2xl p-6 border border-[var(--border)]"
    >
      {/* Category header */}
      <div className="text-sm font-medium text-[var(--accent)] mb-4 tracking-wide">
        {spec.category}
      </div>

      {/* Specs list */}
      <div className="space-y-3">
        {spec.items.map((item, i) => (
          <motion.div
            key={item.label}
            className="flex justify-between items-center"
            initial={{ opacity: 0, x: -15 }}
            animate={isInView ? { opacity: 1, x: 0 } : { opacity: 0, x: -15 }}
            transition={{ duration: 0.3, delay: index * 0.1 + i * 0.05 + 0.2 }}
          >
            <span className="text-[var(--muted)] text-sm">{item.label}</span>
            <span className="font-medium text-sm">{item.value}</span>
          </motion.div>
        ))}
      </div>
    </motion.div>
  );
}

export function Specs() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <section id="specs" ref={ref} className="py-32 bg-[var(--neutral-50)] dark:bg-[var(--neutral-900)]">
      <div className="max-w-6xl mx-auto px-6">
        {/* Section header */}
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
          transition={{ duration: 0.6 }}
        >
          <Badge className="mb-4">Specifications</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl mb-6">
            <span className="font-serif italic text-[var(--accent)]">Built for</span>
            <br />
            <span className="font-semibold">performance.</span>
          </h2>
          <p className="text-lg text-[var(--muted)] max-w-xl mx-auto">
            Powerful hardware meets elegant software. Every detail optimized for speed, reliability, and longevity.
          </p>
        </motion.div>

        {/* Specs grid */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {specs.map((spec, index) => (
            <SpecCard key={spec.category} spec={spec} index={index} />
          ))}
        </div>

        {/* Bottom note */}
        <motion.div
          className="text-center mt-12"
          initial={{ opacity: 0 }}
          animate={isInView ? { opacity: 1 } : { opacity: 0 }}
          transition={{ delay: 0.8 }}
        >
          <p className="text-sm text-[var(--muted)]">
            All specifications subject to change. Final product may vary slightly.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
