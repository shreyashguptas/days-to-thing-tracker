"use client";

import { useRef } from "react";
import { motion, useScroll, useTransform, useInView } from "framer-motion";
import { Badge } from "../ui/badge";

const steps = [
  {
    number: "01",
    title: "Set it up once",
    description: "Place DaysTracker on your kitchen counter, mount it on a wall, or put it anywhere you'll see it daily. Connect power and you're done.",
    visual: (
      <div className="relative w-full aspect-square max-w-md mx-auto">
        {/* Kitchen counter scene */}
        <div className="absolute inset-0 bg-gradient-to-br from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20 rounded-3xl" />
        <div className="absolute bottom-0 left-0 right-0 h-1/3 bg-gradient-to-t from-amber-100/50 to-transparent rounded-b-3xl" />

        {/* Device on counter */}
        <motion.div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
          animate={{ y: [0, -5, 0] }}
          transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="w-48 h-32 bg-white rounded-xl shadow-2xl flex items-center p-3">
            <div className="w-24 h-20 bg-gray-900 rounded-lg mr-3" />
            <div className="w-10 h-10 rounded-full bg-gray-800" />
          </div>
        </motion.div>

        {/* Coffee cup */}
        <div className="absolute bottom-8 left-8 w-12 h-14 bg-white rounded-lg shadow-lg" />

        {/* Plant */}
        <div className="absolute bottom-8 right-8">
          <div className="w-8 h-12 bg-green-500 rounded-t-full" />
          <div className="w-10 h-6 bg-amber-600 rounded-lg -mt-1" />
        </div>
      </div>
    ),
  },
  {
    number: "02",
    title: "Add your tasks",
    description: "Scan the QR code to open the mobile interface. Add tasks like 'Water plants every 3 days' or 'Change air filter monthly'. Takes 30 seconds.",
    visual: (
      <div className="relative w-full aspect-square max-w-md mx-auto">
        {/* Phone mockup */}
        <motion.div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-56 h-[450px] bg-gray-900 rounded-[3rem] p-2 shadow-2xl"
          animate={{ rotateY: [0, 5, 0] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="w-full h-full bg-white rounded-[2.5rem] overflow-hidden">
            {/* Status bar */}
            <div className="h-8 bg-gray-100 flex items-center justify-center">
              <div className="w-20 h-5 bg-gray-900 rounded-full" />
            </div>

            {/* App content */}
            <div className="p-4">
              <div className="text-lg font-bold text-gray-900 mb-4">Add Task</div>

              {/* Form fields */}
              <div className="space-y-3">
                <div className="h-10 bg-gray-100 rounded-lg px-3 flex items-center">
                  <span className="text-gray-400 text-sm">Water plants</span>
                </div>
                <div className="h-10 bg-gray-100 rounded-lg px-3 flex items-center justify-between">
                  <span className="text-gray-400 text-sm">Every 3 days</span>
                  <span className="text-[var(--pop-coral)]">▼</span>
                </div>
                <motion.div
                  className="h-12 bg-gradient-to-r from-[var(--pop-coral)] to-[var(--pop-orange)] rounded-xl flex items-center justify-center"
                  animate={{ scale: [1, 1.02, 1] }}
                  transition={{ duration: 2, repeat: Infinity }}
                >
                  <span className="text-white font-semibold">Save Task</span>
                </motion.div>
              </div>

              {/* Existing tasks preview */}
              <div className="mt-6 space-y-2">
                {["Air filter", "Pet grooming", "Car service"].map((task, i) => (
                  <div key={task} className="flex items-center gap-2 p-2 bg-gray-50 rounded-lg">
                    <div className={`w-2 h-2 rounded-full ${i === 0 ? "bg-[var(--pop-coral)]" : i === 1 ? "bg-[var(--pop-orange)]" : "bg-[var(--pop-mint)]"}`} />
                    <span className="text-sm text-gray-700">{task}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </motion.div>

        {/* Decorative dots */}
        <div className="absolute top-10 right-10 w-4 h-4 bg-[var(--pop-coral)] rounded-full opacity-50" />
        <div className="absolute bottom-20 left-10 w-3 h-3 bg-[var(--pop-mint)] rounded-full opacity-50" />
      </div>
    ),
  },
  {
    number: "03",
    title: "Stay on track",
    description: "Just glance at DaysTracker throughout your day. Color-coded urgency shows you what needs attention. Turn the knob to see details and mark tasks complete.",
    visual: (
      <div className="relative w-full aspect-square max-w-md mx-auto flex items-center justify-center">
        {/* Central device with glowing screen */}
        <motion.div
          className="relative"
          animate={{ scale: [1, 1.02, 1] }}
          transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="w-64 h-44 bg-white rounded-2xl shadow-2xl p-4 flex items-center">
            {/* Screen with active display */}
            <div className="w-36 h-28 bg-gray-900 rounded-lg overflow-hidden mr-4">
              <div className="p-2 h-full flex flex-col">
                <div className="text-[var(--pop-coral)] text-[8px] font-bold mb-1">OVERDUE</div>
                <div className="flex-1 space-y-1">
                  <div className="bg-[var(--pop-coral)]/20 rounded p-1">
                    <div className="text-white text-[8px] font-medium">Water plants</div>
                    <div className="text-[var(--pop-coral)] text-[6px]">2 days overdue</div>
                  </div>
                  <div className="bg-[var(--pop-orange)]/20 rounded p-1">
                    <div className="text-white text-[8px] font-medium">Check smoke detector</div>
                    <div className="text-[var(--pop-orange)] text-[6px]">Due today</div>
                  </div>
                </div>
              </div>
            </div>

            {/* Rotary encoder with rotation indicator */}
            <motion.div
              className="w-14 h-14 rounded-full bg-gradient-to-br from-gray-700 to-gray-900 flex items-center justify-center shadow-lg"
              animate={{ rotate: [0, 15, 0, -15, 0] }}
              transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
            >
              <div className="w-10 h-10 rounded-full bg-gray-800 flex items-center justify-center">
                <div className="w-1 h-4 bg-gray-400 rounded-full" />
              </div>
            </motion.div>
          </div>

          {/* Glow effect */}
          <div className="absolute inset-0 bg-gradient-to-r from-[var(--pop-coral)]/20 to-[var(--pop-orange)]/20 rounded-2xl blur-xl -z-10" />
        </motion.div>

        {/* Floating checkmarks */}
        <motion.div
          className="absolute top-8 right-12 w-10 h-10 bg-[var(--pop-mint)] rounded-full flex items-center justify-center text-white shadow-lg"
          animate={{ y: [0, -10, 0], opacity: [0.5, 1, 0.5] }}
          transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
          </svg>
        </motion.div>

        <motion.div
          className="absolute bottom-12 left-8 w-8 h-8 bg-[var(--pop-mint)] rounded-full flex items-center justify-center text-white shadow-lg"
          animate={{ y: [0, 10, 0], opacity: [0.3, 0.8, 0.3] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut", delay: 1 }}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
          </svg>
        </motion.div>
      </div>
    ),
  },
];

function Step({ step, index }: { step: typeof steps[0]; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-20%" });

  return (
    <motion.div
      ref={ref}
      className="grid lg:grid-cols-2 gap-12 lg:gap-20 items-center"
      initial={{ opacity: 0, y: 80 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 80 }}
      transition={{ duration: 0.8, delay: 0.2 }}
    >
      <div className={`${index % 2 === 1 ? "lg:order-2" : ""}`}>
        <div className="inline-block text-7xl font-bold gradient-text mb-4">
          {step.number}
        </div>
        <h3 className="text-3xl md:text-4xl font-bold mb-4">{step.title}</h3>
        <p className="text-xl text-[var(--muted)] leading-relaxed">
          {step.description}
        </p>
      </div>
      <div className={`${index % 2 === 1 ? "lg:order-1" : ""}`}>
        {step.visual}
      </div>
    </motion.div>
  );
}

export function HowItWorks() {
  const containerRef = useRef(null);

  return (
    <section
      id="how-it-works"
      ref={containerRef}
      className="py-32 bg-gradient-to-b from-white to-[var(--subtle-sky)] dark:from-[var(--dark-bg)] dark:to-[var(--dark-card)]"
    >
      <div className="max-w-7xl mx-auto px-6">
        {/* Section header */}
        <div className="text-center mb-24">
          <Badge variant="blue" className="mb-4">How It Works</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6">
            <span>Simple as</span>
            <br />
            <span className="gradient-text">one, two, three.</span>
          </h2>
          <p className="text-xl text-[var(--muted)] max-w-2xl mx-auto">
            No complicated setup. No learning curve. Just place it, add tasks, and let it work for you.
          </p>
        </div>

        {/* Steps */}
        <div className="space-y-32">
          {steps.map((step, index) => (
            <Step key={step.number} step={step} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}
