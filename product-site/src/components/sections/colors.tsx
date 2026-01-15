"use client";

import { useRef, useState } from "react";
import { motion, useInView, AnimatePresence } from "framer-motion";
import { Badge } from "../ui/badge";

const colors = [
  {
    name: "Pure White",
    value: "#FFFFFF",
    textColor: "#1A1A2E",
  },
  {
    name: "Charcoal",
    value: "#2D3436",
    textColor: "#FFFFFF",
  },
  {
    name: "Coral",
    value: "#E85D4C",
    textColor: "#FFFFFF",
  },
  {
    name: "Sage",
    value: "#8FA87E",
    textColor: "#FFFFFF",
  },
  {
    name: "Sky Blue",
    value: "#6B9AC4",
    textColor: "#FFFFFF",
  },
  {
    name: "Blush",
    value: "#E8B4B8",
    textColor: "#1A1A2E",
  },
];

function DeviceMockup({ color, isSelected }: { color: typeof colors[0]; isSelected: boolean }) {
  return (
    <motion.div
      className="relative cursor-pointer"
      whileHover={{ scale: 1.03 }}
      animate={{ scale: isSelected ? 1.05 : 1 }}
      transition={{ duration: 0.2 }}
    >
      {/* Device body */}
      <div
        className="w-28 h-[4.5rem] md:w-36 md:h-[5.5rem] rounded-xl shadow-elevated transition-all duration-300 flex items-center p-2.5 md:p-3"
        style={{ backgroundColor: color.value }}
      >
        {/* Screen */}
        <div className="w-[58%] h-[85%] bg-[var(--neutral-900)] rounded-lg mr-2">
          <div className="w-full h-full p-1.5 flex flex-col">
            <div className="text-[var(--accent)] text-[5px] md:text-[6px] font-medium tracking-wider">DASHBOARD</div>
            <div className="grid grid-cols-2 gap-0.5 flex-1 mt-0.5">
              <div className="bg-[var(--status-overdue)]/25 rounded-sm flex items-center justify-center">
                <span className="text-[var(--status-overdue)] text-[7px] md:text-[9px] font-bold">2</span>
              </div>
              <div className="bg-[var(--status-today)]/25 rounded-sm flex items-center justify-center">
                <span className="text-[var(--status-today)] text-[7px] md:text-[9px] font-bold">1</span>
              </div>
              <div className="bg-[var(--status-upcoming)]/25 rounded-sm flex items-center justify-center">
                <span className="text-[var(--status-upcoming)] text-[7px] md:text-[9px] font-bold">4</span>
              </div>
              <div className="bg-[var(--status-later)]/25 rounded-sm flex items-center justify-center">
                <span className="text-[var(--status-later)] text-[7px] md:text-[9px] font-bold">8</span>
              </div>
            </div>
          </div>
        </div>

        {/* Rotary encoder */}
        <div className="w-[28%] aspect-square rounded-full bg-[var(--neutral-800)] flex items-center justify-center">
          <div className="w-[70%] h-[70%] rounded-full bg-[var(--neutral-700)]">
            <div className="w-0.5 h-1/3 bg-[var(--neutral-500)] rounded-full mx-auto" />
          </div>
        </div>
      </div>

      {/* Selection ring */}
      <AnimatePresence>
        {isSelected && (
          <motion.div
            className="absolute -inset-1.5 border-2 border-[var(--accent)] rounded-2xl pointer-events-none"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
          />
        )}
      </AnimatePresence>
    </motion.div>
  );
}

export function Colors() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  const [selectedColor, setSelectedColor] = useState(colors[0]);

  return (
    <section
      id="colors"
      ref={ref}
      className="py-32 bg-[var(--neutral-50)] dark:bg-[var(--neutral-900)]"
    >
      <div className="max-w-6xl mx-auto px-6">
        {/* Section header */}
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
          transition={{ duration: 0.6 }}
        >
          <Badge className="mb-4">Colors</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl mb-6">
            <span className="font-serif italic text-[var(--accent)]">Pick your</span>
            <br />
            <span className="font-semibold">perfect finish.</span>
          </h2>
          <p className="text-lg text-[var(--muted)] max-w-xl mx-auto">
            Choose the color that matches your space and personality.
          </p>
        </motion.div>

        {/* Large preview */}
        <motion.div
          className="relative max-w-xl mx-auto mb-16"
          initial={{ opacity: 0, scale: 0.95 }}
          animate={isInView ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.6, delay: 0.2 }}
        >
          <AnimatePresence mode="wait">
            <motion.div
              key={selectedColor.name}
              initial={{ opacity: 0, y: 15 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -15 }}
              transition={{ duration: 0.3 }}
              className="aspect-[16/10] rounded-2xl shadow-floating flex items-center justify-center p-8"
              style={{ backgroundColor: selectedColor.value }}
            >
              {/* Large device mockup */}
              <div className="relative w-full max-w-md">
                <div
                  className="w-full aspect-[2/1] rounded-xl flex items-center p-5"
                  style={{ backgroundColor: selectedColor.value }}
                >
                  {/* Screen */}
                  <div className="w-[60%] h-[85%] bg-[var(--neutral-900)] rounded-lg mr-4 overflow-hidden">
                    <div className="w-full h-full p-3 flex flex-col">
                      <div className="text-[var(--accent)] text-[10px] font-medium tracking-wider mb-2">DASHBOARD</div>
                      <div className="grid grid-cols-2 gap-1.5 flex-1">
                        <div className="bg-[var(--status-overdue)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--status-overdue)] text-xl font-bold">2</span>
                          <span className="text-[var(--neutral-400)] text-[8px]">Overdue</span>
                        </div>
                        <div className="bg-[var(--status-today)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--status-today)] text-xl font-bold">1</span>
                          <span className="text-[var(--neutral-400)] text-[8px]">Today</span>
                        </div>
                        <div className="bg-[var(--status-upcoming)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--status-upcoming)] text-xl font-bold">4</span>
                          <span className="text-[var(--neutral-400)] text-[8px]">This Week</span>
                        </div>
                        <div className="bg-[var(--status-later)]/20 rounded-lg flex flex-col items-center justify-center">
                          <span className="text-[var(--status-later)] text-xl font-bold">12</span>
                          <span className="text-[var(--neutral-400)] text-[8px]">Total</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Rotary encoder */}
                  <div className="w-16 h-16 rounded-full bg-[var(--neutral-800)] flex items-center justify-center shadow-lg">
                    <div className="w-11 h-11 rounded-full bg-[var(--neutral-700)] flex items-center justify-center">
                      <div className="w-1 h-5 bg-[var(--neutral-500)] rounded-full" />
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          </AnimatePresence>

          {/* Color name */}
          <motion.div
            className="absolute -bottom-6 left-1/2 -translate-x-1/2 bg-white dark:bg-[var(--neutral-800)] px-5 py-2 rounded-full shadow-elevated"
            key={selectedColor.name}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.2 }}
          >
            <span className="font-medium text-sm">{selectedColor.name}</span>
          </motion.div>
        </motion.div>

        {/* Color grid - centered */}
        <motion.div
          className="flex flex-wrap justify-center gap-4 md:gap-6"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
          transition={{ duration: 0.6, delay: 0.3 }}
        >
          {colors.map((color) => (
            <motion.div
              key={color.name}
              onClick={() => setSelectedColor(color)}
            >
              <DeviceMockup
                color={color}
                isSelected={selectedColor.name === color.name}
              />
            </motion.div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
