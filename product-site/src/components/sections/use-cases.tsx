"use client";

import { useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Badge } from "../ui/badge";
import { Card } from "../ui/card";

const useCases = [
  {
    category: "Home Care",
    tasks: [
      "Change air filters every 30 days",
      "Deep clean kitchen weekly",
      "Vacuum every 3 days",
      "Clean refrigerator monthly",
    ],
  },
  {
    category: "Plants & Garden",
    tasks: [
      "Water indoor plants every 3 days",
      "Fertilize every 2 weeks",
      "Rotate plants monthly",
      "Check for pests weekly",
    ],
  },
  {
    category: "Pet Care",
    tasks: [
      "Groom dog every month",
      "Flea treatment every 30 days",
      "Vet checkup every 6 months",
      "Nail trim every 2 weeks",
    ],
  },
  {
    category: "Vehicle",
    tasks: [
      "Oil change every 3 months",
      "Tire rotation every 6 months",
      "Car wash every 2 weeks",
      "Check tire pressure monthly",
    ],
  },
  {
    category: "Health & Wellness",
    tasks: [
      "Take vitamins daily",
      "Dentist checkup every 6 months",
      "Replace toothbrush every 3 months",
      "Eye exam yearly",
    ],
  },
  {
    category: "Appliances",
    tasks: [
      "Replace water filter every 6 months",
      "Descale coffee maker monthly",
      "Clean dishwasher monthly",
      "Check smoke detectors yearly",
    ],
  },
];

function UseCaseCard({ useCase, index }: { useCase: typeof useCases[0]; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
    >
      <Card className="h-full">
        {/* Header */}
        <h3 className="text-sm font-medium text-[var(--accent)] mb-4 tracking-wide">
          {useCase.category}
        </h3>

        {/* Tasks */}
        <ul className="space-y-3">
          {useCase.tasks.map((task, i) => (
            <motion.li
              key={task}
              className="flex items-start gap-3"
              initial={{ opacity: 0, x: -12 }}
              animate={isInView ? { opacity: 1, x: 0 } : { opacity: 0, x: -12 }}
              transition={{ duration: 0.3, delay: index * 0.1 + i * 0.08 + 0.2 }}
            >
              <div className="w-1.5 h-1.5 rounded-full mt-2 bg-[var(--accent)]" />
              <span className="text-[var(--muted)] text-sm">{task}</span>
            </motion.li>
          ))}
        </ul>
      </Card>
    </motion.div>
  );
}

export function UseCases() {
  const containerRef = useRef(null);
  const isInView = useInView(containerRef, { once: true, margin: "-100px" });

  return (
    <section
      id="use-cases"
      ref={containerRef}
      className="py-32 bg-[var(--background)]"
    >
      <div className="max-w-6xl mx-auto px-6">
        {/* Section header */}
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
          transition={{ duration: 0.6 }}
        >
          <Badge className="mb-4">Use Cases</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl mb-6">
            <span className="font-serif italic text-[var(--accent)]">Track everything</span>
            <br />
            <span className="font-semibold">that matters.</span>
          </h2>
          <p className="text-lg text-[var(--muted)] max-w-xl mx-auto">
            From household chores to personal health, DaysTracker keeps all your recurring tasks organized and visible.
          </p>
        </motion.div>

        {/* Use cases grid */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {useCases.map((useCase, index) => (
            <UseCaseCard key={useCase.category} useCase={useCase} index={index} />
          ))}
        </div>

        {/* Bottom CTA */}
        <motion.div
          className="text-center mt-16"
          initial={{ opacity: 0, y: 15 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
        >
          <p className="text-lg text-[var(--muted)] mb-2">
            And so much more...
          </p>
          <p className="text-lg font-medium text-[var(--accent)]">
            If it repeats, DaysTracker tracks it.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
