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

  const gradients = [
    "from-[var(--pop-coral)] to-[var(--pop-orange)]",
    "from-[var(--pop-orange)] to-[var(--pop-yellow)]",
    "from-[var(--pop-mint)] to-[var(--pop-cyan)]",
    "from-[var(--pop-cyan)] to-[var(--pop-blue)]",
    "from-[var(--pop-blue)] to-[var(--pop-purple)]",
    "from-[var(--pop-purple)] to-[var(--pop-pink)]",
  ];

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
      className="bg-white dark:bg-[var(--dark-card)] rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-[var(--dark-border)]"
    >
      {/* Category header */}
      <div className={`inline-flex items-center gap-2 px-3 py-1 rounded-full bg-gradient-to-r ${gradients[index]} text-white text-sm font-medium mb-4`}>
        {spec.category}
      </div>

      {/* Specs list */}
      <div className="space-y-3">
        {spec.items.map((item, i) => (
          <motion.div
            key={item.label}
            className="flex justify-between items-center"
            initial={{ opacity: 0, x: -20 }}
            animate={isInView ? { opacity: 1, x: 0 } : { opacity: 0, x: -20 }}
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
    <section id="specs" ref={ref} className="py-32">
      <div className="max-w-7xl mx-auto px-6">
        {/* Section header */}
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 40 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 40 }}
          transition={{ duration: 0.6 }}
        >
          <Badge variant="cyan" className="mb-4">Specifications</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6">
            <span>Built for</span>
            <br />
            <span className="gradient-text-cool">performance.</span>
          </h2>
          <p className="text-xl text-[var(--muted)] max-w-2xl mx-auto">
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
          <p className="text-[var(--muted)]">
            All specifications subject to change. Final product may vary slightly.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
