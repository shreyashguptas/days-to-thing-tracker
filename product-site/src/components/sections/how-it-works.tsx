"use client";

import { useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Badge } from "../ui/badge";

const steps = [
  {
    number: "01",
    title: "Set it up once",
    description: "Place DaysTracker on your kitchen counter, mount it on a wall, or put it anywhere you'll see it daily. Connect power and you're done.",
    visual: (
      <div className="relative w-full aspect-square max-w-sm mx-auto">
        {/* Kitchen counter scene */}
        <div className="absolute inset-0 bg-[var(--neutral-100)] dark:bg-[var(--neutral-800)] rounded-2xl" />

        {/* Device on counter */}
        <motion.div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
          animate={{ y: [0, -4, 0] }}
          transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="w-44 h-28 bg-white rounded-xl shadow-elevated flex items-center p-3">
            <div className="w-24 h-18 bg-[var(--neutral-900)] rounded-lg mr-2" />
            <div className="w-9 h-9 rounded-full bg-[var(--neutral-800)]" />
          </div>
        </motion.div>

        {/* Decorative elements */}
        <div className="absolute bottom-8 left-8 w-10 h-12 bg-white rounded-lg shadow-subtle" />
        <div className="absolute bottom-8 right-8">
          <div className="w-6 h-10 bg-[var(--status-upcoming)] rounded-t-full" />
          <div className="w-8 h-5 bg-[var(--neutral-400)] rounded-lg -mt-1" />
        </div>
      </div>
    ),
  },
  {
    number: "02",
    title: "Add your tasks",
    description: "Scan the QR code to open the mobile interface. Add tasks like 'Water plants every 3 days' or 'Change air filter monthly'. Takes 30 seconds.",
    visual: (
      <div className="relative w-full aspect-square max-w-sm mx-auto">
        {/* Phone mockup */}
        <motion.div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-48 h-[380px] bg-[var(--neutral-900)] rounded-[2.5rem] p-2 shadow-floating"
          animate={{ rotateY: [0, 3, 0] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="w-full h-full bg-white rounded-[2rem] overflow-hidden">
            {/* Status bar */}
            <div className="h-7 bg-[var(--neutral-100)] flex items-center justify-center">
              <div className="w-16 h-4 bg-[var(--neutral-900)] rounded-full" />
            </div>

            {/* App content */}
            <div className="p-4">
              <div className="text-base font-semibold text-[var(--neutral-900)] mb-4">Add Task</div>

              {/* Form fields */}
              <div className="space-y-3">
                <div className="h-9 bg-[var(--neutral-100)] rounded-lg px-3 flex items-center">
                  <span className="text-[var(--neutral-500)] text-sm">Water plants</span>
                </div>
                <div className="h-9 bg-[var(--neutral-100)] rounded-lg px-3 flex items-center justify-between">
                  <span className="text-[var(--neutral-500)] text-sm">Every 3 days</span>
                  <span className="text-[var(--accent)]">▼</span>
                </div>
                <motion.div
                  className="h-10 bg-[var(--accent)] rounded-lg flex items-center justify-center"
                  animate={{ scale: [1, 1.02, 1] }}
                  transition={{ duration: 2, repeat: Infinity }}
                >
                  <span className="text-white font-medium text-sm">Save Task</span>
                </motion.div>
              </div>

              {/* Existing tasks preview */}
              <div className="mt-5 space-y-2">
                {["Air filter", "Pet grooming", "Car service"].map((task, i) => (
                  <div key={task} className="flex items-center gap-2 p-2 bg-[var(--neutral-50)] rounded-lg">
                    <div className={`w-1.5 h-1.5 rounded-full ${i === 0 ? "bg-[var(--status-overdue)]" : i === 1 ? "bg-[var(--status-today)]" : "bg-[var(--status-upcoming)]"}`} />
                    <span className="text-xs text-[var(--neutral-700)]">{task}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    ),
  },
  {
    number: "03",
    title: "Stay on track",
    description: "Just glance at DaysTracker throughout your day. Color-coded urgency shows you what needs attention. Turn the knob to see details and mark tasks complete.",
    visual: (
      <div className="relative w-full aspect-square max-w-sm mx-auto flex items-center justify-center">
        {/* Central device with glowing screen */}
        <motion.div
          className="relative"
          animate={{ scale: [1, 1.01, 1] }}
          transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="w-56 h-36 bg-white rounded-xl shadow-floating p-4 flex items-center">
            {/* Screen with active display */}
            <div className="w-32 h-24 bg-[var(--neutral-900)] rounded-lg overflow-hidden mr-3">
              <div className="p-2 h-full flex flex-col">
                <div className="text-[var(--accent)] text-[7px] font-medium tracking-wider mb-1">OVERDUE</div>
                <div className="flex-1 space-y-1">
                  <div className="bg-[var(--status-overdue)]/20 rounded p-1.5">
                    <div className="text-white text-[8px] font-medium">Water plants</div>
                    <div className="text-[var(--status-overdue)] text-[6px]">2 days overdue</div>
                  </div>
                  <div className="bg-[var(--status-today)]/20 rounded p-1.5">
                    <div className="text-white text-[8px] font-medium">Check smoke detector</div>
                    <div className="text-[var(--status-today)] text-[6px]">Due today</div>
                  </div>
                </div>
              </div>
            </div>

            {/* Rotary encoder with rotation indicator */}
            <motion.div
              className="w-12 h-12 rounded-full bg-[var(--neutral-800)] flex items-center justify-center shadow-lg"
              animate={{ rotate: [0, 12, 0, -12, 0] }}
              transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
            >
              <div className="w-8 h-8 rounded-full bg-[var(--neutral-700)] flex items-center justify-center">
                <div className="w-0.5 h-3 bg-[var(--neutral-400)] rounded-full" />
              </div>
            </motion.div>
          </div>
        </motion.div>

        {/* Floating checkmarks */}
        <motion.div
          className="absolute top-6 right-10 w-8 h-8 bg-[var(--status-upcoming)] rounded-full flex items-center justify-center text-white shadow-elevated"
          animate={{ y: [0, -8, 0], opacity: [0.6, 1, 0.6] }}
          transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
          </svg>
        </motion.div>

        <motion.div
          className="absolute bottom-10 left-6 w-6 h-6 bg-[var(--status-upcoming)] rounded-full flex items-center justify-center text-white shadow-elevated"
          animate={{ y: [0, 6, 0], opacity: [0.4, 0.8, 0.4] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut", delay: 1 }}
        >
          <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
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
      className="grid lg:grid-cols-2 gap-12 lg:gap-16 items-center"
      initial={{ opacity: 0, y: 60 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 60 }}
      transition={{ duration: 0.7, delay: 0.1 }}
    >
      <div className={`${index % 2 === 1 ? "lg:order-2" : ""}`}>
        <div className="text-6xl font-serif text-[var(--accent)] mb-4">
          {step.number}
        </div>
        <h3 className="text-3xl md:text-4xl font-semibold mb-4">{step.title}</h3>
        <p className="text-lg text-[var(--muted)] leading-relaxed">
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
      className="py-32 bg-[var(--background)]"
    >
      <div className="max-w-6xl mx-auto px-6">
        {/* Section header */}
        <div className="text-center mb-24">
          <Badge className="mb-4">How It Works</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl mb-6">
            <span className="font-serif italic text-[var(--accent)]">Simple as</span>
            <br />
            <span className="font-semibold">one, two, three.</span>
          </h2>
          <p className="text-lg text-[var(--muted)] max-w-xl mx-auto">
            No complicated setup. No learning curve. Just place it, add tasks, and let it work for you.
          </p>
        </div>

        {/* Steps */}
        <div className="space-y-28">
          {steps.map((step, index) => (
            <Step key={step.number} step={step} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}
