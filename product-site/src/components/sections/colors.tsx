"use client";

import { useRef, useState } from "react";
import { motion, useInView, AnimatePresence } from "framer-motion";
import { Badge } from "../ui/badge";

const colors = [
  {
    name: "Pure White",
    value: "#FFFFFF",
    accent: "#F5F5F5",
    textColor: "#1A1A2E",
    category: "subtle",
    popular: true,
  },
  {
    name: "Coral Pop",
    value: "#FF6B6B",
    accent: "#FF8787",
    textColor: "#FFFFFF",
    category: "popping",
    popular: true,
  },
  {
    name: "Mint Fresh",
    value: "#1DD1A1",
    accent: "#2EE6B6",
    textColor: "#FFFFFF",
    category: "popping",
  },
  {
    name: "Ocean Blue",
    value: "#54A0FF",
    accent: "#6AAFFF",
    textColor: "#FFFFFF",
    category: "popping",
  },
  {
    name: "Sunset Orange",
    value: "#FF9F43",
    accent: "#FFB366",
    textColor: "#FFFFFF",
    category: "popping",
  },
  {
    name: "Deep Purple",
    value: "#5F27CD",
    accent: "#7C4DFF",
    textColor: "#FFFFFF",
    category: "popping",
  },
  {
    name: "Charcoal",
    value: "#2D3436",
    accent: "#3D4548",
    textColor: "#FFFFFF",
    category: "subtle",
  },
  {
    name: "Blush Pink",
    value: "#FF6B9D",
    accent: "#FF85AD",
    textColor: "#FFFFFF",
    category: "popping",
  },
  {
    name: "Sage Green",
    value: "#A8E6CF",
    accent: "#B8EFDB",
    textColor: "#1A1A2E",
    category: "subtle",
  },
  {
    name: "Sky Blue",
    value: "#74B9FF",
    accent: "#85C5FF",
    textColor: "#1A1A2E",
    category: "subtle",
  },
  {
    name: "Lemon Yellow",
    value: "#FECA57",
    accent: "#FFD56B",
    textColor: "#1A1A2E",
    category: "popping",
  },
  {
    name: "Lavender",
    value: "#D6BCFA",
    accent: "#E0C9FF",
    textColor: "#1A1A2E",
    category: "subtle",
  },
];

function DeviceMockup({ color, isSelected }: { color: typeof colors[0]; isSelected: boolean }) {
  return (
    <motion.div
      className="relative cursor-pointer group"
      whileHover={{ scale: 1.05 }}
      animate={{ scale: isSelected ? 1.1 : 1 }}
      transition={{ duration: 0.3 }}
    >
      {/* Device body */}
      <div
        className="w-24 h-16 md:w-32 md:h-20 rounded-xl shadow-lg transition-all duration-300 flex items-center p-2 md:p-3"
        style={{ backgroundColor: color.value }}
      >
        {/* Screen */}
        <div className="w-[55%] h-[80%] bg-gray-900 rounded-md mr-1.5 md:mr-2">
          <div className="w-full h-full p-1 flex flex-col">
            <div className="text-[var(--pop-coral)] text-[4px] md:text-[5px] font-bold">DASHBOARD</div>
            <div className="grid grid-cols-2 gap-0.5 flex-1 mt-0.5">
              <div className="bg-[var(--pop-coral)]/30 rounded-sm flex items-center justify-center">
                <span className="text-[var(--pop-coral)] text-[6px] md:text-[8px] font-bold">2</span>
              </div>
              <div className="bg-[var(--pop-orange)]/30 rounded-sm flex items-center justify-center">
                <span className="text-[var(--pop-orange)] text-[6px] md:text-[8px] font-bold">1</span>
              </div>
              <div className="bg-[var(--pop-mint)]/30 rounded-sm flex items-center justify-center">
                <span className="text-[var(--pop-mint)] text-[6px] md:text-[8px] font-bold">4</span>
              </div>
              <div className="bg-[var(--pop-blue)]/30 rounded-sm flex items-center justify-center">
                <span className="text-[var(--pop-blue)] text-[6px] md:text-[8px] font-bold">8</span>
              </div>
            </div>
          </div>
        </div>

        {/* Rotary encoder */}
        <div className="w-[25%] aspect-square rounded-full bg-gray-800 flex items-center justify-center">
          <div className="w-[70%] h-[70%] rounded-full bg-gray-700">
            <div className="w-0.5 h-1/3 bg-gray-500 rounded-full mx-auto" />
          </div>
        </div>
      </div>

      {/* Selection ring */}
      <AnimatePresence>
        {isSelected && (
          <motion.div
            className="absolute -inset-2 border-2 rounded-2xl pointer-events-none"
            style={{ borderColor: color.category === "subtle" ? color.textColor : color.value }}
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.9 }}
          />
        )}
      </AnimatePresence>

      {/* Popular badge */}
      {color.popular && (
        <div className="absolute -top-1 -right-1 md:-top-2 md:-right-2 bg-[var(--pop-coral)] text-white text-[8px] md:text-xs px-1.5 md:px-2 py-0.5 rounded-full font-medium">
          Popular
        </div>
      )}
    </motion.div>
  );
}

export function Colors() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  const [selectedColor, setSelectedColor] = useState(colors[0]);
  const [filter, setFilter] = useState<"all" | "popping" | "subtle">("all");

  const filteredColors = colors.filter(
    (c) => filter === "all" || c.category === filter
  );

  return (
    <section
      id="colors"
      ref={ref}
      className="py-32 bg-gradient-to-b from-[var(--subtle-lavender)] to-white dark:from-[var(--dark-card)] dark:to-[var(--dark-bg)]"
    >
      <div className="max-w-7xl mx-auto px-6">
        {/* Section header */}
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 40 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 40 }}
          transition={{ duration: 0.6 }}
        >
          <Badge variant="purple" className="mb-4">Colors</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6">
            <span>Pick your</span>
            <br />
            <span className="gradient-text">perfect vibe.</span>
          </h2>
          <p className="text-xl text-[var(--muted)] max-w-2xl mx-auto">
            From bold and vibrant to calm and subtle. Choose the color that matches your space and personality.
          </p>
        </motion.div>

        {/* Filter tabs */}
        <motion.div
          className="flex justify-center gap-4 mb-12"
          initial={{ opacity: 0, y: 20 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 20 }}
          transition={{ duration: 0.6, delay: 0.2 }}
        >
          {[
            { value: "all" as const, label: "All Colors" },
            { value: "popping" as const, label: "Popping" },
            { value: "subtle" as const, label: "Subtle" },
          ].map((tab) => (
            <button
              key={tab.value}
              onClick={() => setFilter(tab.value)}
              className={`px-6 py-2 rounded-full font-medium transition-all duration-300 ${
                filter === tab.value
                  ? "bg-gradient-to-r from-[var(--pop-coral)] to-[var(--pop-orange)] text-white shadow-lg"
                  : "bg-white dark:bg-[var(--dark-card)] text-[var(--muted)] hover:text-[var(--foreground)] shadow-sm"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </motion.div>

        {/* Large preview */}
        <motion.div
          className="relative max-w-2xl mx-auto mb-16"
          initial={{ opacity: 0, scale: 0.9 }}
          animate={isInView ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.9 }}
          transition={{ duration: 0.6, delay: 0.3 }}
        >
          <AnimatePresence mode="wait">
            <motion.div
              key={selectedColor.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.4 }}
              className="aspect-[16/10] rounded-3xl shadow-2xl flex items-center justify-center p-8"
              style={{ backgroundColor: selectedColor.value }}
            >
              {/* Large device mockup */}
              <div className="relative w-full max-w-lg">
                <div
                  className="w-full aspect-[2/1] rounded-2xl flex items-center p-6"
                  style={{ backgroundColor: selectedColor.value, boxShadow: "inset 0 0 40px rgba(0,0,0,0.1)" }}
                >
                  {/* Screen */}
                  <div className="w-[60%] h-[80%] bg-gray-900 rounded-lg mr-4 overflow-hidden">
                    <div className="w-full h-full p-4 flex flex-col">
                      <div className="text-[var(--pop-coral)] text-xs font-bold mb-2">DASHBOARD</div>
                      <div className="grid grid-cols-2 gap-2 flex-1">
                        <div className="bg-[var(--pop-coral)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--pop-coral)] text-2xl font-bold">2</span>
                          <span className="text-gray-400 text-[10px]">Overdue</span>
                        </div>
                        <div className="bg-[var(--pop-orange)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--pop-orange)] text-2xl font-bold">1</span>
                          <span className="text-gray-400 text-[10px]">Today</span>
                        </div>
                        <div className="bg-[var(--pop-mint)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--pop-mint)] text-2xl font-bold">4</span>
                          <span className="text-gray-400 text-[10px]">This Week</span>
                        </div>
                        <div className="bg-[var(--pop-blue)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--pop-blue)] text-2xl font-bold">12</span>
                          <span className="text-gray-400 text-[10px]">Total</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Rotary encoder */}
                  <div className="w-20 h-20 rounded-full bg-gradient-to-br from-gray-700 to-gray-900 flex items-center justify-center shadow-xl">
                    <div className="w-14 h-14 rounded-full bg-gradient-to-br from-gray-600 to-gray-800 flex items-center justify-center">
                      <div className="w-1.5 h-6 bg-gray-400 rounded-full" />
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          </AnimatePresence>

          {/* Color name */}
          <motion.div
            className="absolute -bottom-8 left-1/2 -translate-x-1/2 bg-white dark:bg-[var(--dark-card)] px-6 py-2 rounded-full shadow-lg"
            key={selectedColor.name}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
          >
            <span className="font-semibold">{selectedColor.name}</span>
          </motion.div>
        </motion.div>

        {/* Color grid */}
        <motion.div
          className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 gap-4 md:gap-6 justify-items-center"
          initial={{ opacity: 0, y: 40 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 40 }}
          transition={{ duration: 0.6, delay: 0.4 }}
        >
          <AnimatePresence mode="popLayout">
            {filteredColors.map((color) => (
              <motion.div
                key={color.name}
                layout
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.8 }}
                onClick={() => setSelectedColor(color)}
              >
                <DeviceMockup
                  color={color}
                  isSelected={selectedColor.name === color.name}
                />
              </motion.div>
            ))}
          </AnimatePresence>
        </motion.div>
      </div>
    </section>
  );
}
